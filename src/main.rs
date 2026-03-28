mod error;
mod logger;
mod systems;
mod utils;

use crate::error::{Result, YbuildError};
use log::{error, info};
use std::path::{Path, PathBuf};
use std::time::Duration;
use systems::{BuildOptions, get_systems};
use xshell::Shell;

struct Cli {
    options: BuildOptions,
    target_dir: Option<PathBuf>,
    watch: bool,
    select_system: bool,
    select_command: bool,
    custom_command: Option<String>,
}

impl Cli {
    fn from_args() -> Result<Self> {
        let mut args = pico_args::Arguments::from_env();
        if args.contains(["-h", "--help"]) {
            print_help();
            std::process::exit(0);
        }

        if args.contains(["-l", "--list"]) {
            println!("Supported build systems:");
            for sys in get_systems() {
                println!("  - {}", sys.name());
            }
            std::process::exit(0);
        }

        let options = BuildOptions {
            release: args.contains(["-r", "--release"]),
            run: args.contains(["-x", "--run"]),
            test: args.contains(["-t", "--test"]),
            select_system: args.contains(["-s", "--system"]),
            select_command: args.contains(["-S", "--select"]),
        };

        let watch = args.contains(["-w", "--watch"]);
        let select_system = options.select_system;
        let select_command = options.select_command;
        let custom_command = args
            .opt_value_from_str::<_, String>(["-c", "--command"])
            .map_err(|e| YbuildError::Other(e.to_string()))?;

        let target_dir = args
            .opt_free_from_str::<String>()
            .map_err(|e| YbuildError::Other(e.to_string()))?
            .map(PathBuf::from);

        Ok(Self {
            options,
            target_dir,
            watch,
            select_system,
            select_command,
            custom_command,
        })
    }
}

fn print_help() {
    let cyan = "\x1b[1;36m";
    let reset = "\x1b[0m";
    println!("{}ybuild{} - A universal build utility\n", cyan, reset);
    println!("Usage: ybuild [OPTIONS] [DIRECTORY]\n");
    println!("Options:");
    println!("  -x, --run          Build and execute");
    println!("  -t, --test         Run project tests");
    println!("  -r, --release      Enable release optimizations");
    println!("  -w, --watch        Rerun on file changes");
    println!("  -c, --command <C>  Execute custom command");
    println!("  -s, --system       Force selection of build system");
    println!("  -S, --select       Force selection of subcommand/target");
    println!("  -l, --list         List all supported systems");
    println!("  -h, --help         Show this help message\n");
}

fn get_config_path() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|base| {
        let mut path = base.home_dir().to_path_buf();
        path.push(".config");
        path.push("ybuild");
        path.push("config.json");
        path
    })
}

fn load_preference(project_path: &Path) -> Option<(String, Option<String>)> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(config_path).ok()?;
    let json: tinyjson::JsonValue = content.parse().ok()?;
    let projects = json.get::<std::collections::HashMap<String, tinyjson::JsonValue>>()?;
    let val = projects.get(project_path.to_str()?)?;

    match val {
        tinyjson::JsonValue::String(s) => Some((s.clone(), None)),
        tinyjson::JsonValue::Object(obj) => {
            let system = obj.get("system")?.get::<String>()?.clone();
            let command = obj.get("command").and_then(|v| v.get::<String>()).cloned();
            Some((system, command))
        }
        _ => None,
    }
}

