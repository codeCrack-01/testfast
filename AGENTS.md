# Agent Instructions for coderag

This file provides context for AI coding assistants working on this project.

## Project Overview

**coderag** is a Rust-based code understanding & RAG tool, specifically optimized for generating automated test suites for FastAPI (Python) web applications. Primary constraint: strict **token optimization** to minimize Cloud LLM API costs.

## Architecture

```
src/
  main.rs               # Entry point — thin, delegates to modules
  cache.rs              # File-hash cache (.coderag/cache.json)
  ast_engine/           # Tree-sitter parsing → AST skeletons
    mod.rs              # Re-exports
    queries.rs          # Tree-sitter query strings (import, class, func, decorator)
    skeleton.rs         # Data structures: FileSkeleton, Import
    extractor.rs        # extract_skeleton(): file → FileSkeleton with bodies stripped
  graph_router/         # Dependency graph resolver (imports + FastAPI Depends())
    mod.rs              # Full implementation with recursive BFS resolution
  context_mgr/          # "The Token Miser" — coverage delta detection
    mod.rs              # find_deltas(): source vs test comparison
  orchestrator/         # LLM prompt assembler
    mod.rs              # build_prompt(): source + deltas → prompt with tiktoken
```

## Current State (Session 4)

### ✅ Implemented — AST Engine
- **`queries.rs`**: 5 tree-sitter queries — `import_query`, `import_from_query`, `class_query`, `decorated_query`, `function_query`
- **`skeleton.rs`**: `Import` struct, `FnDef` struct (name + is_decorated), `ClassDef` struct, `FileSkeleton` struct (path, imports, functions, classes, source_text, token_count)
- **`extractor.rs`**: `extract_skeleton(path)` — reads a `.py` file, parses with tree-sitter, extracts structured imports + function/class names, strips function bodies, estimates token count

### ✅ Implemented — Graph Router
- **`graph_router/mod.rs`**: `DependencyGraph` struct (map of file → dependencies), `resolve()` function that converts dotted module names to file paths and parses dependencies

### ✅ Implemented — Context Manager ("The Token Miser")
- **`context_mgr/mod.rs`**: `CoverageDelta` struct (uncovered functions), `find_deltas()` — compares source function names against test files to find uncovered functions

### ✅ Implemented — Orchestrator
- **`orchestrator/mod.rs`**: `Prompt` struct (text + token_count), `build_prompt()` — packages source skeleton + coverage deltas into an LLM prompt with instructions

### 🎉 All 4 Components Done

The full pipeline is wired in `main.rs` and works end-to-end.

### 🔜 Next Up
- Further refinements (real tokenization with tiktoken, dependency graph integration, actual LLM API calls)

## Tech Stack & Conventions

- **Language**: Rust (edition 2024)
- **Async runtime**: tokio (full features)
- **Parsing**: tree-sitter 0.22 + tree-sitter-python 0.21
- **Tokenization**: tiktoken-rs 0.5 (not yet wired in — uses `len/4` heuristic)
- **Error handling**: anyhow (use `anyhow::Result`; prefer `bail!` / `context()`)
- **Style**: Follow standard Rust idioms (`cargo fmt`, `cargo clippy`)
- **No unsafe code** unless absolutely necessary and justified

## Commands

| Command         | Description                    |
|-----------------|--------------------------------|
| `cargo build`   | Build the project              |
| `cargo run --bin testfast -- --pretend [PATH]` | Run in dry-run mode |
| `cargo run --bin testfast -- -v` | Show version       |
| `cargo run --bin testfast -- --no-auto [PATH]` | Generate tests without auto-fix |
| `cargo run --bin testfast -- --more --re [PATH]` | Full source, fresh generation |
| `cargo test`    | Run tests                      |
| `cargo fmt`     | Format code                    |
| `cargo clippy`  | Lint                           |
| `cargo doc --open` | Build & open docs           |

## Architecture Guidelines

- Keep `main.rs` minimal — extract logic into modules under `src/`.
- Module pattern: `src/<module>/mod.rs` or `src/<module>.rs`.
- Public API surfaces should have doc comments.
- Tests go in a `tests` module at the bottom of each file or in `tests/` directory.

## Important Rules for the Agent

1. **Read AGENTS.md first** — this file is the source of truth for project context.
2. **Log significant changes** in `SESSION_LOG.md` (create/append):
   - When you add new files, dependencies, or major logic.
   - When you change architecture or project conventions.
   - When you fix bugs or complete features.
   - Each entry should have: date, description, files affected.
3. **Update AGENTS.md** if you change project structure, dependencies, conventions, or commands.
4. **Do not add comments to source code** unless the logic is non-obvious and the intent would be unclear without them.
5. **Do not create documentation files** (e.g., README, docs/) unless the user explicitly asks.
6. **Before writing code**, check existing patterns and conventions in the codebase.
7. **For Rust**: prefer `use`ing the parent module path rather than `super::` when possible; keep imports organized.
8. **Commit only when explicitly asked** — never commit without user request.
9. **This project is being built collaboratively with a trainee.** Explain Rust concepts, ask questions, and guide rather than just write code. Let the trainee write code whenever possible and review their work.
