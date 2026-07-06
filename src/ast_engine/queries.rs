use tree_sitter::Query;

/// Captures `import foo` and `import foo.bar` statements.
pub fn import_query() -> Query {
    Query::new(
        &tree_sitter_python::language(),
        "(import_statement name: (dotted_name) @import_name) @import",
    )
    .expect("invalid import query")
}

/// Captures `from foo import bar, baz` statements.
pub fn import_from_query() -> Query {
    Query::new(
        &tree_sitter_python::language(),
        "(import_from_statement
           module_name: (dotted_name) @from_module
           name: (dotted_name) @import_name) @import_from",
    )
    .expect("invalid import_from query")
}

/// Captures class definitions including name, bases, and body.
pub fn class_query() -> Query {
    Query::new(
        &tree_sitter_python::language(),
        "(class_definition
           name: (identifier) @class_name
           superclasses: (argument_list) @class_bases
           body: (block) @class_body) @class_def",
    )
    .expect("invalid class query")
}

/// Captures decorated functions (FastAPI routes, etc).
pub fn decorated_query() -> Query {
    Query::new(
        &tree_sitter_python::language(),
        "(decorated_definition
           (decorator) @decorator
           definition: (function_definition
             name: (identifier) @func_name
             body: (block) @func_body)) @decorated_func",
    )
    .expect("invalid decorated query")
}

/// Captures top-level bare functions (no decorators, not class methods).
pub fn function_query() -> Query {
    Query::new(
        &tree_sitter_python::language(),
        "(module (function_definition
           name: (identifier) @func_name
           body: (block) @func_body)) @function_def",
    )
    .expect("invalid function query")
}
