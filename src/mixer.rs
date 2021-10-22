use crate::model::{Card, CardConfig, CardStatus, Event, MixerChannel, Model};
use crate::model2::card::{CardId, ChannelCount, ChannelId, SampleRate, Volume};
use alsa::card::Iter as CardIter;
use alsa::mixer::{Mixer, Selem, SelemChannelId};
use alsa::pcm::{HwParams, PCM};
use alsa::Direction;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MixerController {
    model: Model,
    //locks: HashMap<CardId, CardLock>,
}

const SAMPLE_RATES: [u32; 20] = [
    8000,   // Telephone Audio
    11025,  // 1/4 CD Audio (Low Quality MPEG)
    16000,  // 2x Telephone, VoIP
    22050,  // 1/2 CD Audio (Common in Cheap USB audio)
    32000,  // Mini DV / DAT / NICAM digital Audio
    37286,  // Linux snd-pcsp (don't ask)
    37800,  // CD-ROM XA Audio
    44056,  // Digtal Audio locked to NTSC Video (44.1k/1.001) (EIAJ Spec)
    44100,  // CD Audio
    47250,  // Early PCM Recorders
    48000,  // Mini DV / DAT / DVD
    50000,  // Early PCM Recorders
    50400,  // Early Mitsubishi PCM Recorders
    64000,  // Uncommon - Included for compatibility
    88200,  // CD Audio 2x oversampling
    96000,  // DvD Audio 2x oversampling
    176400, // CD Audio 4x oversampling (Also HDCD)
    192000, // DVD Audio 4x oversampling (and most sound cards on PC, and Bluray/HD DVD)
    352800, // CD 8x (DXD & SACD)
    384000, // DVD 8x (have never ever seen anything enumerate this fast)
];

