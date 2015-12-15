use log::{Log, LogRecord, LogMetadata, MaxLogLevelFilter};

pub struct Logger {
    max_level_filter: MaxLogLevelFilter,
}

impl Logger {
    pub fn new(max_level_filter: MaxLogLevelFilter) -> Logger {
        Logger { max_level_filter: max_level_filter }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        let level = match self.max_level_filter.get().to_log_level() {
            None => return false,
            Some(level) => level,
        };
        metadata.level() <= level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}
