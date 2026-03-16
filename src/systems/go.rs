use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct GoBuild;

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
