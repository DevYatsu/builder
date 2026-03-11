use crate::error::{BuildError, Result};
use std::path::{Path, PathBuf};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildOptions {
    pub run: bool,
    pub release: bool,
    pub test: bool,
}

pub trait BuildSystem {
    fn detect(&self, sh: &Shell) -> bool;
    fn name(&self) -> &'static str;
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()>;
}

pub fn get_systems() -> Vec<Box<dyn BuildSystem>> {
    vec![
        Box::new(RustBuild),
        Box::new(MakeBuild),
        Box::new(JustBuild),
        Box::new(CMakeBuild),
        Box::new(NodeBuild),
        Box::new(BunBuild),
        Box::new(DenoBuild),
        Box::new(GoBuild),
        Box::new(UvBuild),
        Box::new(PythonBuild),
        Box::new(SwiftBuild),
        Box::new(FlutterBuild),
        Box::new(DockerBuild),
        Box::new(MavenBuild),
        Box::new(GradleBuild),
        Box::new(ZigBuild),
        Box::new(DotnetBuild),
    ]
}

#[derive(Debug, Clone, Copy)]
struct RustBuild;
impl BuildSystem for RustBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Cargo.toml")
    }
    fn name(&self) -> &'static str {
        "Rust"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        let rel = if options.release {
            vec!["--release"]
        } else {
            vec![]
        };
        cmd!(sh, "cargo {verb} {rel...}")
            .run()
            .map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct MakeBuild;
impl BuildSystem for MakeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Makefile")
    }
    fn name(&self) -> &'static str {
        "Makefile"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        cmd!(sh, "make").run()?;
        if options.test {
            cmd!(sh, "make test").run()?;
        }
        if options.run && cmd!(sh, "make run").run().is_err() {
            execute_recently_modified_binary(sh, ".")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct JustBuild;
impl BuildSystem for JustBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("justfile") || sh.path_exists("Justfile")
    }
    fn name(&self) -> &'static str {
        "Just"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let recipe = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        cmd!(sh, "just {recipe}").run().map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct CMakeBuild;
impl BuildSystem for CMakeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("CMakeLists.txt")
    }
    fn name(&self) -> &'static str {
        "CMake"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let build_dir = if sh.path_exists("build/CMakeCache.txt") {
            "build"
        } else if sh.path_exists("CMakeCache.txt") {
            "."
        } else {
            let mut args = vec!["-B", "build", "-S", "."];
            if cmd!(sh, "ninja --version").read().is_ok() {
                args.extend(["-G", "Ninja"]);
            }
            if options.release {
                args.push("-DCMAKE_BUILD_TYPE=Release");
            }
            cmd!(sh, "cmake {args...}").run()?;
            "build"
        };
        let config = if options.release {
            vec!["--config", "Release"]
        } else {
            vec![]
        };
        cmd!(sh, "cmake --build {build_dir} {config...}").run()?;
        if options.test {
            cmd!(sh, "ctest --test-dir {build_dir}").run()?;
        }
        if options.run {
            execute_recently_modified_binary(sh, build_dir)?;
        }
        Ok(())
    }
}

fn execute_recently_modified_binary(sh: &Shell, search_dir: &str) -> Result<()> {
    let mut most_recent = None;
    let mut max_time = std::time::UNIX_EPOCH;
    let skip_dirs = [
        ".git",
        "node_modules",
        "target",
        "build",
        "dist",
        ".venv",
        "zig-cache",
        "zig-out",
        "CMakeFiles",
        ".swiftpm",
        ".dart_tool",
        "__pycache__",
        "obj",
        "bin",
    ];
    let mut dirs = if search_dir != "." {
        vec![PathBuf::from(search_dir)]
    } else {
        vec![PathBuf::from(".")]
    };
    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = sh.read_dir(&dir) {
            for path in entries {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || skip_dirs.contains(&name.as_ref()) {
                    continue;
                }
                let full_path = sh.current_dir().join(&path);
                if full_path.is_dir() {
                    dirs.push(path);
                } else if is_executable(&full_path)
                    && let Ok(meta) = std::fs::metadata(&full_path)
                    && let Ok(modified) = meta.modified()
                    && modified > max_time
                {
                    max_time = modified;
                    most_recent = Some(path);
                }
            }
        }
    }
    if let Some(exe) = most_recent {
        log::info!("executing: {}", exe.display());
        cmd!(sh, "{exe}").run().map_err(BuildError::from)
    } else {
        log::warn!("no executable found");
        Ok(())
    }
}

fn is_executable(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        #[cfg(windows)]
        {
            if ext_str != "exe" && ext_str != "bat" && ext_str != "cmd" {
                return false;
            }
        }
        #[cfg(not(windows))]
        {
            let _ = ext_str;
            return false;
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.is_file() && meta.permissions().mode() & 0o111 != 0;
        }
    }
    #[cfg(windows)]
    {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        return name.ends_with(".exe") || name.ends_with(".bat") || name.ends_with(".cmd");
    }
    false
}

#[derive(Debug, Clone, Copy)]
struct NodeBuild;
impl BuildSystem for NodeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("package.json")
    }
    fn name(&self) -> &'static str {
        "Node.js"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let pm = if sh.path_exists("pnpm-lock.yaml") {
            "pnpm"
        } else if sh.path_exists("yarn.lock") {
            "yarn"
        } else {
            "npm"
        };
        let script = if options.test {
            "test"
        } else if options.run {
            if sh.path_exists("main.js") || sh.path_exists("index.js") {
                return cmd!(sh, "node .").run().map_err(BuildError::from);
            }
            "start"
        } else {
            "build"
        };
        cmd!(sh, "{pm} run {script}")
            .run()
            .map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct BunBuild;
