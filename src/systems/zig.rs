use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct ZigBuild;

impl BuildSystem for ZigBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("build.zig")
    }

    fn name(&self) -> &'static str {
        "Zig"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let run_flag = if options.test {
            Some("test")
        } else if options.run {
            Some("run")
        } else {
            None
        };
        let opt = if options.release {
            Some("-Doptimize=ReleaseFast")
        } else {
            None
        };
        cmd!(sh, "zig build {run_flag...} {opt...}")
            .run()
            .map_err(BuildError::from)
    }
}
