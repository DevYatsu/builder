use crate::error::Result;
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

    fn description(&self) -> &'static str {
        "Build and run Go projects"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
        let verb = options.verb();
        cmd!(sh, "go {verb} .").run()?;
        Ok(None)
    }
}
