use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct FlutterBuild;

impl BuildSystem for FlutterBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pubspec.yaml") && sh.path_exists("lib")
    }

    fn name(&self) -> &'static str {
        "Flutter"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        let rel = if options.release && verb != "test" {
            vec!["--release"]
        } else {
            vec![]
        };
        cmd!(sh, "flutter {verb} {rel...}")
            .run()
            .map_err(BuildError::from)
    }
}
