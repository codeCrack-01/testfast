use clap::Parser;

#[derive(Parser)]
#[command(name = "testfast", about = "FastAPI test generator using LLM")]
pub struct Cli {
    /// Show version and exit
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    /// Dry-run: show the prompt without calling LLM or saving anything
    #[arg(long)]
    pub pretend: bool,

    /// Include full source context (not just signatures) for better test quality
    #[arg(long)]
    pub more: bool,

    /// Regenerate all tests from scratch (deletes existing test files and agent memory)
    #[arg(long)]
    pub re: bool,

    /// Disable auto-fix loop (tests are auto-fixed by default)
    #[arg(long)]
    pub no_auto: bool,

    /// Path to the FastAPI project (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: String,
}
