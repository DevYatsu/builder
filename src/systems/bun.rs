use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct BunBuild;

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
