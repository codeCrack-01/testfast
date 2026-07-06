use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::ast_engine::skeleton::FileSkeleton;
use crate::orchestrator::build_fix_prompt;
use crate::generator;

const MAX_RETRIES: u32 = 3;

/// Run pytest on the generated test file and return (passed, output).
fn run_pytest(project_dir: &Path) -> Result<(bool, String)> {
    let test_file = project_dir.join("tests").join("test_generated.py");
    if !test_file.exists() {
        anyhow::bail!("No test file found at {}", test_file.display());
    }

    let output = Command::new("python")
        .arg("-m")
        .arg("pytest")
        .arg(&test_file)
        .arg("-x")       // stop on first failure
        .arg("--tb=short")
        .arg("-q")        // quiet
        .current_dir(project_dir)
        .output()
        .context("Failed to run pytest")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stderr.is_empty() { stdout } else { format!("{stdout}\n{stderr}") };

    Ok((output.status.success(), combined))
}

pub fn auto_fix_loop(
    project_dir: &Path,
    main_file: &Path,
    source: &FileSkeleton,
    deps: &[FileSkeleton],
    api_key: &str,
    more: bool,
) -> Result<()> {
    for attempt in 1..=MAX_RETRIES {
        println!("\n--- Auto-fix attempt {attempt}/{MAX_RETRIES} ---");

        let (passed, output) = run_pytest(project_dir)?;
        if passed {
            println!("All tests passed!");
            return Ok(());
        }

        // Read current test code
        let test_path = project_dir.join("tests").join("test_generated.py");
        let current_code = std::fs::read_to_string(&test_path)
            .context("Failed to read generated test file")?;

        // Truncate pytest output to avoid blowing the token budget
        let truncated_output = if output.len() > 2000 {
            format!("{}... (truncated)", &output[..2000])
        } else {
            output.clone()
        };

        println!("Failures detected. Asking LLM to fix...");

        let fix_prompt = build_fix_prompt(source, deps, &current_code, &truncated_output, more)?;
        let fixed_code = crate::llm::generate(&fix_prompt.text, api_key)?;
        println!("Received fix ({} chars)", fixed_code.len());

        generator::save_tests(project_dir, main_file, &fixed_code)?;
    }

    // Final check
    let (passed, output) = run_pytest(project_dir)?;
    if passed {
        println!("All tests passed!");
    } else {
        println!("Auto-fix exhausted {MAX_RETRIES} attempts. Remaining failures:");
        println!("{output}");
        anyhow::bail!("Some tests still failing after {MAX_RETRIES} retries");
    }
    Ok(())
}
