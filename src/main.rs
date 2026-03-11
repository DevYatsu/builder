use log::info;
use std::io::{self, Write};
use xshell::Shell;

mod error;
mod logger;
mod systems;

use error::{BuildError, Result};
use systems::{BuildOptions, get_systems};

fn print_help() {
    let cyan = "\x1b[1;36m";
    let reset = "\x1b[0m";
    println!("{}builder{} - A universal build utility\n", cyan, reset);
    println!("Usage: builder [OPTIONS] [DIRECTORY]\n");
    println!("Options:");
    println!("  -x, --run      Build and execute");
    println!("  -t, --test     Run project tests");
    println!("  -r, --release  Enable release optimizations");
    println!("  -l, --list     List all supported systems");
    println!("  -d, --dir <D>  Target directory");
    println!("  -h, --help     Show this help message\n");
}

fn main() {
    logger::init();

    if let Err(e) = run() {
        log::error!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        print_help();
        return Ok(());
    }

    if args.contains(["-l", "--list"]) {
        println!("Supported build systems:");
        for sys in get_systems() {
            println!("  - {}", sys.name());
        }
        return Ok(());
    }

    let options = BuildOptions {
        release: args.contains(["-r", "--release"]),
        run: args.contains(["-x", "--run"]),
        test: args.contains(["-t", "--test"]),
    };

    let target_dir: Option<String> = args
        .opt_value_from_str(["-d", "--dir"])?
        .or(args.opt_free_from_str()?);

    let sh = Shell::new()?;

    if let Some(d) = target_dir {
        let path = std::path::Path::new(&d);
        if !path.exists() {
            return Err(BuildError::NotFound(d));
        }
        sh.change_dir(path);
        info!("dir: {}", path.display());
    }

    let detected = get_systems()
        .into_iter()
        .filter(|s| s.detect(&sh))
        .collect::<Vec<_>>();

    if detected.is_empty() {
        return Err(BuildError::NoSystemFound);
    }

    let choice = if detected.len() == 1 {
        0
    } else {
        println!("\nMultiple build systems detected:");
        for (i, sys) in detected.iter().enumerate() {
            println!("  {}. {}", i + 1, sys.name());
        }
        loop {
            print!("\nChoice (1-{}): ", detected.len());
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if let Ok(val) = input.trim().parse::<usize>()
                && val >= 1
                && val <= detected.len()
            {
                break val - 1;
            }
            println!("Invalid selection, please try again.");
        }
    };

    let sys = &detected[choice];
    info!("using: {}", sys.name());

    let mode = if options.test {
        "testing"
    } else if options.run {
        "running"
    } else {
        "building"
    };
    info!("{}...", mode);

    sys.execute(&sh, &options).map_err(|e| {
        log::error!("{}: {}", mode, e);
        e
    })?;

    Ok(())
}
