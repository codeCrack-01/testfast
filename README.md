# coderag

A Rust-based code understanding & RAG tool for generating automated test suites for FastAPI (Python) web applications. Optimized for minimal LLM token consumption.

## Architecture

| Component | Status | Purpose |
|---|---|---|
| **AST Engine** | ✅ Done | Tree-sitter parsing → stripped AST skeletons |
| **Graph Router** | 🔜 Next | Import/Dependency resolution |
| **Context Manager** | ⏳ | Coverage delta detection |
| **Orchestrator** | ⏳ | LLM prompt assembly |

## Quick Start

```bash
cargo build
cargo run
```

## Project Layout

```
src/
  main.rs               # Entry point
  ast_engine/           # Tree-sitter parsing → AST skeletons
    queries.rs          # Query strings (import, class, func, decorator)
    skeleton.rs         # FileSkeleton, Import structs
    extractor.rs        # File → stripped skeleton
  graph_router/         # Dependency graph (imports + Depends())
  context_mgr/          # "The Token Miser"
  orchestrator/         # LLM prompt assembler
```

## Dependencies

| Crate | Purpose |
|---|---|
| tree-sitter 0.22 | AST parsing |
| tree-sitter-python 0.21 | Python grammar |
| tiktoken-rs 0.5 | LLM tokenization (not yet wired) |
| tokio 1.0 | Async runtime |
| anyhow 1.0 | Error handling |
