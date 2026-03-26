mod error;
mod logger;
mod systems;
mod utils;

use crate::error::{Result, YbuildError};
use log::info;
use std::path::{Path, PathBuf};
use systems::{BuildOptions, get_systems};
use xshell::Shell;

struct Cli {
    options: BuildOptions,
    target_dir: Option<PathBuf>,
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
        };

        let target_dir = args
            .opt_value_from_str::<_, String>(["-d", "--dir"])
            .map_err(|e| YbuildError::Other(e.to_string()))?
            .or(args
                .opt_free_from_str::<String>()
                .map_err(|e| YbuildError::Other(e.to_string()))?)
            .map(PathBuf::from);

        Ok(Self {
            options,
            target_dir,
        })
    }
}

fn print_help() {
    let cyan = "\x1b[1;36m";
    let reset = "\x1b[0m";
    println!("{}ybuild{} - A universal build utility\n", cyan, reset);
    println!("Usage: ybuild [OPTIONS] [DIRECTORY]\n");
    println!("Options:");
    println!("  -x, --run      Build and execute");
    println!("  -t, --test     Run project tests");
    println!("  -r, --release  Enable release optimizations");
    println!("  -l, --list     List all supported systems");
    println!("  -d, --dir <D>  Target directory");
    println!("  -h, --help     Show this help message\n");
}

fn get_config_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("ybuild");
        path.push("config.json");
        path
    })
}

fn load_preference(project_path: &Path) -> Option<String> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(config_path).ok()?;
    let json: tinyjson::JsonValue = content.parse().ok()?;
    let projects = json.get::<std::collections::HashMap<String, tinyjson::JsonValue>>()?;
    projects
        .get(project_path.to_str()?)?
        .get::<String>()
        .cloned()
}

fn save_preference(project_path: &Path, system_name: &str) -> Result<()> {
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

    projects.insert(
        project_path
            .to_str()
            .ok_or_else(|| YbuildError::Config("Invalid project path".to_string()))?
            .to_string(),
        system_name.to_string().into(),
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
    let systems = get_systems();
    let detected: Vec<_> = systems.iter().filter(|s| s.detect(&sh)).collect();

    if detected.is_empty() {
        return Err(YbuildError::NoBuildSystem);
    }

    let system = if detected.len() == 1 {
        detected[0]
    } else {
        match load_preference(&current_dir) {
            Some(name) => {
                if let Some(s) = detected.iter().find(|s| s.name() == name) {
                    *s
                } else {
                    select_and_save_system(&detected, &current_dir)?
                }
            }
            None => select_and_save_system(&detected, &current_dir)?,
        }
    };

    info!("using: {}", system.name());
    system.execute(&sh, &cli.options)
}

fn select_and_save_system<'a>(
    detected: &[&'a Box<dyn systems::BuildSystem>],
    project_path: &Path,
) -> Result<&'a Box<dyn systems::BuildSystem>> {
    let options: Vec<String> = detected.iter().map(|s| s.name().to_string()).collect();
    let selected_name =
        utils::select_command("Multiple build systems detected. Select one:", options)?;

    let system = detected
        .iter()
        .find(|s| s.name() == selected_name)
        .ok_or_else(|| YbuildError::Build("Selected system not found".to_string()))?;

    save_preference(project_path, system.name())?;
    Ok(*system)
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
