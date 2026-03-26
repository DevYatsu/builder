use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
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

    fn description(&self) -> &'static str {
        "Build and run C/C++ projects using CMake"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let build_type = if options.release { "Release" } else { "Debug" };

        let build_dir = "build";
        if !sh.path_exists(build_dir) {
            sh.create_dir(build_dir)?;
        }

        cmd!(sh, "cmake -B {build_dir} -DCMAKE_BUILD_TYPE={build_type}").run()?;
        cmd!(sh, "cmake --build {build_dir}").run()?;

        if options.run {
            crate::utils::execute_recently_modified_binary(sh, build_dir)?;
        }

        Ok(())
    }
}
