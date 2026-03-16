use crate::error::{BuildError, Result};
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

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        let config = if options.release {
            vec!["-c", "release"]
        } else {
            vec![]
        };
        cmd!(sh, "swift {verb} {config...}")
            .run()
            .map_err(BuildError::from)
    }
}
