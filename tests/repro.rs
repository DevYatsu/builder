use std::fs;
use std::process::Command;
use std::path::PathBuf;

fn get_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("builder")
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[test]
fn test_cmake_bin_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let src_path = std::path::Path::new("tests/src/cmake_bin");
    copy_dir_all(&src_path, temp_dir.path()).expect("failed to copy test project");
    
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    
    assert!(output.status.success(), "Command failed with status: {:?}\nSTDERR: {}", output.status, stderr);
    assert!(stdout.contains("using: CMake"));
    assert!(stdout.contains("Hello world!"));
}

#[test]
fn test_make_norun_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let src_path = std::path::Path::new("tests/src/make_norun");
    copy_dir_all(&src_path, temp_dir.path()).expect("failed to copy test project");
    
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    
    assert!(output.status.success(), "Command failed with status: {:?}\nSTDERR: {}", output.status, stderr);
    assert!(stdout.contains("using: Makefile"));
    assert!(stdout.contains("Hello world!"));
}
