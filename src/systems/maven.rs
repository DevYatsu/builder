use crate::error::{BuildError, Result};
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

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.push("spring-boot:run");
        } else {
            args.push("package");
        }
        cmd!(sh, "mvn {args...}").run().map_err(BuildError::from)
    }
}
