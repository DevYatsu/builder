use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct DockerBuild;

impl BuildSystem for DockerBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Dockerfile")
    }

    fn name(&self) -> &'static str {
        "Docker"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        cmd!(sh, "docker build . -t app_image").run()?;
        if options.run {
            cmd!(sh, "docker run -it --rm app_image").run()?;
        }
        Ok(())
    }
}
