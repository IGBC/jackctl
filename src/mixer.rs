use alsa::mixer::{Selem,SelemId,Mixer};
use alsa::card::Iter as CardIter;

use std::fmt;

pub struct MixerChannel {
    id: SelemId,
    has_switch: bool,
    has_volume: bool,
}

#[derive(Debug)]
pub struct Card {
    id: i32,
    name: String,
    mixer: Mixer,
    channels: Vec<MixerChannel>,
}

#[derive(Debug)]
pub struct MixerModel {
    cards: Vec<Card>
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

            cards.push(Card{
                id: card.get_index(),
                name: card.get_name().unwrap(),
                mixer,
                channels,
            })
        }
        Self {
            cards
        }
    }
}

impl Card {
    
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