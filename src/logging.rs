fn parse_log_level(v: &str) -> log::LevelFilter {
    let upper = v.to_uppercase();
    match (&upper) as &str {
        //Debug
        "TRACE" | "-1" => log::LevelFilter::Trace,
        "DEBUG" | "0" => log::LevelFilter::Debug,
        "INFO" | "1" => log::LevelFilter::Info,
        "WARM" | "2" => log::LevelFilter::Warn,
        "ERROR" | "3" => log::LevelFilter::Error,
        "OFF" | "4" => log::LevelFilter::Off,
        _ => log::LevelFilter::Info,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn initialize_logging() {
    let vars = std::env::vars();

    let mut log_level = log::LevelFilter::Info;
    let mut engine_log_level = log::LevelFilter::Info;
    let mut wgpu_log_level = log::LevelFilter::Warn;

    let mut log_to_file = false;

    for (name, value) in vars {
        match (&name.to_uppercase()) as &str {
            "LOG_LEVEL" => log_level = parse_log_level(&value),
            "ENGINE_LOG_LEVEL" => engine_log_level = parse_log_level(&value),
            "GENERATE_LOGS" => log_to_file = true,
            "WGPU_LOG_LEVEL" => wgpu_log_level = parse_log_level(&value),
            _ => {}
        }
    }

    let mut b = lunar_logger::Builder::new()
        .add_crate_filter("wgpu", wgpu_log_level)
        .add_crate_filter("wgpu_hal", wgpu_log_level)
        .add_crate_filter("lunar_engine", engine_log_level)
        .default_filter(log_level);
    if log_to_file {
        b = b.log_to_file()
    }
    b.init().unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn initialize_logging() {
    wasm_logger::init(wasm_logger::Config::default());
}
