//! Jackctl main entrypoint

mod cb_channel;
mod error;
mod model2;
mod rts;
mod settings;
mod ui;

use gio::prelude::*;
use model2::Model;
use std::env::args;

fn main() {
    // Load and initialise settings first
    let dir = settings::scaffold();
    let set = settings::Settings::init(dir.config_dir()).unwrap();

    let jack_if = rts::jack::JackRuntime::start(set.clone()).unwrap();
    let card_if = rts::hardware::HardwareHandle::new();
    let (win, app, ui_if) = ui::create_ui();

    Model::start(jack_if, ui_if, card_if, set);

    win.show();
    app.run(&args().collect::<Vec<_>>());

    println!("Jackctl Exiting, Goodbye");
}
