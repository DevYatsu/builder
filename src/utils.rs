use crate::error::{BuildError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use xshell::Shell;

pub fn execute_interactive(sh: &Shell, cmd_name: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd_name)
        .args(args)
        .current_dir(sh.current_dir())
        .status()
        .map_err(BuildError::from)?;
    if status.success() {
        Ok(())
    } else {
        Err(BuildError::NoSystemFound) // Temporary error
    }
}

pub fn execute_recently_modified_binary(sh: &Shell, search_dir: &str) -> Result<()> {
    let mut most_recent = None;
    let mut max_time = std::time::UNIX_EPOCH;
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
    let mut dirs = if search_dir != "." {
        vec![PathBuf::from(search_dir)]
    } else {
        vec![PathBuf::from(".")]
    };
    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = sh.read_dir(&dir) {
            for path in entries {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || skip_dirs.contains(&name.as_ref()) {
                    continue;
                }
                let full_path = sh.current_dir().join(&path);
                if full_path.is_dir() {
                    dirs.push(path);
                } else if is_executable(&full_path)
                    && let Ok(meta) = std::fs::metadata(&full_path)
                    && let Ok(modified) = meta.modified()
                    && modified > max_time
                {
                    max_time = modified;
                    most_recent = Some(path);
                }
            }
        }
    }
    if let Some(exe) = most_recent {
        log::info!("executing: {}", exe.display());
        execute_interactive(sh, sh.current_dir().join(exe).to_str().unwrap(), &[])
    } else {
        log::warn!("no executable found");
        Ok(())
    }
}

pub fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.is_file() && meta.permissions().mode() & 0o111 != 0;
        }
    }
    #[cfg(windows)]
    {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return ext_str == "exe" || ext_str == "bat" || ext_str == "cmd";
        }
    }
    false
}
