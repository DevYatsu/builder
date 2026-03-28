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

    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<Option<String>> {
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

        if options.run || options.select_command {
            let mut entry = ["main.py", "app.py"]
                .into_iter()
                .find(|f| sh.path_exists(f))
                .unwrap_or("main.py")
                .to_string();

            if options.select_command {
                let py_files: Vec<String> = sh
                    .read_dir(".")?
                    .into_iter()
                    .filter(|p| p.extension().map_or(false, |ext| ext == "py"))
                    .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                    .collect();

                if !py_files.is_empty() {
                    if let Some(selected) =
                        crate::utils::select_option("Select python script to run", py_files)?
                    {
                        entry = selected;
                    }
                }
            }

            let full_cmd = format!("{py} {entry}");
            crate::utils::execute_interactive(sh, py, &[&entry])?;
            return Ok(Some(full_cmd));
        }

        Ok(None)
    }
}
