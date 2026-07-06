use std::path::Path;

use anyhow::Result;

use ast_engine::extractor::extract_skeleton;
use ast_engine::skeleton::FileSkeleton;

mod ast_engine;
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
    println!("Source preview: {}", &skel.source_text[..200]);
    println!("Estimated tokens: {}", &skel.token_count);
}

fn main() -> Result<()> {
    let skel = extract_skeleton(Path::new("test_app.py"))?;
    print_skeleton_summary(&skel);
    Ok(())
}
