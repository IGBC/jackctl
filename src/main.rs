mod jack;
mod mixer;
mod model;
mod process_manager;
mod ui;

fn main() {
    {
        if gtk::init().is_err() {
            println!("Failed to start GTK, please ensure all dependancies are installed");
        }

        // ctrlc::set_handler(|| gtk::main_quit()).expect("Error setting Ctrl-C handler");

        let model = model::ModelInner::new();

        let proc_manager = process_manager::ProcessManager::new(model.clone());
        let jack_controller = jack::JackController::new(model.clone());
        let alsa_controller = mixer::MixerController::new(model.clone());
        let window = ui::init_ui(
            model.clone(),
            jack_controller.clone(),
            alsa_controller.clone(),
        );
        window.borrow().show();
        gtk::main();
    }

    println!("Jackctl Exiting, Goodbye");
}
