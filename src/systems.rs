use xshell::{Shell, cmd};

pub struct BuildOptions {
    pub run: bool,
    pub release: bool,
    pub test: bool,
}

pub trait BuildSystem {
    fn detect(&self, sh: &Shell) -> bool;
    fn name(&self) -> &'static str;
    fn execute(&self, sh: &Shell, options: &BuildOptions);
}

pub fn get_systems() -> Vec<Box<dyn BuildSystem>> {
    vec![
        Box::new(RustBuild),
        Box::new(MakeBuild),
        Box::new(CMakeBuild),
        Box::new(NodeBuild),
        Box::new(BunBuild),
        Box::new(DenoBuild),
        Box::new(GoBuild),
        Box::new(DockerBuild),
        Box::new(MavenBuild),
        Box::new(GradleBuild),
        Box::new(ZigBuild),
        Box::new(DotnetBuild),
    ]
}

struct RustBuild;
impl BuildSystem for RustBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Cargo.toml")
    }
    fn name(&self) -> &'static str {
        "Rust"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
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
        if let Err(e) = cmd!(sh, "cargo {verb} {rel...}").run() {
            log::error!("{verb} failed: {e}");
            std::process::exit(1);
        }
    }
}

struct MakeBuild;
impl BuildSystem for MakeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Makefile")
    }
    fn name(&self) -> &'static str {
        "Makefile"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        if let Err(e) = cmd!(sh, "make").run() {
            log::error!("build failed: {e}");
            std::process::exit(1);
        }

        if options.test
            && let Err(e) = cmd!(sh, "make test").run()
        {
            log::error!("tests failed: {e}");
            std::process::exit(1);
        }

        if options.run && cmd!(sh, "make run").run().is_err() {
            execute_recently_modified_binary(sh);
        }
    }
}

struct CMakeBuild;
impl BuildSystem for CMakeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("CMakeLists.txt")
    }
    fn name(&self) -> &'static str {
        "CMake"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let has_build_cache = sh.path_exists("build/CMakeCache.txt");
        let has_root_cache = sh.path_exists("CMakeCache.txt");

        let build_dir = if has_build_cache {
            "build"
        } else if has_root_cache {
            "."
        } else {
            let mut args = vec!["-B", "build", "-S", "."];
            if cmd!(sh, "ninja --version").read().is_ok() {
                args.extend(["-G", "Ninja"]);
            }
            if options.release {
                args.push("-DCMAKE_BUILD_TYPE=Release");
            }
            if let Err(e) = cmd!(sh, "cmake {args...}").run() {
                log::error!("configuration failed: {e}");
                std::process::exit(1);
            }
            "build"
        };

        let config = if options.release {
            vec!["--config", "Release"]
        } else {
            vec![]
        };
        if let Err(e) = cmd!(sh, "cmake --build {build_dir} {config...}").run() {
            log::error!("build failed: {e}");
            std::process::exit(1);
        }

        if options.test
            && let Err(e) = cmd!(sh, "ctest --test-dir {build_dir}").run()
        {
            log::error!("tests failed: {e}");
            std::process::exit(1);
        }

        if options.run {
            execute_recently_modified_binary(sh);
        }
    }
}

