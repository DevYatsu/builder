use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct UvBuild;

impl BuildSystem for UvBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pyproject.toml") && cmd!(sh, "uv --version").run().is_ok()
    }

    fn name(&self) -> &'static str {
        "uv"
    }

    fn description(&self) -> &'static str {
        "Build and run Python projects using the uv package manager"
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
        cmd!(sh, "uv {args...}").run()?;
        Ok(())
    }
}
