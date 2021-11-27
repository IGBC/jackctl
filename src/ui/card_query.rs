use gtk::prelude::*;
use gtk::{
    Application, Box, Button, CheckButton, Dialog, DialogFlags, Label, Orientation, ResponseType,
    Window,
};
use std::sync::Arc;

/// A dialog to ask the user about their sound card
pub struct CardQuery {
    inner: Dialog,
}

impl CardQuery {
    pub fn new(app: &Application, parent: &Window) -> Self {
        println!("creating card window");
        let inner = Dialog::with_buttons(
            Some("Use HDA Intel PCH?"),
            Some(parent),
            DialogFlags::all(),
            &[("Yes", ResponseType::Yes), ("No", ResponseType::No)],
        );
        //adding the window to the app allows it to not block the main window or other threads
        app.add_window(&inner);
        inner.set_modal(true);
        inner.set_default_response(ResponseType::Yes);
        let vbox = inner.get_content_area();
        vbox.set_orientation(Orientation::Vertical);
        vbox.set_margin_start(15);
        vbox.set_margin_end(15);
        vbox.set_margin_top(15);
        vbox.set_margin_bottom(5);
        vbox.set_spacing(5);

        let msg = Label::new(Some("Activate Sound Device HDA Intel PCH in JACK?"));
        vbox.pack_start(&msg, true, false, 0);

        let msg = Label::new(Some(
            "Activating this device will add it to the JACK connection graph \
        for use with other JACK clients. Only one sound system may use the device at a time so it \
        will become unavailable to non JACK applications",
        ));
        msg.set_line_wrap(true);

        vbox.pack_start(&msg, true, false, 0);

        let cb = CheckButton::with_label("Remember my choice for this device");
        vbox.pack_start(&cb, false, false, 0);

        inner.add(&vbox);

        inner.resize(250, 250);

        inner.show_all();

        Self { inner }
    }
}
