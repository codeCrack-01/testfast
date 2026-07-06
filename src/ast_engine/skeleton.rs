// The "stripped" representation of a Python file.
// We'll define structs like FileSkeleton, FnDef, ModelDef here.

pub struct Import {
    pub name: Option<String>,
    pub module: Option<String>,
}

pub struct FnDef {
    pub name: String,
    pub is_decorated: bool,
}

pub struct ClassDef {
    pub name: String,
}

pub struct FileSkeleton {
    pub path: String,
    pub imports: Vec<Import>,
    pub functions: Vec<FnDef>,
    pub classes: Vec<ClassDef>,
    pub source_text: String,
    pub token_count: u32,
}
