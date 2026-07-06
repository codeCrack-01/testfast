// Dependency graph resolver.
// Walks imports + FastAPI Depends() to find related files.
use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;

use crate::ast_engine::skeleton::FileSkeleton;
use crate::cache::SkeletonCache;

pub struct DependencyGraph {
    /// Maps a file path → list of files it directly imports.
    pub map: HashMap<String, Vec<String>>,
}

/// Convert a dotted module name to a candidate file path.
/// `parent_path` is the file doing the importing (used for relative imports).
/// `root_dir` is the project root for absolute imports.
fn module_to_path(
    module: &str,
    parent_path: &Path,
    root_dir: &Path,
) -> Option<std::path::PathBuf> {
    let path_str = module.replace('.', "/");

    let base = if module.starts_with('.') {
        // Relative import: resolve from the parent file's directory
        let stripped = path_str.trim_start_matches('/');
        parent_path.parent()?.join(stripped)
    } else {
        root_dir.join(&path_str)
    };

    let as_file = base.with_extension("py");
    if as_file.exists() {
        return Some(as_file);
    }

    let as_init = base.join("__init__.py");
    if as_init.exists() {
        return Some(as_init);
    }

    None
}

/// Recursively resolve imports: BFS through the dependency tree starting from
/// the source file's imports. Returns the graph and all parsed dependency
/// skeletons (including transitive dependencies).
pub fn resolve(
    skel: &FileSkeleton,
    root_dir: &Path,
    cache: &mut SkeletonCache,
) -> Result<(DependencyGraph, Vec<FileSkeleton>)> {
    let mut dep_skeletons: Vec<FileSkeleton> = Vec::new();
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<(String, String)> = Vec::new(); // (parent_path, module)

    // Seed the queue with the source file's imports
    let source_path = skel.path.clone();
    for import in &skel.imports {
        if let Some(module) = &import.module {
            queue.push((source_path.clone(), module.clone()));
        }
    }

    while let Some((parent, module)) = queue.pop() {
        if !visited.insert(module.clone()) {
            continue;
        }

        let Some(candidate) = module_to_path(&module, Path::new(&parent), root_dir) else {
            continue;
        };

        let dep = match cache.get_or_extract(&candidate) {
            Ok(skel) => skel,
            Err(e) => {
                eprintln!("Warning: skipped {}: {e}", candidate.display());
                continue;
            }
        };

        let dep_path = dep.path.clone();

        // Record the edge: parent → dep
        graph.entry(parent).or_default().push(dep_path.clone());

        // Queue up this dependency's own imports
        for import in &dep.imports {
            if let Some(sub_module) = &import.module
                && !visited.contains(sub_module)
            {
                queue.push((dep_path.clone(), sub_module.clone()));
            }
        }

        dep_skeletons.push(dep);
    }

    // Ensure the source file is in the graph (even with no deps)
    graph.entry(source_path).or_default();

    Ok((DependencyGraph { map: graph }, dep_skeletons))
}
