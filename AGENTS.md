# Agent Instructions for coderag

This file provides context for AI coding assistants working on this project.

## Project Overview

**coderag** is a Rust-based code understanding & RAG tool, specifically optimized for generating automated test suites for FastAPI (Python) web applications. Primary constraint: strict **token optimization** to minimize Cloud LLM API costs.

## Architecture

```
src/
  main.rs               # Entry point — thin, delegates to modules
  ast_engine/           # Tree-sitter parsing → AST skeletons
    mod.rs              # Re-exports
    queries.rs          # Tree-sitter query strings (import, class, func, decorator)
    skeleton.rs         # Data structures: FileSkeleton, Import
    extractor.rs        # extract_skeleton(): file → FileSkeleton with bodies stripped
  graph_router/         # (Not yet implemented)
    mod.rs              # Dependency graph resolver (imports + FastAPI Depends())
  context_mgr/          # (Not yet implemented)
    mod.rs              # "The Token Miser" — coverage delta detection
  orchestrator/         # (Not yet implemented)
    mod.rs              # Async LLM prompt assembler
```

## Current State (Session 2)

### ✅ Implemented — AST Engine
- **`queries.rs`**: 5 tree-sitter queries — `import_query`, `import_from_query`, `class_query`, `decorated_query`, `function_query`
- **`skeleton.rs`**: `Import` struct (pub fields: `module`, `name` as `Option<String>`) and `FileSkeleton` struct (`path`, `imports`, `source_text`, `token_count`)
- **`extractor.rs`**: `extract_skeleton(path)` — reads a `.py` file, parses with tree-sitter, extracts structured imports, strips function bodies (replaces with `...`), estimates token count

### 🔜 Next Up
- **`main.rs`**: User started writing `print_skeleton_summary()` but it only prints the path. Needs to be completed to test the AST engine end-to-end.
- **Graph Router**: Walk imports from `FileSkeleton.imports` to resolve file dependencies
- **Context Manager**: Compare existing tests vs source skeletons to find coverage deltas
- **Orchestrator**: Package compressed context + deltas into LLM prompt

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
| `cargo run`     | Run the binary                 |
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
