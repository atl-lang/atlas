use atlas_launcher::self_path::{find_appended_bytecode, ATLAS_BC_MAGIC};

// Multi-module archive format:
// [launcher][module_count u32 LE][bc_len u32 LE][bc_bytes]...[ATLAS_BC_MAGIC 16][payload_len u64 LE 8]
fn make_test_binary(modules: &[&[u8]]) -> Vec<u8> {
    let fake_launcher = b"ELF_FAKE_LAUNCHER_BYTES_HERE";
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(&(modules.len() as u32).to_le_bytes());
    for bc in modules {
        payload.extend_from_slice(&(bc.len() as u32).to_le_bytes());
        payload.extend_from_slice(bc);
    }
    let payload_len = payload.len() as u64;

    let mut buf = Vec::new();
    buf.extend_from_slice(fake_launcher);
    buf.extend_from_slice(&payload);
    buf.extend_from_slice(&ATLAS_BC_MAGIC);
    buf.extend_from_slice(&payload_len.to_le_bytes());
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
    let mod1 = b"module_one_bytecode";
    let mod2 = b"module_two_bytecode";
    let binary = make_test_binary(&[mod1, mod2]);

    let tmp = tempfile_path();
    std::fs::write(&tmp, &binary).expect("write temp file");

    let result = find_appended_bytecode(&tmp).expect("io error");
    assert!(result.is_some(), "sentinel should be found");
    let modules = result.expect("already checked Some");
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0], mod1.as_slice());
    assert_eq!(modules[1], mod2.as_slice());

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
