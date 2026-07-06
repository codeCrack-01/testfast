# Contributing

Thanks for your interest! This is a small project and all contributions are welcome.

## Quick Start

```bash
git clone https://github.com/yourusername/coderag
cd coderag
cargo build
cargo test
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Run `cargo test` to make sure tests pass
- No unsafe code unless absolutely necessary and justified

## Project Structure

```
src/
  main.rs               # Entry point — thin CLI dispatcher
  cli.rs                # Clap argument definitions
  llm.rs                # LLM API calls (OpenAI, Anthropic, Groq, Gemini)
  generator.rs          # Saves generated tests + conftest
  autofix.rs            # Pytest runner + retry loop
  cache.rs              # File-hash cache (SHA256 + JSON)
  test_agent.rs         # test_agent.md persistence (style guide)
  ast_engine/           # Tree-sitter parsing
    queries.rs          # Tree-sitter query strings
    skeleton.rs         # Struct definitions
    extractor.rs        # File → FileSkeleton extraction
  graph_router/         # Import dependency resolution
  context_mgr/          # Coverage delta detection
  orchestrator/         # LLM prompt assembly
```

## Making Changes

1. Create a branch: `git checkout -b my-feature`
2. Make your changes
3. Run `cargo fmt && cargo clippy && cargo test`
4. Commit: `git commit -m "description of change"`
5. Push and open a pull request

## Adding a New LLM Provider

1. Add the variant to the `Provider` enum in `src/llm.rs`
2. Add auto-detection in `detect_provider()` (key prefix check)
3. Implement the generate function
4. Add the default model URL to the `generate()` match
5. Add to `README.md` provider table

## Release Process

Maintainers: to publish a new release:

```bash
# Update version in Cargo.toml
git tag v<new-version>
git push --tags
```

GitHub Actions builds binaries for all platforms and creates a release.

## Questions?

Open an issue or start a discussion on GitHub.