fn execute_recently_modified_binary(sh: &Shell) {
    let mut most_recent = None;
    let mut max_time = std::time::UNIX_EPOCH;

    let mut dirs = vec![std::path::PathBuf::from(".")];
    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = sh.read_dir(&dir) {
            for path in entries {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                
                if name.starts_with('.')
                    || name == "node_modules"
                    || name == "deps"
                    || name == "target"
                {
                    continue;
                }

                let full_path = sh.current_dir().join(&path);
                if full_path.is_dir() {
                    dirs.push(path);
                } else {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = std::fs::metadata(&full_path) {
                            if meta.is_file() 
                                && meta.permissions().mode() & 0o111 != 0 
                                && !name.ends_with(".sh")
                                && !name.ends_with(".rs")
                                && !name.ends_with(".txt")
                            {
                                if let Ok(modified) = meta.modified() {
                                    if modified > max_time {
                                        max_time = modified;
                                        most_recent = Some(path);
                                    }
                                }
                            }
                        }
                    }
                    #[cfg(windows)]
                    {
                        if name.ends_with(".exe") {
                            if let Ok(meta) = std::fs::metadata(&full_path) {
                                if let Ok(modified) = meta.modified() {
                                    if modified > max_time {
                                        max_time = modified;
                                        most_recent = Some(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(exe) = most_recent {
        log::info!("executing: {}", exe.display());
        if let Err(e) = cmd!(sh, "{exe}").run() {
            log::error!("execution failed: {e}");
            std::process::exit(1);
        }
    } else {
        log::warn!("no executable found");
    }
}

struct NodeBuild;
impl BuildSystem for NodeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("package.json")
    }
    fn name(&self) -> &'static str {
        "Node.js"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
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
            "start"
        } else {
            "build"
        };

        if let Err(e) = cmd!(sh, "{pm} run {script}").run() {
            log::error!("{pm} {script} failed: {e}");
            std::process::exit(1);
        }
    }
}

struct BunBuild;
impl BuildSystem for BunBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("bun.lockb") || sh.path_exists("bunfig.toml")
    }
    fn name(&self) -> &'static str {
        "Bun"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let cmd = if options.test {
            "test"
        } else if options.run {
            "run ."
        } else {
            "run build"
        };
        if let Err(e) = cmd!(sh, "bun {cmd}").run() {
            log::error!("bun {cmd} failed: {e}");
            std::process::exit(1);
        }
    }
}

struct DenoBuild;
impl BuildSystem for DenoBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("deno.json") || sh.path_exists("deno.jsonc")
    }
    fn name(&self) -> &'static str {
        "Deno"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let cmd = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "task build"
        };
        
        let mut args = vec![cmd];
        if cmd == "run" {
            args.push("-A");
            if sh.path_exists("main.ts") {
                args.push("main.ts");
            } else if sh.path_exists("main.js") {
                args.push("main.js");
            }
        }

        if let Err(e) = cmd!(sh, "deno {args...}").run() {
            log::error!("deno {cmd} failed: {e}");
            std::process::exit(1);
        }
    }
}

struct GoBuild;
impl BuildSystem for GoBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("go.mod")
    }
    fn name(&self) -> &'static str {
        "Go"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let res = if options.test {
            cmd!(sh, "go test ./...").run()
        } else if options.run {
            cmd!(sh, "go run .").run()
        } else {
            cmd!(sh, "go build").run()
        };
        if let Err(e) = res {
            log::error!("go command failed: {e}");
            std::process::exit(1);
        }
    }
}

struct DockerBuild;
impl BuildSystem for DockerBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Dockerfile")
    }
    fn name(&self) -> &'static str {
        "Docker"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        if let Err(e) = cmd!(sh, "docker build . -t app_image").run() {
            log::error!("docker build failed: {e}");
            std::process::exit(1);
        }
        if options.run
            && let Err(e) = cmd!(sh, "docker run -it --rm app_image").run()
        {
            log::error!("docker run failed: {e}");
            std::process::exit(1);
        }
    }
}

struct MavenBuild;
impl BuildSystem for MavenBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pom.xml")
    }
    fn name(&self) -> &'static str {
        "Maven"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let target = if options.test {
            "test"
        } else if options.run {
            "spring-boot:run"
        } else {
            "package"
        };
        if let Err(e) = cmd!(sh, "mvn {target}").run() {
            log::error!("maven target failed: {e}");
            std::process::exit(1);
        }
    }
}

struct GradleBuild;
impl BuildSystem for GradleBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("build.gradle") || sh.path_exists("build.gradle.kts")
    }
    fn name(&self) -> &'static str {
        "Gradle"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let exe = if sh.path_exists("gradlew") {
            "./gradlew"
        } else {
            "gradle"
        };
        let target = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        if let Err(e) = cmd!(sh, "{exe} {target}").run() {
            log::error!("gradle target failed: {e}");
            std::process::exit(1);
        }
    }
}

struct ZigBuild;
impl BuildSystem for ZigBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("build.zig")
    }
    fn name(&self) -> &'static str {
        "Zig"
    }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
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
        if let Err(e) = cmd!(sh, "zig build {run_flag...} {opt...}").run() {
            log::error!("zig build failed: {e}");
            std::process::exit(1);
        }
    }
}

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
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
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
        if let Err(e) = cmd!(sh, "dotnet {verb} {config...}").run() {
            log::error!("dotnet {verb} failed: {e}");
            std::process::exit(1);
        }
    }
}
