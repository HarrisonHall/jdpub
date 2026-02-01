//! Logging.

use super::*;

/// Initialize logging.
pub fn init(cli: &Cli) -> Result<()> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let level = if cfg!(debug_assertions) {
        LevelFilter::TRACE
    } else {
        if cli.verbose {
            LevelFilter::TRACE
        } else if cli.debug {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        }
    };
    let format_layer = fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact();
    let filter_layer = tracing_subscriber::filter::Targets::new()
        .with_default(LevelFilter::TRACE)
        .with_target("durf_parser", LevelFilter::WARN)
        .with_target("epub_builder", LevelFilter::WARN)
        .with_target("globset", LevelFilter::WARN)
        .with_target("html5ever", LevelFilter::WARN)
        .with_target("hyper", LevelFilter::WARN)
        .with_target("hyper_util", LevelFilter::WARN)
        .with_target("reqwest", LevelFilter::WARN)
        .with_target("scraper", LevelFilter::WARN)
        .with_target("jdpub", level);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();

    Ok(())
}
