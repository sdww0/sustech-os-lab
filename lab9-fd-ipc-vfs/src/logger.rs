use log::{Metadata, Record};
use ostd::early_println;
use owo_colors::Style;

struct ColorLogger;

static LOGGER: ColorLogger = ColorLogger;

impl log::Log for ColorLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let record_style = Style::new().default_color();
        let level_style = match record.level() {
            log::Level::Error => Style::new().red(),
            log::Level::Warn => Style::new().bright_yellow(),
            log::Level::Info => Style::new().blue(),
            log::Level::Debug => Style::new().bright_green(),
            log::Level::Trace => Style::new().bright_black(),
        };

        early_println!(
            "{} {}",
            level_style.style(format_args!("[{:<5}]", record.level())),
            record_style.style(record.args())
        );
    }

    fn flush(&self) {}
}

pub(super) fn init() {
    ostd::logger::inject_logger(&LOGGER);
}
