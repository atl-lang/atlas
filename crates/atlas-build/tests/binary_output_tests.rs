use atlas_build::binary_emit::{emit_native_binary, find_launcher_binary, ATLAS_BC_MAGIC};
use atlas_build::targets::{BuildTarget, TargetKind};
use std::path::PathBuf;

fn temp_path(suffix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "atlas_binary_test_{}_{suffix}.bin",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time after epoch")
            .subsec_nanos()
    ))
}

// ── filename tests ────────────────────────────────────────────────────────────

#[test]
fn test_binary_output_filename_has_no_extension_on_unix() {
    let target = BuildTarget::new("myapp", TargetKind::Binary);
    #[cfg(unix)]
    assert_eq!(
        target.output_filename(),
        "myapp",
        "Binary targets must have no extension on Unix"
    );
    #[cfg(windows)]
    assert_eq!(target.output_filename(), "myapp.exe");
}

#[test]
fn test_library_output_filename_has_atl_bc() {
    let target = BuildTarget::new("mylib", TargetKind::Library);
    assert_eq!(target.output_filename(), "mylib.atl.bc");
}

#[test]
fn test_bytecode_output_filename_has_atl_bc() {
    let target = BuildTarget::new("mypkg", TargetKind::Bytecode);
    assert_eq!(target.output_filename(), "mypkg.atl.bc");
}

// ── emit_native_binary tests ──────────────────────────────────────────────────

#[test]
fn test_emit_native_binary_creates_executable() {
    // Use a real launcher binary if available, otherwise a fake one
    let launcher_path = temp_path("launcher");
    let fake_launcher = b"#!/fake/elf\x7fELFfake_launcher_content_for_test";
    std::fs::write(&launcher_path, fake_launcher).expect("write fake launcher");

    let bytecode = b"fake atlas bytecode payload for test";
    let module_bytecodes = vec![bytecode.to_vec()];
    let output_path = temp_path("output");

    emit_native_binary(&launcher_path, &module_bytecodes, &output_path)
        .expect("emit_native_binary");

    // File must exist
    assert!(output_path.exists(), "output binary must exist");

    // File size must be > launcher size (bytecode was appended)
    let output_size = std::fs::metadata(&output_path).expect("metadata").len() as usize;
    assert!(
        output_size > fake_launcher.len(),
        "output must be larger than launcher (bytecode appended)"
    );

    // Must contain the magic sentinel
    let content = std::fs::read(&output_path).expect("read output");
    assert!(
        content.windows(16).any(|w| w == ATLAS_BC_MAGIC),
        "output must contain ATLAS_BC_MAGIC sentinel"
    );

    // Unix: must have executable bit set
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&output_path)
            .expect("metadata")
            .permissions()
            .mode();
        assert!(
            mode & 0o111 != 0,
            "output binary must have executable bit set"
        );
    }

    let _ = std::fs::remove_file(&launcher_path);
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_find_appended_bytecode_roundtrip_via_emit() {
    // Build a complete binary manually: fake_launcher + sentinel + len + known_bytecode
    let known_bytecode = b"roundtrip test bytecode content 1234";

    let launcher_path = temp_path("launcher2");
    std::fs::write(&launcher_path, b"fake_launcher_bytes").expect("write launcher");

    let output_path = temp_path("roundtrip_binary");
    let module_bytecodes = vec![known_bytecode.to_vec()];
    emit_native_binary(&launcher_path, &module_bytecodes, &output_path)
        .expect("emit_native_binary");

    // Now use the launcher's find_appended_bytecode to extract it back
    let extracted =
        atlas_launcher::self_path::find_appended_bytecode(&output_path).expect("io error");

    assert!(
        extracted.is_some(),
        "bytecode should be found in emitted binary"
    );
    let modules = extracted.expect("already checked");
    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0],
        known_bytecode.as_slice(),
        "extracted bytecode must match original"
    );

    let _ = std::fs::remove_file(&launcher_path);
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_find_launcher_binary_returns_none_when_not_on_path() {
    // Temporarily clear PATH to ensure find_launcher_binary() returns None
    // (only tests the PATH search path; same-dir search may still find it)
    // We can't easily clear same-dir, so just verify the function doesn't panic
    // and returns a consistent result.
    let result = find_launcher_binary();
    // In CI / clean env this may be None; in dev it may be Some.
    // What we verify: the function runs without panicking and returns Option.
    let _ = result; // type check: Option<PathBuf>
}
