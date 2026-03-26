use crate::error::Result;
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

    fn description(&self) -> &'static str {
        "Build and run projects using Gradle"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let task = if options.test { "test" } else { "build" };
        cmd!(sh, "./gradlew {task}").run()?;
        if options.run {
            cmd!(sh, "./gradlew run").run()?;
        }
        Ok(())
    }
}
