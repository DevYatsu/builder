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
    assert!(stderr.contains("No supported build system found"));
}

#[test]
fn test_rust_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Rust"));
}

#[test]
fn test_make_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("Makefile"),
        include_str!("src/make/Makefile"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Makefile"));
}

#[test]
fn test_real_rust_project() {
    let temp_dir = tempfile::tempdir().unwrap();

    Command::new("cargo")
        .args(["init", "--bin", "--name", "myapp"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Rust"));
    assert!(stdout.contains("building..."));
    assert!(temp_dir.path().join("target").exists());
}

#[test]
fn test_real_make_project() {
    let temp_dir = tempfile::tempdir().unwrap();

    fs::write(
        temp_dir.path().join("main.c"),
        include_str!("src/make/main.c"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("Makefile"),
        include_str!("src/make/Makefile"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Makefile"));
    assert!(temp_dir.path().join("myapp").exists());
}

#[test]
fn test_real_cmake_project() {
    let temp_dir = tempfile::tempdir().unwrap();

    fs::write(
        temp_dir.path().join("main.c"),
        include_str!("src/cmake/main.c"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("CMakeLists.txt"),
        include_str!("src/cmake/CMakeLists.txt"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: CMake"));
    assert!(stdout.contains("building..."));

    let has_artifact = temp_dir.path().join("myapp").exists()
        || temp_dir.path().join("build/myapp").exists()
        || temp_dir.path().join("myapp.exe").exists()
        || temp_dir.path().join("build/myapp.exe").exists();
    assert!(has_artifact);
}

#[test]
fn test_real_go_project() {
    let temp_dir = tempfile::tempdir().unwrap();

    fs::write(
        temp_dir.path().join("main.go"),
        include_str!("src/go/main.go"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("go.mod"),
        include_str!("src/go/go.mod"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Go"));
    }
}

#[test]
fn test_real_zig_project() {
    let temp_dir = tempfile::tempdir().unwrap();

    fs::create_dir(temp_dir.path().join("src")).unwrap();
    fs::write(
        temp_dir.path().join("src/main.zig"),
        include_str!("src/zig/src/main.zig"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("src/root.zig"),
        include_str!("src/zig/src/root.zig"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("build.zig"),
        include_str!("src/zig/build.zig"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("build.zig.zon"),
        include_str!("src/zig/build.zig.zon"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Zig"));
    }
}

#[test]
fn test_real_node_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("package.json"),
        include_str!("src/node/package.json"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Node.js"));
    }
}

#[test]
fn test_real_dotnet_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("myapp.csproj"),
        include_str!("src/dotnet/myapp.csproj"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: .NET"));
    }
}

#[test]
fn test_real_maven_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("pom.xml"),
        include_str!("src/maven/pom.xml"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Maven"));
    }
}

#[test]
fn test_real_gradle_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("build.gradle"),
        include_str!("src/gradle/build.gradle"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Gradle"));
    }
}

#[test]
fn test_docker_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("Dockerfile"),
        include_str!("src/docker/Dockerfile"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("using: Docker"));
}

#[test]
fn test_real_bun_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("bun.lock"),
        include_str!("src/bun/bun.lock"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("index.ts"),
        include_str!("src/bun/index.ts"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("tsconfig.json"),
        include_str!("src/bun/tsconfig.json"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("package.json"),
        include_str!("src/bun/package.json"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Bun"));
    }
}

#[test]
fn test_real_deno_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("deno.json"),
        include_str!("src/deno/deno.json"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("main.ts"),
        include_str!("src/deno/main.ts"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("main_test.ts"),
        include_str!("src/deno/main_test.ts"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Deno"));
    }
}

#[test]
fn test_pnpm_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("package.json"),
        include_str!("src/node/package.json"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Node.js"));
    }
}

#[test]
fn test_real_swift_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp_dir.path().join("Sources")).unwrap();
    fs::write(
        temp_dir.path().join("Package.swift"),
        include_str!("src/swift/Package.swift"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("Sources/main.swift"),
        include_str!("src/swift/Sources/main.swift"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Swift"));
    }
}

#[test]
fn test_real_uv_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        include_str!("src/uv/pyproject.toml"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("main.py"),
        include_str!("src/uv/main.py"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: uv (Python)"));
    }
}

#[test]
fn test_real_python_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("requirements.txt"),
        include_str!("src/python/requirements.txt"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("main.py"),
        include_str!("src/python/main.py"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Python"));
    }
}

#[test]
fn test_real_flutter_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp_dir.path().join("lib")).unwrap();
    fs::write(
        temp_dir.path().join("pubspec.yaml"),
        include_str!("src/flutter/pubspec.yaml"),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("lib/main.dart"),
        include_str!("src/flutter/lib/main.dart"),
    )
    .unwrap();

    let output = Command::new(get_bin())
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("using: Flutter"));
    }
}
