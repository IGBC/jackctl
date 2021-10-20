use crate::{
    jack::JackController,
    model::PortGroup,
    ui::{utils, Pages},
};
use glib::SignalHandlerId;
use gtk::CheckButton;
use std::{cell::RefCell, rc::Rc};

pub struct MidiMatrix {
    inner: Vec<(u32, u32, CheckButton, SignalHandlerId)>,
}

impl MidiMatrix {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn update(
        &mut self,
        pages: &mut Pages,
        jack: &Rc<RefCell<JackController>>,
        inputs: &PortGroup,
        outputs: &PortGroup,
    ) {
        let (grid, callbacks) = utils::generate_grid(jack, inputs, outputs);

        pages.remove_page("MIDI");
        pages.insert_scrolled("MIDI", &grid);
        self.inner = callbacks;
    }

    pub fn iter(&self) -> &Vec<(u32, u32, CheckButton, SignalHandlerId)> {
        &self.inner
    }
}
