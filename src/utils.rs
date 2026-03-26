use crate::error::{Result, YbuildError};
use std::path::{Path, PathBuf};
use std::process::Command;
use xshell::Shell;

pub fn execute_interactive(sh: &Shell, cmd_name: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd_name)
        .args(args)
        .current_dir(sh.current_dir())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(YbuildError::Other(format!(
            "Command failed with exit code: {:?}",
            status.code()
        )))
    }
}

pub fn select_command(prompt: &str, options: Vec<String>) -> Result<String> {
    use dialoguer::{Select, theme::ColorfulTheme};
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&options)
        .default(0)
        .interact_opt()
        .map_err(|e| YbuildError::Other(e.to_string()))?;

    selection
        .map(|idx| options[idx].clone())
        .ok_or(YbuildError::SelectionCanceled)
}

pub fn select_option(prompt: &str, options: Vec<String>) -> Result<Option<String>> {
    match options.len() {
        0 => Ok(None),
        1 => Ok(Some(options[0].clone())),
        _ => Ok(Some(select_command(prompt, options)?)),
    }
}

pub fn execute_recently_modified_binary(sh: &Shell, search_dir: &str) -> Result<()> {
    let skip_dirs = [
        ".git",
        "node_modules",
        ".venv",
        "zig-cache",
        "CMakeFiles",
        ".swiftpm",
        ".dart_tool",
        "__pycache__",
        "obj",
    ];

    let mut executables = Vec::new();
    let mut dirs = vec![PathBuf::from(search_dir)];

    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = sh.read_dir(&dir) {
            for path in entries {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if name.starts_with('.') || skip_dirs.contains(&name) {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(path);
                } else if is_executable(&path) {
                    executables.push(path);
                }
            }
        }
    }

    if executables.is_empty() {
        log::warn!("No executables found in {}", search_dir);
        return Ok(());
    }

    executables.sort_by_cached_key(|path| {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH)
    });
    executables.reverse();

    let options: Vec<String> = executables
        .iter()
        .filter_map(|p| p.to_str().map(|s| s.to_string()))
        .collect();

    if let Some(selected) = select_option("Select executable to run", options)? {
        log::info!("executing: {}", selected);
        execute_interactive(sh, &selected, &[])
    } else {
        Ok(())
    }
}

pub fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                let ext = ext.to_lowercase();
                ext == "exe" || ext == "bat" || ext == "cmd"
            })
            .unwrap_or(false)
    }
}
