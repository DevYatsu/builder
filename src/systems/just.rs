use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct JustBuild;

impl BuildSystem for JustBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("justfile") || sh.path_exists("Justfile")
    }

    fn name(&self) -> &'static str {
        "Just"
    }

    fn description(&self) -> &'static str {
        "Build and run projects using the just command runner"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
        if options.select_command {
            let output = cmd!(sh, "just --summary").read().unwrap_or_default();
            let recipes: Vec<String> = output.split_whitespace().map(|s| s.to_string()).collect();

            if !recipes.is_empty() {
                if let Some(selected) = crate::utils::select_option("Select just recipe", recipes)?
                {
                    cmd!(sh, "just {selected}").run()?;
                    return Ok(Some(format!("just {selected}")));
                }
            }
        }

        let recipe = options.verb();
        cmd!(sh, "just {recipe}").run()?;
        Ok(None)
    }
}
