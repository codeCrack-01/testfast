use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

const STYLE_TEMPLATE: &str = "\
- Framework: pytest
- Client: FastAPI TestClient
- Fixtures: none used so far
";

pub struct TestAgent {
    covered: Vec<String>,          // function names already tested
    style: String,                 // style guide from the file
    file_path: String,             // where to save
}

impl TestAgent {
    pub fn load(project_dir: &Path) -> Self {
        let file_path = project_dir.join("test_agent.md");
        let file_path_str = file_path.to_string_lossy().to_string();

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => return Self {
                covered: Vec::new(),
                style: STYLE_TEMPLATE.to_string(),
                file_path: file_path_str,
            },
        };

        let mut covered = Vec::new();
        let mut style = String::new();
        let mut in_style = false;

        for line in content.lines() {
            if line.trim() == "## Coverage" {
                in_style = false;
            } else if line.trim() == "## Style" {
                in_style = true;
            } else if let Some(func) = line.strip_prefix("- `").and_then(|s| s.split("` →").next()) {
                covered.push(func.trim().to_string());
            } else if in_style && line.starts_with("- ") {
                style.push_str(line);
                style.push('\n');
            }
        }

        if style.is_empty() {
            style = STYLE_TEMPLATE.to_string();
        }

        Self { covered, style, file_path: file_path_str }
    }

    #[allow(dead_code)]
    pub fn covered_functions(&self) -> &[String] {
        &self.covered
    }

    pub fn style_context(&self) -> &str {
        &self.style
    }

    pub fn record_coverage(&mut self, func_name: &str) {
        if !self.covered.contains(&func_name.to_string()) {
            self.covered.push(func_name.to_string());
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut content = String::from("# Test Agent Memory\n\n## Coverage\n\n");
        for func in &self.covered {
            content.push_str(&format!("- `{}` → `test_{}`\n", func, func));
        }
        content.push_str("\n## Style\n\n");
        content.push_str(&self.style);
        fs::write(&self.file_path, &content)
            .with_context(|| format!("Failed to write {}", self.file_path))?;
        Ok(())
    }
}
