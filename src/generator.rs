use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Strip markdown code fences from LLM output.
fn strip_markdown_fences(raw: &str) -> String {
    let mut code = raw.trim();
    // Strip opening ```python or ``` (with optional whitespace after)
    if let Some(rest) = code.strip_prefix("```python") {
        code = rest;
    } else if let Some(rest) = code.strip_prefix("```") {
        code = rest;
    }
    let code = code.trim();
    // Strip trailing ```
    let code = if let Some(pos) = code.rfind("```") {
        code[..pos].trim()
    } else {
        code
    };
    code.to_string()
}

/// Save LLM-generated test code to the project's `tests/` directory.
/// Also creates `tests/conftest.py` that adds the project root to sys.path.
pub fn save_tests(project_dir: &Path, source_path: &Path, code: &str) -> Result<()> {
    let code = strip_markdown_fences(code);
    let tests_dir = project_dir.join("tests");
    if !tests_dir.exists() {
        fs::create_dir_all(&tests_dir)
            .with_context(|| format!("Failed to create {}", tests_dir.display()))?;
    }

    let module_name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("app");

    let conftest = tests_dir.join("conftest.py");
    if !conftest.exists() {
        let conftest_content = format!(
            r##"import sys
from pathlib import Path

# Add project root to sys.path so `from main import app` works
sys.path.insert(0, str(Path(__file__).parent.parent))

import pytest
from fastapi.testclient import TestClient
from {module_name} import app

# Enable pytest.mark.anyio for async test support
pytest_plugins = ("anyio",)


@pytest.fixture
def client() -> TestClient:
    return TestClient(app)
"##
        );
        fs::write(&conftest, conftest_content)
            .with_context(|| format!("Failed to write {}", conftest.display()))?;
        println!("Created {}", conftest.display());
    }

    let test_file = tests_dir.join("test_generated.py");
    fs::write(&test_file, code)
        .with_context(|| format!("Failed to write {}", test_file.display()))?;
    println!("Generated {}", test_file.display());

    Ok(())
}
