//! Jackctl main entrypoint

mod cb_channel;
mod error;
mod model2;
mod settings;
mod rts;

// mod jack;
// mod mixer;
// mod model;
// mod process_manager;
// mod ui;

// use gio::prelude::*;
// use std::env::args;

fn main() {
    // Load and initialise settings first
    let dir = settings::scaffold();
    let set = settings::Settings::init(dir.config_dir()).unwrap();

    println!("{:?}", set.r().app());

    let jack = rts::jack::JackRuntime::start().unwrap();
    let model = model2::Model::new(jack, set);
    
    // if gtk::init().is_err() {
    //     println!("Failed to start GTK, please ensure all dependancies are installed");
    // }

    // // due to a bug this button is basically panic on demand, however it does the job.
    // ctrlc::set_handler(|| gtk::main_quit()).expect("Error setting Ctrl-C handler");

    // let model = model::ModelInner::new();

    // let proc_manager = process_manager::ProcessManager::new(model.clone());
    // let jack_controller = jack::JackController::new(model.clone());
    // let _alsa_controller = mixer::MixerController::new(model.clone());
    // let (window, app) = ui::init_ui(model.clone(), jack_controller.clone());
    // window.borrow().show();

    // app.run(&args().collect::<Vec<_>>());
    // proc_manager.borrow_mut().end();

    println!("Jackctl Exiting, Goodbye");
}
