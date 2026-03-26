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

fn setup_test_project(src_dir: &str) -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let src_path = std::path::Path::new("tests/src").join(src_dir);
    copy_dir_all(&src_path, temp_dir.path()).expect("failed to copy test project");
    temp_dir
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
    assert!(stderr.contains("No supported build system found"));
}

#[test]
fn test_rust_detection() {
    let temp_dir = setup_test_project("rust");
    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Rust"));
}

#[test]
fn test_make_detection() {
    let temp_dir = setup_test_project("make");
    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Makefile"));
}

#[test]
fn test_real_rust_project() {
    let temp_dir = setup_test_project("rust");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Rust"));
    assert!(stdout.contains("Hello world!"));
}

#[test]
fn test_real_make_project() {
    let temp_dir = setup_test_project("make");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Makefile"));
    assert!(stdout.contains("Hello world!"));
}

#[test]
fn test_real_cmake_project() {
    let temp_dir = setup_test_project("cmake");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: CMake"));
    assert!(stdout.contains("Hello world!"));
}

#[test]
fn test_real_go_project() {
    let temp_dir = setup_test_project("go");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Go"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_zig_project() {
    let temp_dir = setup_test_project("zig");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stdout.contains("using: Zig"));
        assert!(stdout.contains("Hello world!") || stderr.contains("Hello world!"));
    }
}

#[test]
fn test_real_node_project() {
    let temp_dir = setup_test_project("node");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: JavaScript/TypeScript"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_dotnet_project() {
    let temp_dir = setup_test_project("dotnet");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: .NET"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_maven_project() {
    let temp_dir = setup_test_project("maven");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Maven"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_gradle_project() {
    let temp_dir = setup_test_project("gradle");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Gradle"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_docker_detection() {
    let temp_dir = setup_test_project("docker");
    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Docker"));
}

#[test]
fn test_real_bun_project() {
    let temp_dir = setup_test_project("bun");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: JavaScript/TypeScript"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_deno_project() {
    let temp_dir = setup_test_project("deno");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: JavaScript/TypeScript"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_pnpm_detection() {
    let temp_dir = setup_test_project("node");
    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: JavaScript/TypeScript"));
    }
}

#[test]
fn test_real_swift_project() {
    let temp_dir = setup_test_project("swift");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Swift"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_uv_project() {
    let temp_dir = setup_test_project("uv");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: uv"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_python_project() {
    let temp_dir = setup_test_project("python");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Python"));
        assert!(stdout.contains("Hello world!"));
    }
}

#[test]
fn test_real_flutter_project() {
    let temp_dir = setup_test_project("flutter");
    let output = Command::new(get_bin())
        .arg("-x")
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Flutter"));
        assert!(stdout.contains("Hello world!"));
    }
}
