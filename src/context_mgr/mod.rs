use crate::ast_engine::skeleton::{FileSkeleton, FnDef};

pub struct CoverageDelta {
    pub uncovered: Vec<FnDef>,
}

/// For each function in `source`, check if any test file has a
/// matching `test_<name>` function. If not, it's uncovered.
pub fn find_deltas(source: &FileSkeleton, tests: &[FileSkeleton]) -> CoverageDelta {
    let mut uncovered = Vec::new();

    for func in &source.functions {
        let test_name = format!("test_{}", func.name);

        // `.any()` returns true if **any** test file has a function
        // with the matching name
        let covered = tests.iter().any(|test| {
            test.functions.iter().any(|f| f.name == test_name)
        });

        if !covered {
            uncovered.push(FnDef {
                name: func.name.clone(),
                is_decorated: func.is_decorated,
            });
        }
    }

    CoverageDelta { uncovered }
}
