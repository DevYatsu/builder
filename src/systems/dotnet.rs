use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct DotnetBuild;

impl BuildSystem for DotnetBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.read_dir(".")
            .map(|entries| {
                entries.iter().any(|e| {
                    e.extension()
                        .is_some_and(|ext| ext == "sln" || ext == "csproj" || ext == "fsproj")
                })
            })
            .unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        ".NET"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = if options.test {
            "test"
        } else if options.run {
            "run"
        } else {
            "build"
        };
        let config = if options.release {
            vec!["-c", "Release"]
        } else {
            vec![]
        };
        cmd!(sh, "dotnet {verb} {config...}")
            .run()
            .map_err(BuildError::from)
    }
}
