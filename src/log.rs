//! Logging utilities

use std::{fs::OpenOptions, io::Write};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, EnvFilter};

pub(crate) fn parse_log_level() {
    let filter = EnvFilter::try_from_env("JACKCTL_LOG")
        .unwrap_or_default()
        .add_directive(LevelFilter::INFO.into())
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("async_io=error".parse().unwrap())
        .add_directive("polling=error".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    fmt()
        .with_env_filter(filter)
        // .json()
        // .with_writer(|| {
        //     OpenOptions::new()
        //         .create(true)
        //         .write(true)
        //         .open("log.json")
        //         .unwrap()
        // })
        .init();
    info!("Initialised logger: welcome to jackctl!");
}

/// Create an oops (a fatal crash) with an associated error message
pub(crate) fn oops<S: Into<String>>(msg: S, code: u16) -> ! {
    error!("{}", msg.into());
    std::process::exit(code.into());
}
