#[cfg(test)]
mod tests;

use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub enum LoggerError {
    LoggerAlreadySet,
}

pub struct Logger {
    filters: Vec<(String, FilterType, log::LevelFilter)>,
    log_to_file: bool,
    default_level: log::LevelFilter,
    time_format: String,
}

#[derive(Clone, Copy)]
pub enum FilterType {
    Module,
    Crate,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            log_to_file: false,
            default_level: log::LevelFilter::Info,
            time_format: "%Y-%m-%d %H:%M:%S".into(),
        }
    }

    pub fn enable_logger(self) -> Result<(), LoggerError> {
        if INTERNAL_LOGGER.set(Arc::new(self)).is_err() {
            return Err(LoggerError::LoggerAlreadySet);
        }
        let logger = INTERNAL_LOGGER.get().unwrap();
        if log::set_logger(logger.as_ref() as &dyn log::Log).is_err() {
            Err(LoggerError::LoggerAlreadySet)
        } else {
            Ok(())
        }
    }

    fn add_filter(&mut self, module_name: &str, filter_type: FilterType, level: log::LevelFilter) {
        self.filters
            .push((module_name.to_owned(), filter_type, level.into()));
    }
}

fn filter(filter: &str, filter_type: FilterType, data: &str) -> bool {
    //crate_name::module::module::module:: ...
    let mut split = data.split("::");

    let crate_name = split.next().unwrap();

    let modules = split.collect::<Vec<_>>();

    match filter_type {
        FilterType::Module => modules.contains(&filter),
        FilterType::Crate => crate_name == filter,
    }
}

fn get_time(format: &str) -> String {
    let time = chrono::Local::now();
    format!("{}", time.format(format))
}

fn get_color(level: log::LevelFilter) -> &'static str {
    match level {
        log::LevelFilter::Off => "",
        log::LevelFilter::Error => "\x1b[31m",
        log::LevelFilter::Warn => "\x1b[33m",
        log::LevelFilter::Info => "\x1b[32m",
        log::LevelFilter::Debug => "\x1b[35m",
        log::LevelFilter::Trace => "\x1b[36m",
    }
}

fn format_level(level: log::LevelFilter) -> &'static str {
    match level {
        log::LevelFilter::Off => "",
        log::LevelFilter::Error => "ERROR",
        log::LevelFilter::Warn => "WARN ",
        log::LevelFilter::Info => "INFO ",
        log::LevelFilter::Debug => "DEBUG",
        log::LevelFilter::Trace => "TRACE",
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let msg = record.args();
        let metadata = record.metadata();
        let target = metadata.target();
        let msg_level = metadata.level().to_level_filter();

        let mut filtered = false;

        for (name, filter_type, level) in &self.filters {
            if filter(name, *filter_type, target) {
                //Test if the msg level msg is less severe than the filter level
                if msg_level > *level {
                    return;
                }
                filtered = true;
                break;
            }
        }

        //If not filtered and less severe than default level return
        if !filtered && msg_level > self.default_level {
            return;
        }

        //Passed all checks and can log stuff

        //Format:
        //[TIMESTAMP TARGET LEVEL] MESSAGE
        //

        let time = get_time(&self.time_format);
        let color = get_color(msg_level);
        let msg_level = format_level(msg_level);

        let output = format!(
            "\x1b[90m[\x1b[0m{time} {color}{msg_level} \x1b[0m{target}\x1b[90m]\x1b[0m {msg}"
        );

        println!("{output}");
    }

    fn flush(&self) {}
}

// struct Builder;

// impl Builder {
//     fn new() -> Self {
//         Self
//     }
// }

static INTERNAL_LOGGER: OnceLock<Arc<Logger>> = OnceLock::new();
