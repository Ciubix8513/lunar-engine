#[cfg(test)]
mod tests;

use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, RwLock},
};

#[derive(Debug)]
pub enum LoggerError {
    LoggerAlreadySet,
    FileError(std::io::Error),
    InvalidFiname,
}

pub struct Logger {
    filters: Vec<(String, FilterType, log::LevelFilter)>,
    log_to_file: bool,
    log_filename: PathBuf,
    default_level: log::LevelFilter,
    time_format: String,
    log_file: Option<RwLock<std::fs::File>>,
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
            log_filename: generate_log_name(),
            default_level: log::LevelFilter::Info,
            time_format: "%Y-%m-%d %H:%M:%S".into(),
            log_file: None,
        }
    }

    pub fn enable_logger(self) -> Result<(), LoggerError> {
        let mut logger = self;

        if logger.log_to_file {
            create_file(&logger.log_filename);

            match std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&logger.log_filename)
            {
                Ok(f) => logger.log_file = Some(RwLock::new(f)),
                Err(e) => return Err(LoggerError::FileError(e)),
            }
        }

        if INTERNAL_LOGGER.set(Arc::new(logger)).is_err() {
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

    fn set_log_file_name(&mut self, filename: &Path) -> Result<(), LoggerError> {
        if filename.is_dir() {
            return Err(LoggerError::InvalidFiname);
        }
        self.log_filename = filename.to_owned();
        Ok(())
    }

    fn set_log_to_file(&mut self) {
        self.log_to_file = true;
    }

    fn set_timestamp_format(&mut self, format: &str) {
        self.time_format = format.to_owned();
    }
}

fn create_file(path: &Path) {
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    std::fs::File::create(path).unwrap();
}

fn generate_log_name() -> PathBuf {
    //ISO-8601 time
    let time = get_time("%Y-%m-%dT%H:%M:%S");
    //TODO Think about windows
    let user = std::env::vars().find(|i| i.0 == "USER").unwrap().1;

    format!("/home/{user}/.local/share/lunar-logging/log-{time}.log").into()
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
            "\x1b[90m[\x1b[0m{time} {color}{msg_level} \x1b[0m{target}\x1b[90m]\x1b[0m {msg}\n"
        );

        if let Some(f) = &self.log_file {
            f.write().unwrap().write(output.as_bytes()).unwrap();
        }

        print!("{output}");
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
