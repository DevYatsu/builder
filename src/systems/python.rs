use crate::error::Result;
use crate::systems::{BuildOptions, BuildSystem};
use xshell::{Shell, cmd};

#[derive(Debug, Clone, Copy)]
pub struct PythonBuild;

impl BuildSystem for PythonBuild {
    fn detect(&self, sh: &Shell) -> bool {
        sh.path_exists("requirements.txt")
            || sh.path_exists("setup.py")
            || sh.path_exists("pyproject.toml")
            || sh.path_exists("main.py")
            || sh.path_exists("app.py")
    }

    fn name(&self) -> &'static str {
        "Python"
    }

    fn description(&self) -> &'static str {
        "Run Python scripts and projects"
    }

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()> {
        let py = if sh.path_exists(".venv") {
            if cfg!(windows) {
                ".venv\\Scripts\\python.exe"
            } else {
                ".venv/bin/python"
            }
        } else {
            "python3"
        };

        if options.test {
            cmd!(sh, "{py} -m pytest").run()?;
        }

        if sh.path_exists("requirements.txt") {
            cmd!(sh, "{py} -m pip install -r requirements.txt").run()?;
        }

        if options.run {
            let entry = ["main.py", "app.py"]
                .into_iter()
                .find(|f| sh.path_exists(f))
                .unwrap_or(".");
            crate::utils::execute_interactive(sh, py, &[entry])?;
        }

        Ok(())
    }
}
