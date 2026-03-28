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

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
        let build_type = if options.release { "Release" } else { "Debug" };

        let build_dir = "build";
        if !sh.path_exists(build_dir) {
            sh.create_dir(build_dir)?;
        }

        cmd!(sh, "cmake -B {build_dir} -DCMAKE_BUILD_TYPE={build_type}").run()?;

        if options.select_command {
            let help_output = cmd!(sh, "cmake --build {build_dir} --target help")
                .read()
                .unwrap_or_default();
            let mut targets = Vec::new();
            let mut start_parsing = false;
            for line in help_output.lines() {
                if line.contains("... ") {
                    let target = line.split("... ").nth(1).unwrap_or("").trim();
                    if !target.is_empty() {
                        targets.push(target.to_string());
                    }
                } else if line.contains("The following targets are available:") {
                    start_parsing = true;
                } else if start_parsing {
                    let target = line.trim();
                    if !target.is_empty() && !target.starts_with("---") {
                        // Some generators output list after this line
                        targets.push(target.to_string());
                    }
                }
            }
            targets.sort();
            targets.dedup();

            if !targets.is_empty() {
                if let Some(selected) = crate::utils::select_option("Select CMake target", targets)?
                {
                    let full_cmd = format!("cmake --build {build_dir} --target {selected}");
                    cmd!(sh, "cmake --build {build_dir} --target {selected}").run()?;
                    return Ok(Some(full_cmd));
                }
            }
        }

        cmd!(sh, "cmake --build {build_dir}").run()?;

        if options.run {
            crate::utils::execute_recently_modified_binary(sh, build_dir)?;
        }

        Ok(None)
    }
}
