//! Jackctl main entrypoint

mod cb_channel;
mod error;
mod model2;
mod rts;
mod settings;
mod ui;

fn main() {
    // Load and initialise settings first
    let dir = settings::scaffold();
    let set = settings::Settings::init(dir.config_dir()).unwrap();

    let jack_if = rts::jack::JackRuntime::start(set.clone()).unwrap();
    let card_if = rts::hardware::HardwareHandle::new();
    let (_tx, ui_if) = ui::create_ui();

    model2::Model::start(jack_if, ui_if, card_if, set);

    if gtk::init().is_err() {
        println!("Failed to start GTK, please ensure all dependancies are installed");
    }

    // due to a bug this button is basically panic on demand, however it does the job.
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
