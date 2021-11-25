//! Jackctl main entrypoint

mod cb_channel;
mod error;
mod model;
mod rts;
mod ui;

use gio::prelude::*;
use model::{
    settings::{self, Settings},
    Model,
};
use std::env::args;

fn main() {
    // Load and initialise settings first
    let dir = settings::scaffold();
    let set = Settings::init(dir.config_dir()).unwrap();

    println!("Test log");

    let jack_if = rts::jack::JackRuntime::start(set.clone()).unwrap();
    let card_if = rts::hardware::HardwareHandle::new();
    let (_win, app, ui_if) = ui::create_ui(set.clone());

    Model::start(jack_if, ui_if, card_if, set);

    app.run(&args().collect::<Vec<_>>());

    println!("Jackctl Exiting, Goodbye");
}
