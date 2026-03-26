use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use tinyjson::JsonValue;
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct JavaScriptBuild;

impl BuildSystem for JavaScriptBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("package.json")
            || sh.path_exists("bun.lockb")
            || sh.path_exists("bun.lock")
            || sh.path_exists("pnpm-lock.yaml")
            || sh.path_exists("yarn.lock")
            || sh.path_exists("package-lock.json")
            || sh.path_exists("index.js")
            || sh.path_exists("main.ts")
            || sh.path_exists("index.ts")
    }

    fn name(&self) -> &'static str {
        "JavaScript/TypeScript"
    }

    fn description(&self) -> &'static str {
        "Build and run projects using Node.js, Bun, or Deno"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let pm = self.detect_package_manager(sh)?;

        if options.test {
            return cmd!(sh, "{pm} test").run().map_err(|e| e.into());
        }

        if options.run {
            let script_names = self.get_json_commands(sh, "package.json", "scripts");

            if let Some(selected) =
                crate::utils::select_option("Select script to run", script_names)?
            {
                return cmd!(sh, "{pm} run {selected}").run().map_err(|e| e.into());
            }

            // Fallbacks for projects without explicit scripts
            if pm == "bun" {
                return cmd!(sh, "bun run .").run().map_err(|e| e.into());
            }

            for entry in ["index.js", "main.ts", "index.ts", "server.js"] {
                if sh.path_exists(entry) {
                    return cmd!(sh, "node {entry}").run().map_err(|e| e.into());
                }
            }
        }

        Ok(())
    }
}

impl JavaScriptBuild {
    fn detect_package_manager(&self, sh: &Shell) -> Result<String> {
        let mut managers = Vec::new();
        if sh.path_exists("bun.lockb") || sh.path_exists("bun.lock") {
            managers.push("bun");
        }
        if sh.path_exists("pnpm-lock.yaml") {
            managers.push("pnpm");
        }
        if sh.path_exists("yarn.lock") {
            managers.push("yarn");
        }
        if sh.path_exists("package-lock.json") {
            managers.push("npm");
        }

        match managers.len() {
            0 => {
                if cmd!(sh, "bun --version").run().is_ok() {
                    Ok("bun".to_string())
                } else if cmd!(sh, "deno --version").run().is_ok() {
                    Ok("deno".to_string())
                } else {
                    Ok("npm".to_string())
                }
            }
            1 => Ok(managers[0].to_string()),
            _ => {
                let options = managers.iter().map(|s| s.to_string()).collect();
                crate::utils::select_command(
                    "Multiple lockfiles detected. Select package manager:",
                    options,
                )
            }
        }
    }

    fn get_json_commands(&self, sh: &Shell, file: &str, field: &str) -> Vec<String> {
        sh.read_file(file)
            .ok()
            .and_then(|c| c.parse::<JsonValue>().ok())
            .and_then(|j| match j {
                JsonValue::Object(obj) => obj.get(field).cloned(),
                _ => None,
            })
            .and_then(|v| match v {
                JsonValue::Object(map) => Some(map.keys().cloned().collect()),
                _ => None,
            })
            .unwrap_or_default()
    }
}
