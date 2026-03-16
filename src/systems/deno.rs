use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct DenoBuild;

impl BuildSystem for DenoBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("deno.json") || sh.path_exists("deno.jsonc")
    }

    fn name(&self) -> &'static str {
        "Deno"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let mut args = vec![];
        if options.test {
            args.push("test");
        } else if options.run {
            args.extend(["run", "-A"]);
            if sh.path_exists("main.ts") {
                args.push("main.ts");
            } else if sh.path_exists("index.ts") {
                args.push("index.ts");
            } else {
                args.push(".");
            }
        } else {
            args.extend(["task", "build"]);
        }
        cmd!(sh, "deno {args...}").run().map_err(BuildError::from)
    }
}
