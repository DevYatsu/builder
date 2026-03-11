use std::fs;
use std::path::Path;
use xshell::{cmd, Shell};
use log::warn;

pub struct BuildOptions {
    pub run: bool,
    pub release: bool,
}

pub trait BuildSystem {
    fn detect(&self) -> bool;
    fn name(&self) -> &'static str;
    fn execute(&self, sh: &Shell, options: &BuildOptions);
}

pub fn get_systems() -> Vec<Box<dyn BuildSystem>> {
    vec![
        Box::new(RustBuild),
        Box::new(MakeBuild),
        Box::new(CMakeBuild),
        Box::new(NodeBuild),
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
    fn detect(&self) -> bool { Path::new("Cargo.toml").exists() }
    fn name(&self) -> &'static str { "Rust" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let verb = if options.run { "run" } else { "build" };
        let rel = if options.release { Some("--release") } else { None };
        cmd!(sh, "cargo {verb} {rel...}").run().unwrap();
    }
}

struct MakeBuild;
impl BuildSystem for MakeBuild {
    fn detect(&self) -> bool { Path::new("Makefile").exists() }
    fn name(&self) -> &'static str { "Makefile" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let target = if options.run { Some("run") } else { None };
        cmd!(sh, "make {target...}").run().unwrap();
    }
}

struct CMakeBuild;
impl BuildSystem for CMakeBuild {
    fn detect(&self) -> bool { Path::new("CMakeLists.txt").exists() }
    fn name(&self) -> &'static str { "CMake" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let build_dir = if Path::new("build").exists() { "build" } else { "." };
        
        if build_dir == "build" && !Path::new("build/CMakeCache.txt").exists() {
            let mut args = vec!["-B", "build", "-S", "."];
            if cmd!(sh, "ninja --version").read().is_ok() {
                args.extend(["-G", "Ninja"]);
            }
            if options.release { args.push("-DCMAKE_BUILD_TYPE=Release"); }
            cmd!(sh, "cmake {args...}").run().unwrap();
        }

        let config = if options.release { Some("Release") } else { None };
        cmd!(sh, "cmake --build {build_dir} --config {config...}").run().unwrap();
        
        if options.run { warn!("CMake cannot natively 'run' target. Use -e."); }
    }
}

struct NodeBuild;
impl BuildSystem for NodeBuild {
    fn detect(&self) -> bool { Path::new("package.json").exists() }
    fn name(&self) -> &'static str { "Node.js" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let script = if options.run { "start" } else { "build" };
        cmd!(sh, "npm run {script}").run().unwrap();
    }
}

struct GoBuild;
impl BuildSystem for GoBuild {
    fn detect(&self) -> bool { Path::new("go.mod").exists() }
    fn name(&self) -> &'static str { "Go" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        if options.run {
            cmd!(sh, "go run .").run().unwrap();
        } else {
            cmd!(sh, "go build").run().unwrap();
        }
    }
}

struct DockerBuild;
impl BuildSystem for DockerBuild {
    fn detect(&self) -> bool { Path::new("Dockerfile").exists() }
    fn name(&self) -> &'static str { "Docker" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        cmd!(sh, "docker build . -t app_image").run().unwrap();
        if options.run {
            cmd!(sh, "docker run -it --rm app_image").run().unwrap();
        }
    }
}

struct MavenBuild;
impl BuildSystem for MavenBuild {
    fn detect(&self) -> bool { Path::new("pom.xml").exists() }
    fn name(&self) -> &'static str { "Maven" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let target = if options.run { "spring-boot:run" } else { "package" };
        cmd!(sh, "mvn {target}").run().unwrap();
    }
}

struct GradleBuild;
impl BuildSystem for GradleBuild {
    fn detect(&self) -> bool { Path::new("build.gradle").exists() || Path::new("build.gradle.kts").exists() }
    fn name(&self) -> &'static str { "Gradle" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let exe = if Path::new("gradlew").exists() { "./gradlew" } else { "gradle" };
        let target = if options.run { "run" } else { "build" };
        cmd!(sh, "{exe} {target}").run().unwrap();
    }
}

struct ZigBuild;
impl BuildSystem for ZigBuild {
    fn detect(&self) -> bool { Path::new("build.zig").exists() }
    fn name(&self) -> &'static str { "Zig" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let run_flag = if options.run { Some("run") } else { None };
        let opt = if options.release { Some("-Doptimize=ReleaseFast") } else { None };
        cmd!(sh, "zig build {run_flag...} {opt...}").run().unwrap();
    }
}

struct DotnetBuild;
impl BuildSystem for DotnetBuild {
    fn detect(&self) -> bool {
        fs::read_dir(".").map(|entries| {
            entries.flatten().any(|e| {
                e.path().extension().map_or(false, |ext| {
                    ext == "sln" || ext == "csproj" || ext == "fsproj"
                })
            })
        }).unwrap_or(false)
    }
    fn name(&self) -> &'static str { ".NET" }
    fn execute(&self, sh: &Shell, options: &BuildOptions) {
        let verb = if options.run { "run" } else { "build" };
        let config = if options.release { Some("Release") } else { None };
        cmd!(sh, "dotnet {verb} -c {config...}").run().unwrap();
    }
}
