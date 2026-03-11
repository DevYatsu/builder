use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("builder")
}

#[test]
fn test_help() {
    let output = Command::new(get_bin())
        .arg("--help")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: builder"));
    assert!(stdout.contains("-t, --test"));
}

#[test]
fn test_no_build_system() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no build system found"));
}

#[test]
fn test_rust_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();

    // We expect it to try to run 'cargo build' and fail because Cargo.toml is empty,
    // but at least it should detect it.
    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("detected: Rust"));
}

#[test]
fn test_make_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("Makefile"), "all:\n\techo hello").unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("detected: Makefile"));
}
