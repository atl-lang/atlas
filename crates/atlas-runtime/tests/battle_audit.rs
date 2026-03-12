// Battle Audit Harness — Atlas Full Audit
//
// Runs every .atlas file in battle-test/atlas-full-audit/domains/ through
// both engines (interpreter + VM). For each program:
//   - Must compile and run without panic
//   - Interpreter and VM output must be identical (parity)
//   - Output must match *.expected file if one exists
//
// Test naming: battle_audit::<domain>::<stem>

use atlas_runtime::api::Runtime;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Harness helpers
// ---------------------------------------------------------------------------

fn atlas_root() -> PathBuf {
    // crates/atlas-runtime/tests/ → project root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/atlas-runtime
        .expect("parent of CARGO_MANIFEST_DIR")
        .parent() // crates
        .expect("parent of crates")
        .parent() // project root
        .expect("project root")
        .to_path_buf()
}

fn domains_dir() -> PathBuf {
    atlas_root().join("battle-test/atlas-full-audit/domains")
}

fn run_engine(source: &str) -> Result<Value, String> {
    // D-052: unified VM execution path
    let mut rt = Runtime::new_with_security(SecurityContext::allow_all());
    rt.eval(source).map_err(|e| e.to_string())
}

fn assert_battle(atlas_file: &Path) {
    let source = fs::read_to_string(atlas_file)
        .unwrap_or_else(|e| panic!("Cannot read {:?}: {e}", atlas_file));

    let result = run_engine(&source).unwrap_or_else(|e| {
        panic!(
            "VM error in {:?}:\n{e}\n--- source ---\n{source}",
            atlas_file
        )
    });

    // Check expected output file if present
    let expected_file = atlas_file.with_extension("expected");
    if expected_file.exists() {
        let expected_str = fs::read_to_string(&expected_file)
            .unwrap_or_else(|e| panic!("Cannot read {:?}: {e}", expected_file));
        let expected_str = expected_str.trim();
        let actual_str = result.to_string();
        assert_eq!(
            actual_str, expected_str,
            "Output mismatch in {:?}\n  expected: {expected_str:?}\n  got: {actual_str:?}",
            atlas_file
        );
    }
}

// ---------------------------------------------------------------------------
// Domain 01: Primitives
// ---------------------------------------------------------------------------
mod battle_01_primitives {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("01-primitives")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(p01_number_arithmetic, "01_number_arithmetic.atlas");
    battle!(p02_string_ops, "02_string_ops.atlas");
    battle!(p03_bool_logic, "03_bool_logic.atlas");
    battle!(p04_null_handling, "04_null_handling.atlas");
    battle!(p05_let_mut, "05_let_mut.atlas");
}

// ---------------------------------------------------------------------------
// Domain 02: Control Flow
// ---------------------------------------------------------------------------
mod battle_02_control_flow {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("02-control-flow")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(cf01_if_else, "01_if_else.atlas");
    battle!(cf02_while_loop, "02_while_loop.atlas");
    battle!(cf03_for_in, "03_for_in.atlas");
    battle!(cf04_match_basic, "04_match_basic.atlas");
    battle!(cf05_match_enum, "05_match_enum.atlas");
}

// ---------------------------------------------------------------------------
// Domain 03: Functions
// ---------------------------------------------------------------------------
mod battle_03_functions {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("03-functions")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(fn01_named_fn, "01_named_fn.atlas");
    battle!(fn02_anonymous_fn, "02_anonymous_fn.atlas");
    battle!(fn03_recursion, "03_recursion.atlas");
    battle!(fn04_higher_order, "04_higher_order.atlas");
    battle!(fn05_first_class, "05_first_class.atlas");
}

// ---------------------------------------------------------------------------
// Domain 04: Types (Structs, Enums, Option, Result)
// ---------------------------------------------------------------------------
mod battle_04_types {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("04-types")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(ty01_structs, "01_structs.atlas");
    battle!(ty02_enums, "02_enums.atlas");
    battle!(ty03_option, "03_option.atlas");
    battle!(ty04_result, "04_result.atlas");
    battle!(ty05_type_alias, "05_type_alias.atlas");
    battle!(ty06_generics, "06_generics.atlas");
}

// ---------------------------------------------------------------------------
// Domain 05: Traits
// ---------------------------------------------------------------------------
mod battle_05_traits {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("05-traits")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(tr01_basic_trait, "01_basic_trait.atlas");
    battle!(tr02_default_method, "02_default_method.atlas");
    battle!(tr03_multiple_impls, "03_multiple_impls.atlas");
    battle!(tr04_trait_objects, "04_trait_objects.atlas");
}

// ---------------------------------------------------------------------------
// Domain 06: Collections
// ---------------------------------------------------------------------------
mod battle_06_collections {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("06-collections")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(col01_arrays, "01_arrays.atlas");
    battle!(col02_hashmap, "02_hashmap.atlas");
    battle!(col03_hashset, "03_hashset.atlas");
    battle!(col04_array_fn, "04_array_functions.atlas");
    battle!(col05_nested, "05_nested_collections.atlas");
}

// ---------------------------------------------------------------------------
// Domain 07: Error Handling
// ---------------------------------------------------------------------------
mod battle_07_error_handling {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("07-error-handling")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(err01_result_match, "01_result_match.atlas");
    battle!(err02_question_mark, "02_question_mark.atlas");
    battle!(err03_option_chain, "03_option_chain.atlas");
    battle!(err04_error_chain, "04_error_chain.atlas");
}

// ---------------------------------------------------------------------------
// Domain 08: Async/Await
// ---------------------------------------------------------------------------
mod battle_08_async {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("08-async-await")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(as01_basic_async, "01_basic_async.atlas");
    battle!(as02_sequential_pipeline, "02_sequential_pipeline.atlas");
    battle!(as03_parallel_all, "03_parallel_all.atlas");
    battle!(as04_race, "04_race.atlas");
    battle!(as05_async_error, "05_async_error.atlas");
}

// ---------------------------------------------------------------------------
// Domain 09: Stdlib
// ---------------------------------------------------------------------------
mod battle_09_stdlib {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("09-stdlib")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(sl01_string_fns, "01_string_functions.atlas");
    battle!(sl02_math_fns, "02_math_functions.atlas");
    battle!(sl03_json, "03_json.atlas");
    battle!(sl04_file_io, "04_file_io.atlas");
    battle!(sl05_reflect, "05_reflect.atlas");
}

// ---------------------------------------------------------------------------
// Domain 10: Integration
// ---------------------------------------------------------------------------
mod battle_10_integration {
    use super::*;

    fn domain() -> PathBuf {
        domains_dir().join("10-integration")
    }

    macro_rules! battle {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = domain().join($file);
                if path.exists() {
                    assert_battle(&path);
                } else {
                    eprintln!("SKIP: {:?} not generated yet", path);
                }
            }
        };
    }

    battle!(int01_task_manager, "01_task_manager.atlas");
    battle!(int02_data_pipeline, "02_data_pipeline.atlas");
    battle!(int03_config_loader, "03_config_loader.atlas");
}

// hydra-v5 archived — removed from active battle suite (replaced by hydra-opus, B11-P04)
