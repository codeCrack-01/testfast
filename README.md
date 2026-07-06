# testfast

Automated test suite generator for FastAPI (Python) web applications. Parses your FastAPI project, detects uncovered functions, and generates pytest tests via LLM (OpenAI, Anthropic, Groq, Gemini, or any OpenAI-compatible provider).

## Features

- **AST-aware parsing** — understands FastAPI routes, dependencies, and imports via tree-sitter
- **Smart prompting** — sends only the context the LLM needs (function signatures + key lines like `return`/`raise`/`await`)
- **Auto-fix loop** — runs pytest after generation and retries the LLM up to 3x if tests fail
- **Multi-provider** — OpenAI, Anthropic, Groq, Gemini, or any OpenAI-compatible API
- **Auto-detection** — detects the LLM provider from your API key prefix (`sk-`, `sk-ant-`, `gsk_`, `AIza`)
- **Dependency resolution** — recursively resolves FastAPI `Depends()` and Python imports
- **Coverage deltas** — compares source functions against existing test files
- **File-hash cache** — avoids re-parsing unchanged files (`.coderag/cache.json`)

## Installation

### From source (requires Rust)

```bash
cargo install --git https://github.com/codeCrack-01/testfast
```

### Pre-built binaries

Download the latest binary for your platform from the [Releases](https://github.com/codeCrack-01/testfast/releases) page.

| Platform | Binary |
|---|---|
| Linux x86_64 | `testfast-x86_64-unknown-linux-gnu` |
| macOS Intel | `testfast-x86_64-apple-darwin` |
| macOS Apple Silicon | `testfast-aarch64-apple-darwin` |
| Windows x86_64 | `testfast-x86_64-pc-windows-msvc.exe` |

## Quick Start

```bash
# Set up your API key
export LLM_KEY=sk-...   # or gsk_..., sk-ant-..., AIza...

# Generate tests for your FastAPI project
testfast ./my_fastapi_project

# Run the generated tests
cd my_fastapi_project && pytest tests/
```

That's it. `testfast` will:
1. Find your FastAPI app entry point
2. Parse all functions and imports
3. Find existing tests (if any)
4. Generate tests for uncovered functions
5. Run the tests and auto-fix any failures (up to 3 retries)

## CLI Reference

```
Usage: testfast [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to the FastAPI project [default: .]

Options:
  -v, --version    Show version and exit
      --pretend    Dry-run: show the prompt without calling LLM or saving anything
      --more       Include full source code (not just key lines) for richer context
      --re         Regenerate all tests from scratch (deletes existing tests + agent memory)
      --no-auto    Disable the auto-fix loop (auto-fix is enabled by default)
  -h, --help       Print help
```

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `LLM_KEY` | Yes | — | API key (auto-detects provider from prefix) |
| `LLM_PROVIDER` | No | auto | Force a provider: `openai`, `anthropic`, `groq`, `gemini` |
| `LLM_MODEL` | No | per-provider | Model name (e.g. `gpt-4o`, `claude-sonnet-4-20250514`) |
| `LLM_BASE_URL` | No | per-provider | Custom API base URL for OpenAI-compatible providers |
| `LLM_TIMEOUT` | No | `120` | HTTP request timeout in seconds |

### Provider defaults

| Provider | Key prefix | Default model | Base URL |
|---|---|---|---|
| OpenAI | `sk-...` | `gpt-4o` | `https://api.openai.com/v1/chat/completions` |
| Anthropic | `sk-ant-...` | `claude-sonnet-4-20250514` | `https://api.anthropic.com/v1/messages` |
| Groq | `gsk_...` | `llama-3.3-70b-versatile` | `https://api.groq.com/openai/v1/chat/completions` |
| Gemini | `AIza...` | `gemini-3.0-flash-preview` | `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions` |

### Quick `.env` setup

Copy-paste for a Gemini setup:

```env
LLM_KEY=AIza...
LLM_PROVIDER=gemini
LLM_MODEL=gemini-3.0-flash-preview
LLM_TIMEOUT=300
```

Or for OpenAI:

```env
LLM_KEY=sk-...
LLM_MODEL=gpt-4o
```

Or any OpenAI-compatible endpoint (local models, etc.):

```env
LLM_KEY=noop
LLM_BASE_URL=http://localhost:8000/v1/chat/completions
LLM_MODEL=my-model
```

## How It Works

1. **Discover** — scans the project for the main FastAPI file (contains `FastAPI()`)
2. **Parse** — extracts imports, function signatures, decorators, classes using tree-sitter
3. **Resolve** — follows imports and `Depends()` calls recursively
4. **Compare** — checks which functions already have tests (`test_<name>` in existing test files)
5. **Generate** — builds a prompt and sends it to the LLM with uncovered function signatures
6. **Auto-fix** — runs `pytest` and feeds failures back to the LLM (up to 3 retries)
7. **Save** — writes `tests/test_generated.py` and `tests/conftest.py`

### Prompt modes

| Mode | What the LLM sees | Token cost |
|---|---|---|
| Default | Signatures + `return`/`raise`/`await` lines from function bodies | Low |
| `--more` | Full raw source code (unmodified) | High |

## Project Structure

```
tests/test_generated.py   # Generated test file (re-generated each run)
tests/conftest.py          # Pytest fixtures (created once, then kept)
test_agent.md              # Test style guide + coverage memory (created once)
.coderag/cache.json        # File-hash cache (committed to repo)
```

## Examples

### Basic usage

```bash
# Generate tests with auto-fix (default)
testfast ./my_project
```

### Full source context + fresh generation

```bash
testfast --more --re ./my_project
```

### Dry-run to preview the prompt

```bash
testfast --pretend ./my_project
```

### Custom model

```bash
LLM_MODEL=gemini-2.5-flash testfast --more ./my_project
```

### Custom API endpoint (e.g. local model)

```bash
LLM_BASE_URL=http://localhost:8000/v1/chat/completions LLM_MODEL=my-model testfast ./my_project
```

## Requirements

- **FastAPI project**: your app must instantiate `app = FastAPI(...)` somewhere in a `.py` file
- **Python**: `pytest` and `anyio` must be installed to run the auto-fix loop
- **Rust**: only needed to build from source (pre-built binaries available)

## License

MIT
