//! Jackctl main entrypoint

#![allow(warnings)]

#[macro_use]
extern crate tracing;

mod cb_channel;
mod error;
mod log;
mod model;
mod rts;
mod ui;

use gio::prelude::*;
use model::{
    settings::{self, Settings},
    Model,
};
use std::{env::args, fs::File};

fn main() {
    log::parse_log_level();

    // Load and initialise settings first
    let dir = settings::scaffold();
    let set = Settings::init(dir.config_dir()).unwrap();

    let jack_if = rts::jack::JackRuntime::start(set.clone()).unwrap();
    let card_if = rts::hardware::HardwareHandle::new();
    let (_win, app, ui_if, _tray) = ui::create_ui(set.clone());

    Model::start(jack_if, ui_if, card_if, set);

    app.run(&args().collect::<Vec<_>>());

    info!("Jackctl Exiting, Goodbye");
}
