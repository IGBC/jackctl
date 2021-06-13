mod ui;
mod jack;
mod model;
mod mixer;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to start GTK, please ensure all dependancies are installed");
    }

    let model = model::ModelInner::new();
    let jack_controller = jack::JackController::new(model.clone());
    let alsa_controller = mixer::MixerController::new(model.clone());
    let window = ui::init_ui(model.clone(), jack_controller.clone());
    window.borrow().show();
    gtk::main();
    println!("Jackctl Exiting, Goodbye");
}