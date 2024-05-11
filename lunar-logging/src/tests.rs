use log::LevelFilter;

use super::*;

#[test]
fn test_logger() {
    let mut logger = Logger::new();
    logger.default_level = LevelFilter::Trace;
    logger.log_to_file = true;
    logger.enable_logger().unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    log::trace!("TEST");
    log::debug!("TEST");
    log::info!("TEST");
    log::warn!("TEST");
    log::error!("TEST");
}

#[test]

fn test_filter() {
    let target = "tests::something::something1::something2";
    assert!(filter("something", FilterType::Module, target));
    assert!(filter("tests", FilterType::Crate, target));

    assert!(!filter("something", FilterType::Crate, target));
    assert!(!filter("tests", FilterType::Module, target));
}
