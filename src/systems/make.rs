use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use crate::utils::execute_recently_modified_binary;
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct MakeBuild;

impl BuildSystem for MakeBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("Makefile")
    }

    fn name(&self) -> &'static str {
        "Makefile"
    }

    fn description(&self) -> &'static str {
        "Build and run projects using Makefile"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
        if options.select_command {
            // Try to get targets (simplified)
            let output = cmd!(sh, "make -pR").read().unwrap_or_default();
            let mut targets = Vec::new();
            for line in output.lines() {
                if !line.starts_with('.') && line.contains(':') && !line.contains('=') {
                    if let Some(target) = line.split(':').next() {
                        let target = target.trim();
                        if !target.is_empty() && !target.contains('%') {
                            targets.push(target.to_string());
                        }
                    }
                }
            }
            targets.sort();
            targets.dedup();

            if !targets.is_empty() {
                if let Some(selected) = crate::utils::select_option("Select make target", targets)?
                {
                    cmd!(sh, "make {selected}").run()?;
                    return Ok(Some(format!("make {selected}")));
                }
            }
        }

        cmd!(sh, "make").run()?;
        if options.test {
            cmd!(sh, "make test").run()?;
        }
        if options.run && crate::utils::execute_interactive(sh, "make", &["run"]).is_err() {
            execute_recently_modified_binary(sh, ".")?;
        }
        Ok(None)
    }
}
