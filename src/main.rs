use std::path::Path;

use anyhow::Result;

use ast_engine::skeleton::FileSkeleton;
use cache::SkeletonCache;
use context_mgr::find_deltas;
use graph_router::resolve;
use orchestrator::build_prompt;

mod ast_engine;
mod cache;
mod context_mgr;
mod graph_router;
mod orchestrator;

fn print_skeleton_summary(skel: &FileSkeleton) {
    println!("Path: {}", &skel.path);
    println!("Imports: {}", &skel.imports.len());
    print!("Functions: ");
    for f in &skel.functions {
        let kind = if f.is_decorated { "decorated" } else { "bare" };
        print!("{}({}) ", f.name, kind);
    }
    println!();
    print!("Classes: ");
    for c in &skel.classes {
        print!("{} ", c.name);
    }
    println!();
    let preview = if skel.source_text.len() > 200 {
        &skel.source_text[..200]
    } else {
        &skel.source_text
    };
    println!("Source preview: {}", preview);
    println!("Estimated tokens: {}", &skel.token_count);
}

fn main() -> Result<()> {
    let mut cache = SkeletonCache::load();

    let source_path = Path::new("tests/overleaf_mcp/overleaf_mcp.py");
    let project_root = Path::new("tests/overleaf_mcp");

    let skel = cache.get_or_extract(source_path)?;
    println!("=== Source ===");
    print_skeleton_summary(&skel);

    let (_graph, dep_skels) = resolve(&skel, project_root, &mut cache)?;
    println!("\n=== Dependencies ({}) ===", dep_skels.len());
    for dep in &dep_skels {
        println!("  {} ({} functions)", dep.path, dep.functions.len());
    }

    let test_path = Path::new("tests/test_app.py");
    let test_skel = cache.get_or_extract(test_path)?;
    println!("\n=== Tests ===");
    print_skeleton_summary(&test_skel);

    let delta = find_deltas(&skel, &[test_skel]);
    println!("\n=== Uncovered ===");
    for f in &delta.uncovered {
        let kind = if f.is_decorated { "decorated" } else { "bare" };
        println!("  {} ({})", f.name, kind);
    }

    let prompt = build_prompt(&skel, &delta)?;
    println!("\n=== Generated Prompt ===");
    println!("{}", &prompt.text);
    println!("--- ({} tokens) ---", prompt.token_count);

    cache.save()?;
    println!("\nCache saved to .coderag/cache.json");
    Ok(())
}
