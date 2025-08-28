use crate::emapi;

struct EmLogger;

impl log::Log for EmLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let target = match record.level() {
                log::Level::Error => emapi::emscripten::LogTarget::ConsoleError,
                log::Level::Warn => emapi::emscripten::LogTarget::ConsoleWarn,
                log::Level::Info => emapi::emscripten::LogTarget::ConsoleInfo,
                log::Level::Debug => emapi::emscripten::LogTarget::ConsoleDebug,
                log::Level::Trace => emapi::emscripten::LogTarget::ConsoleDebug,
            };
            emapi::emscripten::log(target, &format!("{}", record.args()));
        }
    }

    fn flush(&self) {}
}

/// Dispatch info!() etc. to [emscripten::log()].
pub fn setup_logger(level: log::LevelFilter) {
    static LOGGER: EmLogger = EmLogger;

    log::set_logger(&LOGGER).expect("set_logger failed");
    log::set_max_level(level);
}
