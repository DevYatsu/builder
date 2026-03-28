use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone)]
pub struct CustomBuild {
    pub command: String,
}

impl BuildSystem for CustomBuild {
    fn detect(&self, _sh: &Shell) -> bool {
        // Custom commands are manually specified, not detected
        false
    }

    fn name(&self) -> &'static str {
        "Custom Command"
    }

    fn description(&self) -> &'static str {
        "Execute a manually specified command"
    }

    fn execute(&self, sh: &Shell, _options: &BuildOptions) -> Result<Option<String>> {
        // We split the command string to handle arguments correctly with xshell
        // This is a simple implementation; more complex parsing might be needed for quoted strings
        let parts: Vec<&str> = self.command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        let program = parts[0];
        let args = &parts[1..];

        cmd!(sh, "{program} {args...}").run()?;
        Ok(None)
    }
}
