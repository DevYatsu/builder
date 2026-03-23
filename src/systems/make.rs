use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use crate::utils::execute_recently_modified_binary;
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct MakeBuild;

impl BuildSystem for MakeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Makefile")
    }

    fn name(&self) -> &'static str {
        "Makefile"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        cmd!(sh, "make").run()?;
        if options.test {
            cmd!(sh, "make test").run()?;
        }
        if options.run {
            if crate::utils::execute_interactive(sh, "make", &["run"]).is_err() {
                execute_recently_modified_binary(sh, ".")?;
            }
        }
        Ok(())
    }
}
