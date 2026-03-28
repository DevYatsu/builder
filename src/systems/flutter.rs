use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct FlutterBuild;

impl BuildSystem for FlutterBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("pubspec.yaml")
            && sh
                .read_file("pubspec.yaml")
                .map(|c| c.contains("flutter:"))
                .unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        "Flutter"
    }

    fn description(&self) -> &'static str {
        "Build and run Flutter apps"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
        let verb = match options.verb() {
            "test" => "test",
            "run" => "run",
            _ => "build",
        };
        let mut args = vec![verb];
        if options.release && verb != "test" {
            args.push("--release");
        }
        cmd!(sh, "flutter {args...}").run()?;
        Ok(None)
    }
}
