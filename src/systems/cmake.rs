use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use crate::utils::execute_recently_modified_binary;
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct CMakeBuild;

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
