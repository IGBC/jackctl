use crate::ui::window::MainWindow;
use gtk::prelude::*;
use gtk::{Application, Box, Button, CheckButton, Label, Orientation, Window, WindowType};
use std::sync::Arc;

/// A dialog to ask the user about their sound card
pub struct CardQuery {
    inner: Window,
}

impl CardQuery {
    pub fn new(app: &Application) -> Self {
        let inner = Window::new(WindowType::Popup);
        inner.set_application(Some(app));
        let vbox = Box::new(Orientation::Vertical, 0);
        let msg = Label::new(Some("This is a message"));
        vbox.pack_start(&msg, true, false, 0);

        let hbox = Box::new(Orientation::Horizontal, 0);
        vbox.pack_start(&hbox, false, false, 0);
        let nbutton = Button::with_label("No");
        hbox.pack_end(&nbutton, false, false, 0);
        let ybutton = Button::with_label("Yes");
        hbox.pack_end(&ybutton, false, false, 0);

        let cb = CheckButton::with_label("Remember this choice");
        vbox.pack_start(&cb, false, false, 0);

        inner.show_all();
        Self { inner }
    }
}
