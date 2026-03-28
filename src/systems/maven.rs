use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct MavenBuild;

impl BuildSystem for MavenBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pom.xml")
    }

    fn name(&self) -> &'static str {
        "Maven"
    }

    fn description(&self) -> &'static str {
        "Build and run Java projects using Maven"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
        let goal = if options.test { "test" } else { "install" };
        cmd!(sh, "mvn {goal}").run()?;
        if options.run {
            cmd!(sh, "mvn exec:java").run()?;
        }
        Ok(None)
    }
}
