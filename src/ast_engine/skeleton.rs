use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub name: Option<String>,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDef {
    pub name: String,
    pub is_decorated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSkeleton {
    pub path: String,
    pub imports: Vec<Import>,
    pub functions: Vec<FnDef>,
    pub classes: Vec<ClassDef>,
    pub source_text: String,
    pub token_count: u32,
}
