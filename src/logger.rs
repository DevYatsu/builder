use log::{Level, LevelFilter, Metadata, Record};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let (level_str, color_code) = match record.level() {
                Level::Error => ("ERROR", "\x1b[1;31m"), // Bold Red
                Level::Warn => ("WARN", "\x1b[1;33m"),   // Bold Yellow
                Level::Info => ("INFO", "\x1b[1;36m"),   // Bold Cyan
                _ => ("DEBUG", "\x1b[1;30m"),            // Bold Gray
            };

            let reset = "\x1b[0m";

            if record.level() <= Level::Warn {
                eprintln!("{}[{}]{} {}", color_code, level_str, reset, record.args());
            } else {
                println!("{}[{}]{} {}", color_code, level_str, reset, record.args());
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("failed to init logger");
}
