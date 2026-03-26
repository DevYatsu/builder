use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct DotnetBuild;

impl BuildSystem for DotnetBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.read_dir(".")
            .map(|entries| {
                entries.iter().any(|p| {
                    p.extension()
                        .is_some_and(|ext| ext == "csproj" || ext == "fsproj" || ext == "sln")
                })
            })
            .unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        ".NET"
    }

    fn description(&self) -> &'static str {
        "Build and run .NET projects"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let verb = options.verb();
        let config = if options.release { "Release" } else { "Debug" };
        cmd!(sh, "dotnet {verb} -c {config}").run()?;
        Ok(())
    }
}
