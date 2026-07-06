use std::env;
use std::path::Path;

use anyhow::Result;
use clap::Parser;

use ast_engine::skeleton::FileSkeleton;
use cache::SkeletonCache;
use cli::Cli;
use context_mgr::{find_deltas, CoverageDelta};
use graph_router::resolve;
use orchestrator::{build_prompt, Prompt};
use test_agent::TestAgent;

mod ast_engine;
mod autofix;
mod cache;
mod cli;
mod context_mgr;
mod generator;
mod graph_router;
mod llm;
mod orchestrator;
mod test_agent;

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

/// Shared pipeline: parse, resolve, detect deltas, build prompt.
/// Returns cache, prompt, delta, and agent for post-processing.
fn run_pipeline(project_root: &Path, more: bool) -> Result<(SkeletonCache, Prompt, CoverageDelta, TestAgent, String, FileSkeleton)> {
    let mut cache = SkeletonCache::load();
    let agent = TestAgent::load(project_root);

    // Discover the main .py file — look for app/fastapi patterns
    let source_path = find_main_file(project_root)?;
    let skel = cache.get_or_extract(&source_path)?;
    println!("=== Source ===");
    print_skeleton_summary(&skel);

    let (_graph, dep_skels) = resolve(&skel, project_root, &mut cache)?;
    println!("\n=== Dependencies ({}) ===", dep_skels.len());
    for dep in &dep_skels {
        println!("  {} ({} functions)", dep.path, dep.functions.len());
    }

    // Find existing test files in the project
    let test_skels = find_test_files(project_root, &mut cache)?;
    if !test_skels.is_empty() {
        println!("\n=== Tests ===");
        for t in &test_skels {
            println!("  {} ({} functions)", t.path, t.functions.len());
        }
    }

    // Coverage is determined by actual test file inspection (agent memory not used)
    let delta = find_deltas(&skel, &test_skels, &[]);
    println!("\n=== Uncovered ===");
    for f in &delta.uncovered {
        let kind = if f.is_decorated { "decorated" } else { "bare" };
        println!("  {} ({})", f.name, kind);
    }

    if delta.uncovered.is_empty() {
        println!("\n(Nothing to test — all functions covered by test files)");
    }

    let prompt = build_prompt(&skel, &delta, agent.style_context(), more)?;
    println!("\n=== Generated Prompt ({} tokens) ===", prompt.token_count);

    Ok((cache, prompt, delta, agent, source_path.to_string_lossy().to_string(), skel))
}

fn load_env(project_dir: &Path) {
    // Try .env from CWD first, then project dir
    let _ = dotenvy::dotenv();
    let project_env = project_dir.join(".env");
    if project_env.exists() {
        let _ = dotenvy::from_path(&project_env);
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("testfast {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let project_root = Path::new(&cli.path);
    if !project_root.exists() {
        anyhow::bail!("Project directory '{}' not found", cli.path);
    }

    load_env(project_root);

    if cli.re {
        println!("WARNING: --re will delete existing test files and coverage memory.");
        print!("Are you sure? (y/N): ");
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
        // Remove generated test file, conftest, and agent memory
        let _ = std::fs::remove_file(project_root.join("tests").join("test_generated.py"));
        let _ = std::fs::remove_file(project_root.join("tests").join("conftest.py"));
        let _ = std::fs::remove_file(project_root.join("test_agent.md"));
        println!("Cleared existing tests and coverage memory.");
    }

    let (cache, prompt, delta, mut agent, main_file, source) = run_pipeline(project_root, cli.more)?;

    if delta.uncovered.is_empty() {
        println!("All functions covered — nothing to generate.");
        cache.save()?;
        return Ok(());
    }

    if cli.pretend {
        println!("{}", &prompt.text);
        println!("--- ({} tokens) ---", prompt.token_count);
        println!("\n[pretend] No files were touched.");
        return Ok(());
    }

    let api_key = match env::var("LLM_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: LLM_KEY not found.");
            eprintln!("Set it in .env, project/.env, or as an environment variable.");
            eprintln!("Example: echo 'LLM_KEY=sk-...' > .env");
            anyhow::bail!("LLM_KEY is required (use --pretend to dry-run without it)");
        }
    };

    println!("\nCalling LLM...");
    let generated_code = llm::generate(&prompt.text, &api_key)?;
    println!("Received response ({} chars)", generated_code.len());

    generator::save_tests(project_root, Path::new(&main_file), &generated_code)?;

    if !cli.no_auto {
        autofix::auto_fix_loop(project_root, Path::new(&main_file), &source, &api_key, cli.more)?;
    }

    for f in &delta.uncovered {
        agent.record_coverage(&f.name);
    }
    agent.save()?;

    cache.save()?;
    println!("Done.");
    Ok(())
}

/// Find the main FastAPI entry point in the project.
/// Looks for files containing `FastAPI()` or `APIRouter()`.
fn find_main_file(root: &Path) -> Result<std::path::PathBuf> {
    // If the user specified a source, use it
    // Otherwise scan for the most likely file
    for entry in walk_py_files(root) {
        let content = std::fs::read_to_string(&entry).ok();
        if let Some(text) = content
            && (text.contains("FastAPI()") || text.contains("FastAPI("))
        {
            return Ok(entry);
        }
    }
    anyhow::bail!(
        "No FastAPI app found in {}. Does a file contain `app = FastAPI()`?",
        root.display()
    );
}

/// Walk a directory recursively for .py files.
fn walk_py_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden dirs, venvs, site-packages, node_modules, __pycache__
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.')
                && name != "__pycache__"
                && name != "node_modules"
                && name != "venv"
                && name != "env"
                && name != "site-packages"
                && name != "dist-packages"
            {
                files.extend(walk_py_files(&path));
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("py") {
            files.push(path);
        }
    }
    files
}

/// Find all test files (named `test_*.py` or in `tests/` dir) under the project.
fn find_test_files(root: &Path, cache: &mut SkeletonCache) -> Result<Vec<FileSkeleton>> {
    let mut skeletons = Vec::new();
    for path in walk_py_files(root) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if (name.starts_with("test_") || path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) == Some("tests"))
            && let Ok(skel) = cache.get_or_extract(&path)
        {
            skeletons.push(skel);
        }
    }
    Ok(skeletons)
}
