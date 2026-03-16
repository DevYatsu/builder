use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct NodeBuild;

impl BuildSystem for NodeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("package.json")
    }

    fn name(&self) -> &'static str {
        "Node.js"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let pm = if sh.path_exists("pnpm-lock.yaml") {
            "pnpm"
        } else if sh.path_exists("yarn.lock") {
            "yarn"
        } else {
            "npm"
        };
        let script = if options.test {
            "test"
        } else if options.run {
            if sh.path_exists("main.js") || sh.path_exists("index.js") {
                return cmd!(sh, "node .").run().map_err(BuildError::from);
            }
            "start"
        } else {
            "build"
        };
        cmd!(sh, "{pm} run {script}")
            .run()
            .map_err(BuildError::from)
    }
}
