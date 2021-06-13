mod ui;
mod engine;
mod model;
mod mixer;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to start GTK, please ensure all dependancies are installed");
    }

    let model = model::ModelInner::new();
    let controller = engine::Controller::new(model.clone());
    let window = ui::init_ui(model.clone(),controller.clone());
    window.borrow().show();
    gtk::main();
    println!("Jackctl Exiting, Goodbye");
}