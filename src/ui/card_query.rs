use crate::{
    model::{card::Card, events::UiEvent},
    ui::{utils, window::CardQuestionaire, Questionaire, UiRuntime},
};
use atomptr::AtomPtr;
use gtk::prelude::*;
use gtk::{
    Application, Box, Button, CheckButton, Dialog, DialogFlags, Label, Orientation, ResponseType,
    Window,
};
use std::sync::Arc;

/// A dialog to ask the user about their sound card
pub struct CardQuery {
    inner: Dialog,
    label1: Label,
    label2: Label,
    check: CheckButton,
    rt: UiRuntime,
    q: Questionaire<Card>,
    card: AtomPtr<Option<Card>>,
}

impl CardQuery {
    pub(super) fn new(
        arc: CardQuestionaire,
        rt: UiRuntime,
        app: &Application,
        parent: &Window,
    ) -> Questionaire<Card> {
        let q = Questionaire::new();

        // Create the basic Dialog
        let (inner, label1, label2, check) = utils::yes_no_dialog(app, parent);
        inner.show_all();

        let this = Arc::new(Self {
            inner,
            label1,
            label2,
            check,
            rt,
            q: q.clone(),
            card: AtomPtr::new(None),
        });

        let this2 = Arc::clone(&this);
        this.inner.connect_response(move |_, resp| {
            let (store, usage) = match resp {
                ResponseType::Yes => (this2.check.get_active(), true),
                ResponseType::No => (this2.check.get_active(), false),
                _ => (false, false),
            };

            let r = this2.card.swap(None).consume();
            let card = Arc::try_unwrap(r).unwrap().unwrap();
            this2
                .rt
                .sender()
                .send(UiEvent::CardUsage { card, usage, store });
        });

        // Create a glib event to check for updates
        glib::timeout_add_local(200, move || {
            if this.card.get_ref().is_some() {
                return Continue(true);
            }

            match this.q.try_recv() {
                Some(card) => {
                    // Store the element
                    let l1 = format!("Activate sound device '{}'?", card.name);
                    this.label1.set_text(l1.as_str());
                    this.inner.set_title(l1.as_str());
                    this.card.swap(Some(card));
                    Continue(true)
                }
                None => {
                    // This means we should clean-up
                    trace!("No more card questions left...");
                    arc.swap(None);
                    this.inner.hide();
                    Continue(false)
                }
            }
        });

        q
    }
}
