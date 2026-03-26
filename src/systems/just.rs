use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct JustBuild;

impl BuildSystem for JustBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("justfile") || sh.path_exists("Justfile")
    }

    fn name(&self) -> &'static str {
        "Just"
    }

    fn description(&self) -> &'static str {
        "Build and run projects using the just command runner"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let recipe = options.verb();
        cmd!(sh, "just {recipe}").run()?;
        Ok(())
    }
}
