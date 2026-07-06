# Session Log

## 2026-07-05 — Initial project scaffolding

**Description**: Created README.md, AGENTS.md, and SESSION_LOG.md to bootstrap future AI sessions with full project context.

**Files affected**:
- `README.md` — project overview, quick start, dependency table
- `AGENTS.md` — comprehensive AI agent instructions (conventions, commands, rules)
- `SESSION_LOG.md` — this file; changelog for tracking significant progress across sessions

## 2026-07-05 — Session 2: AST Engine implementation

**Description**: Refocused project toward FastAPI test generation with strict token optimization. Defined 4-component architecture (AST Engine, Graph Router, Context Manager, Orchestrator). Set up module directory structure. Implemented the AST Engine core: tree-sitter queries for Python imports, classes, decorated functions, and bare functions; `FileSkeleton` and `Import` data structures; `extract_skeleton()` function that parses a `.py` file, extracts structured imports, strips function bodies, and estimates token count. User (trainee) was guided through Rust concepts: pub visibility, Option, tree-sitter query API, Result/Context, string slicing. User started `print_skeleton_summary()` in main.rs but it's incomplete.

**Files created**:
- `src/ast_engine/mod.rs` — module declarations
- `src/ast_engine/queries.rs` — 5 tree-sitter query functions
- `src/ast_engine/skeleton.rs` — `Import` and `FileSkeleton` structs
- `src/ast_engine/extractor.rs` — `extract_skeleton()` with body-stripping logic
- `src/graph_router/mod.rs` — skeleton
- `src/context_mgr/mod.rs` — skeleton
- `src/orchestrator/mod.rs` — skeleton

**Files modified**:
- `src/main.rs` — added modules, started `print_skeleton_summary()`
- `AGENTS.md` — updated with architecture, current state, rules
- `README.md` — (from previous session)

## 2026-07-06 — Session 3: AST Engine end-to-end verified

**Description**: Completed `print_skeleton_summary()` in main.rs — now prints all four `FileSkeleton` fields (path, imports count, stripped source preview, token count). Wired it into `main()` via `extract_skeleton()` on a test Python file. Successfully ran end-to-end: parsed `test_app.py`, extracted 4 imports, stripped function bodies, estimated 91 tokens.

## 2026-07-06 — Session 4: Graph Router + Context Manager

**Description**: Implemented Graph Router (`src/graph_router/mod.rs`) — `DependencyGraph` struct and `resolve()` function that converts dotted module names to file paths and parses dependencies. Extended `FileSkeleton` with `functions: Vec<FnDef>` and `classes: Vec<ClassDef>`. Updated `extractor.rs` to populate function/class names from tree-sitter captures. Implemented Context Manager (`src/context_mgr/mod.rs`) — `find_deltas()` compares source function names against test files to find uncovered functions. End-to-end test confirmed `list_items` is correctly detected as uncovered.

**Files created**:
- `tests/test_app.py` — test fixture for context manager
- `test_app.py` — sample FastAPI app for testing

**Files modified**:
- `src/ast_engine/skeleton.rs` — added `FnDef`, `ClassDef` structs; `functions`/`classes` fields
- `src/ast_engine/queries.rs` — simplified queries to not require optional `return_type`
- `src/ast_engine/extractor.rs` — populates functions and classes from captures
- `src/graph_router/mod.rs` — full implementation
- `src/context_mgr/mod.rs` — full implementation
- `src/main.rs` — wired all three components, truncation-safe preview
- `AGENTS.md` — (updated below)

## 2026-07-06 — Session 4 (cont'd): tiktoken tokenization + orchestrator

**Description**: Replaced `len/4` heuristic with real tiktoken-rs tokenization using `cl100k_base` encoding in both `extractor.rs` and `orchestrator/mod.rs`. Wired Orchestrator into main pipeline. Full end-to-end pipeline produces accurate token counts.

**Files modified**:
- `src/ast_engine/extractor.rs` — tiktoken for source token count
- `src/orchestrator/mod.rs` — full implementation + tiktoken for prompt token count
- `src/main.rs` — wired orchestrator call

## 2026-07-06 — Session 5: File-hash cache

**Description**: Added persistent file-hash cache (`.coderag/cache.json`) to avoid re-parsing unchanged files. Uses SHA256 content hash to detect modifications. Cache module (`src/cache.rs`) with `SkeletonCache` struct providing `get_or_extract()`, used by both `main.rs` and `graph_router::resolve()`. Cache is committed to the repo for session persistence. Serialization via serde+serde_json.

**Files created**:
- `src/cache.rs` — cache module with `SkeletonCache`

**Files modified**:
- `Cargo.toml` — added serde, serde_json, sha2
- `src/ast_engine/skeleton.rs` — serde derives + Clone
- `src/ast_engine/queries.rs` — fixed import_from_query for relative imports
- `src/graph_router/mod.rs` — recursive resolution, cache integration
- `src/main.rs` — wired cache
- `src/orchestrator/mod.rs` — tiktoken token counting
- `src/ast_engine/extractor.rs` — tiktoken integration
- `SESSION_LOG.md`, `AGENTS.md` — updated
