use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct SwiftBuild;

impl BuildSystem for SwiftBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Package.swift")
    }

    fn name(&self) -> &'static str {
        "Swift"
    }

    fn description(&self) -> &'static str {
        "Build and run Swift projects"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = options.verb();
        let config = if options.release { "release" } else { "debug" };
        cmd!(sh, "swift {verb} -c {config}").run()?;
        Ok(())
    }
}