impl BuildSystem for BunBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("bun.lockb") || sh.path_exists("bunfig.toml")
    }
    fn name(&self) -> &'static str {
        "Bun"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.extend(["run", "."]);
        } else {
            args.extend(["run", "build"]);
        }
        cmd!(sh, "bun {args...}").run().map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct DenoBuild;
impl BuildSystem for DenoBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("deno.json") || sh.path_exists("deno.jsonc")
    }
    fn name(&self) -> &'static str {
        "Deno"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.extend(["run", "-A"]);
            if sh.path_exists("main.ts") {
                args.push("main.ts");
            } else if sh.path_exists("index.ts") {
                args.push("index.ts");
            } else {
                args.push(".");
            }
        } else {
            args.extend(["task", "build"]);
        }
        cmd!(sh, "deno {args...}").run().map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct GoBuild;
impl BuildSystem for GoBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("go.mod")
    }
    fn name(&self) -> &'static str {
        "Go"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.extend(["test", "./..."]);
        } else if options.run {
            args.extend(["run", "."]);
        } else {
            args.push("build");
        }
        cmd!(sh, "go {args...}").run().map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct UvBuild;
impl BuildSystem for UvBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pyproject.toml") && cmd!(sh, "uv --version").run().is_ok()
    }
    fn name(&self) -> &'static str {
        "uv (Python)"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.extend(["run", "pytest"]);
        } else if options.run {
            let entry = if sh.path_exists("main.py") {
                "main.py"
            } else {
                "."
            };
            args.extend(["run", entry]);
        } else {
            args.push("sync");
        }
        cmd!(sh, "uv {args...}").run().map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct PythonBuild;
impl BuildSystem for PythonBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("requirements.txt") || sh.path_exists("setup.py")
    }
    fn name(&self) -> &'static str {
        "Python"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let py = if sh.path_exists(".venv") {
            #[cfg(unix)]
            {
                ".venv/bin/python"
            }
            #[cfg(windows)]
            {
                ".venv\\Scripts\\python.exe"
            }
        } else {
            "python3"
        };
        if options.test {
            cmd!(sh, "{py} -m pytest").run().map_err(BuildError::from)
        } else if options.run {
            let entry = if sh.path_exists("main.py") {
                "main.py"
            } else {
                "."
            };
            cmd!(sh, "{py} {entry}").run().map_err(BuildError::from)
        } else {
            cmd!(sh, "{py} -m pip install -r requirements.txt")
                .run()
                .map_err(BuildError::from)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SwiftBuild;
impl BuildSystem for SwiftBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Package.swift")
    }
    fn name(&self) -> &'static str {
        "Swift"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        let config = if options.release {
            vec!["-c", "release"]
        } else {
            vec![]
        };
        cmd!(sh, "swift {verb} {config...}")
            .run()
            .map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct FlutterBuild;
impl BuildSystem for FlutterBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pubspec.yaml") && sh.path_exists("lib")
    }
    fn name(&self) -> &'static str {
        "Flutter"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        let rel = if options.release && verb != "test" {
            vec!["--release"]
        } else {
            vec![]
        };
        cmd!(sh, "flutter {verb} {rel...}")
            .run()
            .map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct DockerBuild;
impl BuildSystem for DockerBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Dockerfile")
    }
    fn name(&self) -> &'static str {
        "Docker"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        cmd!(sh, "docker build . -t app_image").run()?;
        if options.run {
            cmd!(sh, "docker run -it --rm app_image").run()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct MavenBuild;
impl BuildSystem for MavenBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pom.xml")
    }
    fn name(&self) -> &'static str {
        "Maven"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.push("spring-boot:run");
        } else {
            args.push("package");
        }
        cmd!(sh, "mvn {args...}").run().map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct GradleBuild;
impl BuildSystem for GradleBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("build.gradle") || sh.path_exists("build.gradle.kts")
    }
    fn name(&self) -> &'static str {
        "Gradle"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let exe = if sh.path_exists("gradlew") {
            "./gradlew"
        } else {
            "gradle"
        };
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.push("run");
        } else {
            args.push("build");
        }
        cmd!(sh, "{exe} {args...}").run().map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct ZigBuild;
impl BuildSystem for ZigBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("build.zig")
    }
    fn name(&self) -> &'static str {
        "Zig"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let run_flag = if options.test {
            Some("test")
        } else if options.run {
            Some("run")
        } else {
            None
        };
        let opt = if options.release {
            Some("-Doptimize=ReleaseFast")
        } else {
            None
        };
        cmd!(sh, "zig build {run_flag...} {opt...}")
            .run()
            .map_err(BuildError::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct DotnetBuild;
impl BuildSystem for DotnetBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.read_dir(".")
            .map(|entries| {
                entries.iter().any(|e| {
                    e.extension()
                        .is_some_and(|ext| ext == "sln" || ext == "csproj" || ext == "fsproj")
                })
            })
            .unwrap_or(false)
    }
    fn name(&self) -> &'static str {
        ".NET"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        let config = if options.release {
            vec!["-c", "Release"]
        } else {
            vec![]
        };
        cmd!(sh, "dotnet {verb} {config...}")
            .run()
            .map_err(BuildError::from)
    }
}
