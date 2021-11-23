use crate::ui::window::MainWindow;
use gtk::Window;
use std::sync::Arc;

/// A dialog to ask the user about their sound card
pub struct CardQuery {
    inner: Window,
    parent: Arc<MainWindow>,
}
