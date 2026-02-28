use super::execute_with_io;
use super::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// VM - Additional writeFile tests
// ============================================================================

#[test]
fn vm_test_write_file_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty_write.txt");

    let code = format!(r#"writeFile("{}", "");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "");
}

#[test]
fn vm_test_write_file_unicode() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("unicode.txt");
    let content = "Hello 世界 🌍";

    let code = format!(
        r#"writeFile("{}", "{}");"#,
        path_for_atlas(&test_file),
        content
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, content);
}

#[test]
fn vm_test_write_file_newlines() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("newlines.txt");

    let code = format!(
        r#"writeFile("{}", "line1\nline2\n");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn vm_test_write_file_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("new_file.txt");
    assert!(!test_file.exists());

    let code = format!(r#"writeFile("{}", "content");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(test_file.exists());
}

// ============================================================================
// VM - Additional appendFile tests
// ============================================================================

#[test]
fn vm_test_append_file_multiple() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multi_append.txt");
    fs::write(&test_file, "start\n").unwrap();

    let code = format!(
        r#"appendFile("{}", "line1\n"); appendFile("{}", "line2\n");"#,
        path_for_atlas(&test_file),
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "start\nline1\nline2\n");
}

#[test]
fn vm_test_append_file_empty_content() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append_empty.txt");
    fs::write(&test_file, "base").unwrap();

    let code = format!(r#"appendFile("{}", "");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "base");
}

#[test]
fn vm_test_append_file_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append_denied.txt");

    let code = format!(
        r#"appendFile("{}", "content");"#,
        path_for_atlas(&test_file)
    );

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional fileExists tests
// ============================================================================

#[test]
fn vm_test_file_exists_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("exists_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(
        r#"let result = fileExists("{}"); result;"#,
        path_for_atlas(&test_dir)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

#[test]
fn vm_test_file_exists_no_permission_check() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("exists_test.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(
        r#"let x = fileExists("{}"); x;"#,
        path_for_atlas(&test_file)
    );

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(atlas_runtime::Value::Bool(true))
    ));
}

// ============================================================================
// VM - Additional readDir tests
// ============================================================================

#[test]
fn vm_test_read_dir_empty() {
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();

    let code = format!(r#"let x = readDir("{}"); x;"#, path_for_atlas(&empty_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn vm_test_read_dir_mixed_contents() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file.txt"), "").unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();

    let code = format!(
        r#"let x = readDir("{}"); x;"#,
        path_for_atlas(temp_dir.path())
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.len(), 2);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn vm_test_read_dir_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"readDir("{}");"#, path_for_atlas(&test_dir));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional createDir tests
// ============================================================================

#[test]
fn vm_test_create_dir_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("already_exists");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"createDir("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_create_dir_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("denied");

    let code = format!(r#"createDir("{}");"#, path_for_atlas(&new_dir));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional removeFile tests
// ============================================================================

#[test]
fn vm_test_remove_file_is_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("is_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_file_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("remove_denied.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_file));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional removeDir tests
// ============================================================================

#[test]
fn vm_test_remove_dir_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("not_found");

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_dir_is_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("is_file.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_dir_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("remove_denied");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_dir));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional fileInfo tests
// ============================================================================

#[test]
fn vm_test_file_info_size_check() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info_fields.txt");
    fs::write(&test_file, "12345").unwrap();

    let code = format!(r#"let x = fileInfo("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn vm_test_file_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("not_found.txt");

    let code = format!(r#"fileInfo("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_file_info_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info_denied.txt");
    fs::write(&test_file, "test").unwrap();

    let code = format!(r#"fileInfo("{}");"#, path_for_atlas(&test_file));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional pathJoin tests
// ============================================================================

#[test]
fn vm_test_path_join_many_parts() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let x = pathJoin("a", "b", "c", "d", "e"); x;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.contains("a"));
        assert!(path.contains("e"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_path_join_empty_parts() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let x = pathJoin("", "a", ""); x;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_path_join_absolute_path() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let x = pathJoin("/absolute", "path"); x;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.starts_with("/") || path.starts_with("\\"));
    } else {
        panic!("Expected string");
    }
}

// ============================================================================
