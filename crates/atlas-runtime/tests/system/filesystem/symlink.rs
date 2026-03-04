use super::*;

// ============================================================================
// Symlink Operations Tests (Unix only)
// ============================================================================

#[test]
#[cfg(unix)]
fn test_symlink_creates_symbolic_link() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    let target_str = file_path.to_str().unwrap();
    let link_str = link_path.to_str().unwrap();

    let result = fs::symlink(target_str, link_str, span());
    assert!(result.is_ok());
    assert!(link_path.exists());
    assert!(link_path.is_symlink());
}

#[test]
#[cfg(unix)]
fn test_readlink_returns_symlink_target() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    std::os::unix::fs::symlink(&file_path, &link_path).unwrap();

    let link_str = link_path.to_str().unwrap();
    let result = fs::readlink(link_str, span()).unwrap();
    let target = extract_string(&result);

    assert!(target.contains("target.txt"));
}

#[test]
#[cfg(unix)]
fn test_resolve_symlink_follows_chain() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link1_path = temp.path().join("link1.txt");
    let link2_path = temp.path().join("link2.txt");
    std_fs::write(&file_path, "content").unwrap();

    std::os::unix::fs::symlink(&file_path, &link1_path).unwrap();
    std::os::unix::fs::symlink(&link1_path, &link2_path).unwrap();

    let link2_str = link2_path.to_str().unwrap();
    let result = fs::resolve_symlink(link2_str, span()).unwrap();
    let resolved = extract_string(&result);

    assert!(Path::new(&resolved).exists());
    assert!(Path::new(&resolved).is_file());
}

#[test]
#[cfg(unix)]
fn test_symlink_relative_link() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    let result = fs::symlink("target.txt", link_path.to_str().unwrap(), span());
    assert!(result.is_ok());
    assert!(link_path.is_symlink());
}

#[test]
#[cfg(unix)]
fn test_symlink_absolute_link() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    let result = fs::symlink(
        file_path.to_str().unwrap(),
        link_path.to_str().unwrap(),
        span(),
    );
    assert!(result.is_ok());
    assert!(link_path.is_symlink());
}
