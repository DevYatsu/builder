use log::{error, info};
use std::io::{self, Write};
use xshell::Shell;

mod logger;
mod systems;
use systems::{BuildOptions, get_systems};

fn print_help() {
    println!("Usage: builder [OPTIONS] [DIRECTORY]\n");
    println!("Options:");
    println!("  -x, --run      Native run");
    println!("  -t, --test     Run tests");
    println!("  -r, --release  Release build");
    println!("  -l, --list     List supported build systems");
    println!("  -d, --dir <D>  Target directory");
    println!("  -h, --help     Show this help\n");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init();

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
        if !sh.path_exists(&d) {
            error!("dir '{}' not found", d);
            std::process::exit(1);
        }
        sh.change_dir(&d);
        info!("dir: {}", d);
    }

    let detected = get_systems()
        .into_iter()
        .filter(|s| s.detect())
        .collect::<Vec<_>>();

    let choice = if detected.is_empty() {
        error!("no build system found.");
        std::process::exit(1);
    } else if detected.len() == 1 {
        0
    } else {
        println!("\nMultiple systems found:");
        for (i, sys) in detected.iter().enumerate() {
            println!("  {}. {}", i + 1, sys.name());
        }
        print!("Choice (1-{}): ", detected.len());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().parse::<usize>().map(|v| v - 1).unwrap_or(999)
    };

    if let Some(sys) = detected.get(choice) {
        info!("detected: {}", sys.name());
        let mode = if options.test {
            "test"
        } else if options.run {
            "runn"
        } else {
            "build"
        };
        info!("{}ing...", mode);

        sys.execute(&sh, &options);
    } else {
        error!("invalid selection.");
        std::process::exit(1);
    }

    Ok(())
}
