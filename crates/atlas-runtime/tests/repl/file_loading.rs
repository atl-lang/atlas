use atlas_runtime::repl::ReplCore;

#[test]
fn repl_load_file_nonexistent_error() {
    let mut repl = ReplCore::new();
    let result = repl.load_file(std::path::Path::new("/nonexistent/file.atlas"));
    assert!(result.is_err());
}
