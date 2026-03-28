use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::Shell;

#[derive(Debug, Clone, Copy)]
pub struct RustBuild;

impl BuildSystem for RustBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Cargo.toml")
    }

    fn name(&self) -> &'static str {
        "Rust"
    }

    fn description(&self) -> &'static str {
        "Build and run Rust projects using Cargo"
    }

    fn execute(&self, _sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
        let verb = options.verb();
        let mut args = vec![verb];
        if options.release {
            args.push("--release");
        }
        crate::utils::execute_interactive(_sh, "cargo", &args)?;
        Ok(None)
    }
}
