use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct GradleBuild;

impl BuildSystem for GradleBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("build.gradle") || sh.path_exists("build.gradle.kts")
    }

    fn name(&self) -> &'static str {
        "Gradle"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let exe = if sh.path_exists("gradlew") {
            "./gradlew"
        } else {
            "gradle"
        };
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.push("run");
        } else {
            args.push("build");
        }
        cmd!(sh, "{exe} {args...}").run().map_err(BuildError::from)
    }
}