impl MixerController {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(Self {
            model,
            //locks: HashMap::new(),
        }));

        let this_clone = this.clone();
        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().update();
            Continue(true)
        });

        //card_lock::CardLock::create_server();

        this
    }

    fn update(&mut self) {
        let card_ids: Vec<CardId> = self
            .model
            .lock()
            .unwrap()
            .cards
            .keys()
            .map(|x| *x)
            .collect();
        // first check for new cards
        for alsa_card in CardIter::new().map(|x| x.unwrap()) {
            if !card_ids.contains(&&alsa_card.get_index()) {
                self.model
                    .lock()
                    .unwrap()
                    .get_pipe()
                    .send(Event::AddCard(
                        alsa_card.get_index(),
                        alsa_card.get_name().unwrap(),
                    ))
                    .unwrap();
            }
        }

        let model = self.model.lock().unwrap();
        let keys: Vec<&Card> = model.cards.values().collect();
        for card in keys.iter() {
            // todo map this into a proper match statement
            match card.state {
                CardStatus::Enum => {
                    let inputs = match self.attempt_capture_enumerate(card.id) {
                        Ok((rates, channels)) => {
                            let rate = self.pick_best_rate(&rates);
                            Some(CardConfig {
                                sample_rate: rate,
                                channels,
                            })
                        }
                        _ => None,
                    };

                    let outputs = match self.attempt_playback_enumerate(card.id) {
                        Ok((rates, channels)) => {
                            let rate = self.pick_best_rate(&rates);
                            Some(CardConfig {
                                sample_rate: rate,
                                channels,
                            })
                        }
                        _ => None,
                    };

                    if inputs.is_some() || outputs.is_some() {
                        // this is the old mixer enumeration code, but we're only running it once.
                        // pray that cards do not dynamically change their mixer interfaces.
                        let mixer = Mixer::new(&format!("hw:{}", card.id), false).unwrap();

                        let mut channels: Vec<MixerChannel> = Vec::new();

                        for (id, channel) in mixer.iter().enumerate() {
                            let id = id as u32;
                            let s = Selem::new(channel).unwrap();

                            let name = s.get_id().get_name().unwrap().to_string();
                            println!("Card {}, id {}, name: {}", card.id, id, name);

                            if s.has_capture_volume() {
                                let (volume_min, volume_max) = s.get_capture_volume_range();
                                let mc = MixerChannel::new(
                                    id,
                                    name,
                                    false,
                                    s.has_capture_switch(),
                                    volume_min,
                                    volume_max,
                                );
                                channels.push(mc);
                            } else {
                                if s.has_playback_volume() {
                                    let (volume_min, volume_max) = s.get_playback_volume_range();
                                    let mc = MixerChannel::new(
                                        id,
                                        name,
                                        true,
                                        s.has_playback_switch(),
                                        volume_min,
                                        volume_max,
                                    );
                                    channels.push(mc);
                                }
                            };
                        }

                        model
                            .get_pipe()
                            .send(Event::FinishEnumerateCard(
                                card.id, inputs, outputs, channels,
                            ))
                            .unwrap();
                    } else {
                        println!("Failed to enumerate card {} - {}", card.id, card.name());
                        model
                            .get_pipe()
                            .send(Event::FailEnumerateCard(card.id))
                            .unwrap();
                    }
                }
                CardStatus::Active => match Mixer::new(&format!("hw:{}", card.id), false) {
                    Ok(mixer) => {
                        for (id, elem) in mixer.iter().enumerate() {
                            let selem = Selem::new(elem).unwrap();
                            match card.channels.get(&(id as u32)) {
                                Some(channel) => {
                                    if channel.dirty {
                                        if channel.has_switch {
                                            Self::set_muting(
                                                channel.is_playback,
                                                &selem,
                                                channel.switch,
                                            );
                                        }
                                        Self::set_volume(
                                            channel.is_playback,
                                            &selem,
                                            channel.volume,
                                        );
                                        model
                                            .get_pipe()
                                            .send(Event::CleanChannel(card.id, channel.id))
                                            .unwrap();
                                    } else {
                                        let sw = if channel.has_switch {
                                            Self::get_muting(channel.is_playback, &selem)
                                        } else {
                                            false
                                        };
                                        let volume = Self::get_volume(channel.is_playback, &selem);
                                        model
                                            .get_pipe()
                                            .send(Event::UpdateChannel(
                                                card.id, channel.id, volume, sw,
                                            ))
                                            .unwrap();
                                    }
                                }
                                None => (),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Could not get mixer for card {}: {}", card.id, e);
                        model.get_pipe().send(Event::StopCard(card.id)).unwrap();
                    }
                },
                _ => {
                    // Card is in a state that mixer doesn't need to worry about
                }
            }
        }
    }

    fn attempt_playback_enumerate(
        &self,
        card: CardId,
    ) -> alsa::Result<(Vec<SampleRate>, ChannelCount)> {
        // Open playback device
        let mut rates = Vec::new();
        let pcm = PCM::new(&format!("hw:{}", card), Direction::Playback, false)?;
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_rate_resample(false).unwrap();
        for rate in SAMPLE_RATES.iter() {
            match hwp.test_rate(*rate) {
                Ok(()) => rates.push(*rate),
                Err(_) => (),
            }
        }

        let channels = hwp.get_channels_max().unwrap();
        Ok((rates, channels))
    }

    fn attempt_capture_enumerate(
        &self,
        card: CardId,
    ) -> alsa::Result<(Vec<SampleRate>, ChannelCount)> {
        // Open capture device
        let mut rates = Vec::new();
        let pcm = PCM::new(&format!("hw:{}", card), Direction::Capture, false)?;
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_rate_resample(false).unwrap();
        for rate in SAMPLE_RATES.iter() {
            match hwp.test_rate(*rate) {
                Ok(()) => rates.push(*rate),
                Err(_) => (),
            }
        }

        let channels = hwp.get_channels_max().unwrap();
        Ok((rates, channels))
    }

    fn pick_best_rate(&self, rates: &Vec<SampleRate>) -> SampleRate {
        if rates.contains(&48000) {
            48000
        } else if rates.contains(&44100) {
            44100
        } else {
            *rates.last().unwrap()
        }
    }

    pub fn get_volume(playback: bool, channel: &Selem) -> Volume {
        if playback {
            channel
                .get_playback_volume(SelemChannelId::FrontLeft)
                .unwrap()
        } else {
            channel
                .get_capture_volume(SelemChannelId::FrontLeft)
                .unwrap()
        }
    }

    pub fn get_muting(playback: bool, channel: &Selem) -> bool {
        let val = if playback {
            channel
                .get_playback_switch(SelemChannelId::FrontLeft)
                .unwrap()
        } else {
            channel
                .get_capture_switch(SelemChannelId::FrontLeft)
                .unwrap()
        };
        val == 0
    }

    pub fn set_volume(playback: bool, channel: &Selem, volume: Volume) {
        if playback {
            channel.set_playback_volume_all(volume).unwrap();
        } else {
            channel
                .set_capture_volume(SelemChannelId::FrontLeft, volume)
                .unwrap();
            channel
                .set_capture_volume(SelemChannelId::FrontRight, volume)
                .unwrap();
        }
    }

    pub fn set_muting(playback: bool, channel: &Selem, mute: bool) {
        let value = match mute {
            true => 0,
            false => 1,
        };
        if playback {
            channel.set_playback_switch_all(value).unwrap()
        } else {
            channel.set_capture_switch_all(value).unwrap()
        }
    }
}
