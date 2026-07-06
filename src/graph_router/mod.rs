// Dependency graph resolver.
// Walks imports + FastAPI Depends() to find related files.
use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::ast_engine::extractor::extract_skeleton;
use crate::ast_engine::skeleton::FileSkeleton;

pub struct DependencyGraph {
    /// Maps a file path → list of files it directly imports.
    pub map: HashMap<String, Vec<String>>,
}

/// Given a starting `FileSkeleton`, resolve its imports to actual files
/// under `root_dir` and parse them into the graph.
pub fn resolve(skel: &FileSkeleton, root_dir: &Path) -> Result<DependencyGraph> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut deps: Vec<String> = Vec::new();

    for import in &skel.imports {
        let Some(module) = &import.module else {
            continue;
        };

        // Convert "app.routes" → path candidates
        let path_str = module.replace('.', "/");
        let base = root_dir.join(&path_str);

        let candidate = base.with_extension("py");
        let candidate = if candidate.exists() {
            candidate
        } else {
            base.join("__init__.py")
        };

        if candidate.exists() {
            let dep = extract_skeleton(&candidate)?;
            deps.push(dep.path);
        }
    }

    map.insert(skel.path.clone(), deps);
    Ok(DependencyGraph { map })
}
