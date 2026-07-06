use crate::ast_engine::skeleton::{FileSkeleton, FnDef};

pub struct CoverageDelta {
    pub uncovered: Vec<FnDef>,
}

/// For each function in `source`, check if any test file has
/// a matching `test_<name>` function. If not, it's uncovered.
/// (Agent memory is no longer used here — coverage is determined
/// by actual test file inspection.)
pub fn find_deltas(
    source: &FileSkeleton,
    tests: &[FileSkeleton],
    _known_covered: &[String],
) -> CoverageDelta {
    let mut uncovered = Vec::new();

    for func in &source.functions {
        let test_name = format!("test_{}", func.name);

        let covered = tests.iter().any(|test| {
            test.functions.iter().any(|f| f.name == test_name)
        });

        if !covered {
            uncovered.push(FnDef {
                name: func.name.clone(),
                is_decorated: func.is_decorated,
                signature: func.signature.clone(),
            });
        }
    }

    CoverageDelta { uncovered }
}
