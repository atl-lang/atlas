use super::execute_with_io;
use super::*;
use std::fs;
use tempfile::TempDir;

fn unwrap_atlas_result_ok(val: atlas_runtime::Value) -> atlas_runtime::Value {
    match val {
        atlas_runtime::Value::Result(inner) => match inner {
            Ok(v) => *v,
            Err(e) => panic!("Expected Atlas Ok, got Atlas Err: {:?}", e),
        },
        other => other,
    }
}

// From vm_stdlib_io_tests.rs
// Standard library file I/O tests (VM/Bytecode)
//
// Tests file and directory operations via bytecode execution for VM parity.

// ============================================================================
// VM parity tests - all use pattern: let result = func(); result;
// ============================================================================

#[test]
fn vm_test_read_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, VM!").unwrap();

    let code = format!(r#"let x = file.read("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let inner = unwrap_atlas_result_ok(result.unwrap());
    assert!(matches!(inner, atlas_runtime::Value::String(_)));
}

#[test]
fn vm_test_write_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("output.txt");

    let code = format!(
        r#"file.write("{}", "VM content");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "VM content");
}

#[test]
fn vm_test_append_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append.txt");
    fs::write(&test_file, "line1\n").unwrap();

    let code = format!(
        r#"file.append("{}", "line2\n");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn vm_test_file_exists_true() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("exists.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(
        r#"let result = file.exists("{}"); result;"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(matches!(value, atlas_runtime::Value::Bool(true)));
}

#[test]
fn vm_test_file_exists_false() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(
        r#"let result = file.exists("{}"); result;"#,
        path_for_atlas(&nonexistent)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(false)));
}

#[test]
fn vm_test_read_dir_basic() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

    let code = format!(
        r#"let result = file.readDir("{}"); result;"#,
        path_for_atlas(temp_dir.path())
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Array(_)));
}

#[test]
fn vm_test_create_dir_basic() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("newdir");

    let code = format!(r#"file.createDir("{}");"#, path_for_atlas(&new_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn vm_test_create_dir_nested() {
    let temp_dir = TempDir::new().unwrap();
    let nested_dir = temp_dir.path().join("a/b/c");

    let code = format!(r#"file.createDir("{}");"#, path_for_atlas(&nested_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(nested_dir.exists());
}

#[test]
fn vm_test_remove_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("remove.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"file.remove("{}");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(!test_file.exists());
}

#[test]
fn vm_test_remove_dir_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("rmdir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"file.removeDir("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(!test_dir.exists());
}

#[test]
fn vm_test_file_info_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info.txt");
    fs::write(&test_file, "test content").unwrap();

    let code = format!(
        r#"let result = file.info("{}"); result;"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn vm_test_path_join_basic() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let result = path.join("a", "b", "c"); result;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
}

// ============================================================================
// Additional VM parity tests to match interpreter coverage
// ============================================================================

#[test]
fn vm_test_read_file_utf8() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("utf8.txt");
    fs::write(&test_file, "Hello 你好 🎉").unwrap();

    let code = format!(r#"let x = file.read("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_read_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"file.read("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        atlas_runtime::Value::Result(inner) => {
            assert!(inner.is_err(), "Expected Atlas Err");
        }
        other => panic!("Expected Atlas Result(Err), got {:?}", other),
    }
}

#[test]
fn vm_test_read_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("secret.txt");
    fs::write(&test_file, "secret").unwrap();

    // Execute without granting permissions
    let mut lexer =
        atlas_runtime::Lexer::new(format!(r#"file.read("{}");"#, path_for_atlas(&test_file)));
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = atlas_runtime::Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = atlas_runtime::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = atlas_runtime::Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new(); // No permissions
    let mut vm = atlas_runtime::VM::new(bytecode);
    let result = vm.run(&security);

    // file.read returns Atlas Result(Err) for permission denial
    assert!(result.is_ok());
    let val = result.unwrap().unwrap_or(atlas_runtime::Value::Null);
    assert!(matches!(val, atlas_runtime::Value::Result(Err(_))));
}

#[test]
fn vm_test_write_file_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("overwrite.txt");
    fs::write(&test_file, "original").unwrap();

    let code = format!(
        r#"file.write("{}", "new content");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "new content");
}

#[test]
fn vm_test_write_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("output.txt");

    // Execute without granting permissions
    let mut lexer = atlas_runtime::Lexer::new(format!(
        r#"file.write("{}", "content");"#,
        path_for_atlas(&test_file)
    ));
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = atlas_runtime::Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = atlas_runtime::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = atlas_runtime::Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new(); // No permissions
    let mut vm = atlas_runtime::VM::new(bytecode);
    let result = vm.run(&security);

    // file.write returns Atlas Result(Err) for permission denial
    assert!(result.is_ok());
    let val = result.unwrap().unwrap_or(atlas_runtime::Value::Null);
    assert!(matches!(val, atlas_runtime::Value::Result(Err(_))));
}

#[test]
fn vm_test_append_file_create_if_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("new.txt");

    let code = format!(
        r#"file.append("{}", "content");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "content");
}

#[test]
fn vm_test_read_dir_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent_dir");

    let code = format!(r#"file.readDir("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"file.remove("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        atlas_runtime::Value::Result(inner) => {
            assert!(inner.is_err(), "Expected Atlas Err");
        }
        other => panic!("Expected Atlas Result(Err), got {:?}", other),
    }
}

#[test]
fn vm_test_remove_dir_not_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("notempty");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file.txt"), "").unwrap();

    let code = format!(r#"file.removeDir("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        atlas_runtime::Value::Result(inner) => {
            assert!(inner.is_err(), "Expected Atlas Err");
        }
        other => panic!("Expected Atlas Result(Err), got {:?}", other),
    }
}

#[test]
fn vm_test_file_info_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("infodir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(
        r#"let result = file.info("{}"); result;"#,
        path_for_atlas(&test_dir)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_path_join_single() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let result = path.join("single"); result;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_path_join_no_args() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"path.join();"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional read_file tests
// ============================================================================

#[test]
fn vm_test_read_file_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"let x = file.read("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let inner = unwrap_atlas_result_ok(result.unwrap());
    if let atlas_runtime::Value::String(s) = inner {
        assert_eq!(s.as_str(), "");
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_read_file_invalid_utf8() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("binary.bin");
    fs::write(&test_file, [0xFF, 0xFE, 0xFD]).unwrap();

    let code = format!(r#"file.read("{}");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        atlas_runtime::Value::Result(inner) => {
            assert!(inner.is_err(), "Expected Atlas Err");
        }
        other => panic!("Expected Atlas Result(Err), got {:?}", other),
    }
}

#[test]
fn vm_test_read_file_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multiline.txt");
    let content = "line1\nline2\nline3\n";
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"let x = file.read("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let inner = unwrap_atlas_result_ok(result.unwrap());
    if let atlas_runtime::Value::String(s) = inner {
        assert_eq!(s.as_str(), content);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_read_file_large() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("large.txt");
    let content = "x".repeat(10000);
    fs::write(&test_file, &content).unwrap();

    let code = format!(r#"let x = file.read("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let inner = unwrap_atlas_result_ok(result.unwrap());
    if let atlas_runtime::Value::String(s) = inner {
        assert_eq!(s.len(), 10000);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_read_file_with_bom() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("bom.txt");
    let mut content = vec![0xEF, 0xBB, 0xBF];
    content.extend_from_slice(b"Hello");
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"let x = file.read("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

// ============================================================================
// VM - Additional write_file tests (continued in io_part2.rs)
// ============================================================================