fn save_preference(
    project_path: &Path,
    system_name: &str,
    custom_command: Option<&str>,
) -> Result<()> {
    let config_path = get_config_path()
        .ok_or_else(|| YbuildError::Config("Could not find home directory".to_string()))?;
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut projects = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let json: tinyjson::JsonValue = content
            .parse()
            .map_err(|e: tinyjson::JsonParseError| YbuildError::Json(e.to_string()))?;
        json.get::<std::collections::HashMap<String, tinyjson::JsonValue>>()
            .cloned()
            .unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    let mut entry = std::collections::HashMap::new();
    entry.insert("system".to_string(), system_name.to_string().into());
    if let Some(cmd) = custom_command {
        entry.insert("command".to_string(), cmd.to_string().into());
    }

    projects.insert(
        project_path
            .to_str()
            .ok_or_else(|| YbuildError::Config("Invalid project path".to_string()))?
            .to_string(),
        entry.into(),
    );

    let json = tinyjson::JsonValue::from(projects);
    std::fs::write(
        config_path,
        json.stringify()
            .map_err(|e: tinyjson::JsonGenerateError| YbuildError::Json(e.to_string()))?,
    )?;
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::from_args()?;
    let sh = Shell::new()?;
    if let Some(ref target) = cli.target_dir {
        sh.change_dir(target);
    }

    let current_dir = std::env::current_dir()?.canonicalize()?;

    let (system, custom_cmd_str) = if let Some(cmd_str) = cli.custom_command {
        (
            Box::new(systems::custom::CustomBuild {
                command: cmd_str.clone(),
            }) as Box<dyn systems::BuildSystem>,
            Some(cmd_str),
        )
    } else if cli.select_system {
        let systems = get_systems();
        let detected_indices: Vec<_> = systems
            .iter()
            .enumerate()
            .filter(|(_, s)| s.detect(&sh))
            .map(|(i, _)| i)
            .collect();
        let s = select_and_save_system(&systems, &detected_indices, &current_dir)?;
        (s, None)
    } else {
        match load_preference(&current_dir) {
            Some((name, cmd_opt)) => {
                if let (Some(cmd_str), false) = (cmd_opt.clone(), cli.select_command) {
                    info!("Using cached command: {} (use -S to change)", cmd_str);
                    (
                        Box::new(systems::custom::CustomBuild {
                            command: cmd_str.clone(),
                        }) as Box<dyn systems::BuildSystem>,
                        Some(cmd_str),
                    )
                } else {
                    // System selection or forced re-selection
                    let mut systems = get_systems();
                    if let Some(pos) = systems.iter().position(|s| s.name() == name) {
                        if !cli.select_command {
                            info!("Using cached system: {} (use -s to change)", name);
                        }
                        (systems.remove(pos), None)
                    } else {
                        let detected_indices: Vec<_> = systems
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| s.detect(&sh))
                            .map(|(i, _)| i)
                            .collect();

                        let s = select_and_save_system(&systems, &detected_indices, &current_dir)?;
                        (s, None)
                    }
                }
            }
            None => {
                let systems = get_systems();
                let detected_indices: Vec<_> = systems
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.detect(&sh))
                    .map(|(i, _)| i)
                    .collect();

                if detected_indices.is_empty() {
                    return Err(YbuildError::NoBuildSystem);
                }
                if detected_indices.len() == 1 {
                    save_preference(&current_dir, systems[detected_indices[0]].name(), None)?;
                    (get_systems().remove(detected_indices[0]), None)
                } else {
                    let s = select_and_save_system(&systems, &detected_indices, &current_dir)?;
                    (s, None)
                }
            }
        }
    };

    if let Some(ref cmd_str) = custom_cmd_str {
        save_preference(&current_dir, "Custom Command", Some(cmd_str))?;
    }

    if cli.watch {
        watch_and_run(&sh, &current_dir, system.as_ref(), &cli.options)
    } else {
        match system.execute(&sh, &cli.options)? {
            Some(cmd) => {
                save_preference(&current_dir, "Custom Command", Some(&cmd))?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}

fn select_and_save_system(
    systems: &[Box<dyn systems::BuildSystem>],
    detected_indices: &[usize],
    project_path: &Path,
) -> Result<Box<dyn systems::BuildSystem>> {
    let options: Vec<String> = detected_indices
        .iter()
        .map(|&i| systems[i].name().to_string())
        .collect();

    let selected_name =
        utils::select_command("Multiple build systems detected. Select one:", options)?;

    let idx = *detected_indices
        .iter()
        .find(|&&i| systems[i].name() == selected_name)
        .ok_or_else(|| YbuildError::Build("Selected system not found".to_string()))?;

    save_preference(project_path, systems[idx].name(), None)?;

    // We can't easily remove from the slice, so we re-fetch from get_systems() or
    // just return a new Box if we had clone_box.
    // Since we're refactoring for pragmatism, let's just get the system from a fresh list.
    let mut all_systems = get_systems();
    Ok(all_systems.remove(idx))
}

fn watch_and_run(
    sh: &Shell,
    current_dir: &Path,
    system: &dyn systems::BuildSystem,
    options: &systems::BuildOptions,
) -> Result<()> {
    use notify::{Config, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;

    info!("Watching for changes... (Ctrl-C to stop)");

    // Initial run
    match system.execute(sh, options) {
        Ok(Some(cmd)) => {
            let _ = save_preference(current_dir, "Custom Command", Some(&cmd));
        }
        Ok(None) => {}
        Err(e) => error!("Build failed: {}", e),
    }

    let (tx, rx) = channel();
    let mut watcher = notify::RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| YbuildError::Other(e.to_string()))?;

    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .map_err(|e| YbuildError::Other(e.to_string()))?;

    let mut last_run = std::time::Instant::now();
    let debounce = Duration::from_millis(500);

    for res in rx {
        match res {
            Ok(_) => {
                if last_run.elapsed() > debounce {
                    info!("Change detected, rerunning...");
                    if let Err(e) = system.execute(sh, options) {
                        error!("Build failed: {}", e);
                    }
                    last_run = std::time::Instant::now();
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn main() {
    logger::init();
    ctrlc::set_handler(|| {
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    if let Err(e) = run() {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
