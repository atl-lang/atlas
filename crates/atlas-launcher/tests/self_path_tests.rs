use atlas_launcher::self_path::{find_appended_bytecode, ATLAS_BC_MAGIC};

fn make_test_binary(bytecode: &[u8]) -> Vec<u8> {
    let fake_launcher = b"ELF_FAKE_LAUNCHER_BYTES_HERE";
    let mut buf = Vec::new();
    buf.extend_from_slice(fake_launcher);
    buf.extend_from_slice(&ATLAS_BC_MAGIC);
    buf.extend_from_slice(&(bytecode.len() as u64).to_le_bytes());
    buf.extend_from_slice(bytecode);
    buf
}

fn tempfile_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "atlas_launcher_test_{}.bin",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time is after epoch")
            .subsec_nanos()
    ))
}

#[test]
fn test_find_appended_bytecode_roundtrip() {
    let bytecode = b"hello atlas bytecode payload";
    let binary = make_test_binary(bytecode);

    let tmp = tempfile_path();
    std::fs::write(&tmp, &binary).expect("write temp file");

    let result = find_appended_bytecode(&tmp).expect("io error");
    assert!(result.is_some(), "sentinel should be found");
    assert_eq!(result.expect("already checked Some"), bytecode.as_slice());

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_find_appended_bytecode_no_sentinel() {
    let binary = b"plain binary with no atlas magic bytes anywhere";

    let tmp = tempfile_path();
    std::fs::write(&tmp, binary).expect("write temp file");

    let result = find_appended_bytecode(&tmp).expect("io error");
    assert!(
        result.is_none(),
        "should return None when no sentinel present"
    );

    let _ = std::fs::remove_file(&tmp);
}
