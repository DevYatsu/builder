use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct RustBuild;

impl BuildSystem for RustBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Cargo.toml")
    }

    fn name(&self) -> &'static str {
        "Rust"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = options.verb();
        let rel = if options.release {
            vec!["--release"]
        } else {
            vec![]
        };
        cmd!(sh, "cargo {verb} {rel...}")
            .run()
            .map_err(BuildError::from)
    }
}
