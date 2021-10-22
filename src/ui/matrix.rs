\use crate::{
    jack::JackController,
    model::PortGroup,
    ui::{utils, Pages},
};
use glib::signal::SignalHandlerId;
use gtk::CheckButton;
use std::{cell::RefCell, rc::Rc};

pub struct AudioMatrix {
    inner: Vec<(u32, u32, CheckButton, SignalHandlerId)>,
}

impl AudioMatrix {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    /// This function updates the matrix based on the current model
    pub fn update(
        &mut self,
        pages: &mut Pages,
        jack: &Rc<RefCell<JackController>>,
        inputs: &PortGroup,
        outputs: &PortGroup,
    ) {
        let (grid, callbacks) = utils::generate_grid(jack, inputs, outputs);
        pages.remove_page("Matrix");
        pages.insert_scrolled("Matrix", &grid);
        self.inner = callbacks;
    }

    pub fn iter(&self) -> &Vec<(u32, u32, CheckButton, SignalHandlerId)> {
        &self.inner
    }
}
