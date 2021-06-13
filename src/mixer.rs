use gtk::prelude::*;

use alsa::card::Iter as CardIter;
use alsa::mixer::{Mixer, Selem, SelemId};

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::model::Model;

pub struct MixerController {
    model: Model,
    old_alsa_model: MixerModel,
}

pub struct MixerChannel {
    id: SelemId,
    has_switch: bool,
    has_volume: bool,
}

#[derive(Clone, Debug)]
pub struct Card {
    id: i32,
    name: String,
    channels: Vec<MixerChannel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MixerModel {
    cards: Vec<Card>,
}

impl MixerController {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(Self {
            model,
            old_alsa_model: MixerModel::empty(),
        }));

        this.borrow_mut().update_model();
        let this_clone = this.clone();
        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().update_model();
            Continue(true)
        });

        this
    }

    fn update_model(&mut self) {
        let mut model = self.model.borrow_mut();
        let new_alsa_model = MixerModel::new();
        if new_alsa_model != self.old_alsa_model {
            model.update_mixer(&new_alsa_model);
            self.old_alsa_model = new_alsa_model;
        }
    }
}

impl MixerModel {
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for card in CardIter::new().map(|x| x.unwrap()) {
            let mixer = Mixer::new(&format!("hw:{}", card.get_index()), false).unwrap();
            let mut channels = Vec::new();

            for channel in mixer.iter() {
                let s = Selem::new(channel).unwrap();
                channels.push(MixerChannel {
                    id: s.get_id(),
                    has_switch: s.has_playback_switch() || s.has_capture_switch(),
                    has_volume: s.has_volume(),
                })
            }

            cards.push(Card {
                id: card.get_index(),
                name: card.get_name().unwrap(),
                channels,
            })
        }
        Self { cards }
    }

    pub fn empty() -> Self {
        Self { cards: Vec::new() }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Card> {
        self.cards.iter()
    }
}

impl Card {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Debug for MixerChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MixerChannel")
            .field("id", &(self.id.get_index(), self.id.get_name()))
            .field("has_switch", &self.has_switch)
            .field("has_volume", &self.has_volume)
            .finish()
    }
}

impl std::cmp::PartialEq for MixerChannel {
    fn eq(&self, other: &Self) -> bool {
        self.id.get_index() == other.id.get_index() && self.id.get_name() == other.id.get_name()
    }
}

impl std::clone::Clone for MixerChannel {
    fn clone(&self) -> Self {
        Self {
            has_switch: self.has_switch,
            has_volume: self.has_volume,
            id: SelemId::new(self.id.get_name().unwrap(), self.id.get_index()),
        }
    }
}

impl std::cmp::PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name && self.channels == other.channels
    }
}
