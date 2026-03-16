use crate::error::{BuildError, Result};
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct PythonBuild;

impl BuildSystem for PythonBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("requirements.txt") || sh.path_exists("setup.py")
    }

    fn name(&self) -> &'static str {
        "Python"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let py = if sh.path_exists(".venv") {
            #[cfg(unix)]
            {
                ".venv/bin/python"
            }
            #[cfg(windows)]
            {
                ".venv\\Scripts\\python.exe"
            }
        } else {
            "python3"
        };
        if options.test {
            cmd!(sh, "{py} -m pytest").run().map_err(BuildError::from)
        } else if options.run {
            let entry = if sh.path_exists("main.py") {
                "main.py"
            } else {
                "."
            };
            cmd!(sh, "{py} {entry}").run().map_err(BuildError::from)
        } else {
            cmd!(sh, "{py} -m pip install -r requirements.txt")
                .run()
                .map_err(BuildError::from)
        }
    }
}
