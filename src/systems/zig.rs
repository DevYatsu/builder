use crate::error::Result;
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

    fn description(&self) -> &'static str {
        "Build and run Zig projects"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.push("run");
        } else {
            args.push("build");
        }
        if options.release {
            args.extend(["-Doptimize", "ReleaseSafe"]);
        }
        cmd!(sh, "zig build {args...}").run()?;
        Ok(())
    }
}
