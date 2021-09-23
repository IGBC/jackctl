use gtk::prelude::*;

use alsa::card::Iter as CardIter;
use alsa::mixer::{Mixer, Selem, SelemChannelId};

use alsa::pcm::{HwParams, PCM};
use alsa::Direction;

use std::cell::RefCell;
use std::rc::Rc;

use crate::model::{CardStatus, Model, Event};

pub struct MixerController {
    model: Model,
}

const SAMPLE_RATES: [u32; 19] = [
    8000, 11025, 16000, 22050, 32000, 37800, 44056, 44100, 47250, 48000, 50000, 50400, 64000,
    88200, 96000, 176400, 192000, 352800, 384000,
];

pub type CardId = i32;
pub type ChannelId = u32;
pub type Volume = i64;
pub type SampleRate = u32;

impl MixerController {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(Self { model }));

        let this_clone = this.clone();
        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().update();
            Continue(true)
        });

        this
    }

    fn update(&mut self) {
        let card_ids: Vec<CardId> = self.model.lock().unwrap().cards.keys().map(|x| *x).collect();
        // first check for new cards
        for alsa_card in CardIter::new().map(|x| x.unwrap()) {
            if !card_ids.contains(&&alsa_card.get_index()) {
                self.model.lock().unwrap().update(Event::AddCard(alsa_card.get_index(), alsa_card.get_name().unwrap()));
            }
        }

        for card in self.model.lock().unwrap().cards.values_mut() {
            // todo map this into a proper match statement
            match card.state {
                CardStatus::Unknown => {
                    match self.attempt_capture_enumerate(card.id) {
                        Ok(rates) => {
                            let rate = self.pick_best_rate(&rates);
                            card.inputs = Some(rate);
                        }
                        _ => (),
                    }

                    match self.attempt_playback_enumerate(card.id) {
                        Ok(rates) => {
                            let rate = self.pick_best_rate(&rates);
                            card.outputs = Some(rate);
                        }
                        _ => (),
                    }

                    if card.inputs.is_some() || card.outputs.is_some() {
                        // this is the old mixer enumeration code, but we're only running it once.
                        // pray that cards do not dynamically change their mixer interfaces.
                        let mixer = Mixer::new(&format!("hw:{}", card.id), false).unwrap();

                        for (id, channel) in mixer.iter().enumerate() {
                            let id = id as u32;
                            let s = Selem::new(channel).unwrap();

                            let name = s.get_id().get_name().unwrap().to_string();
                            println!("Card {}, id {}, name: {}", card.id, id, name);

                            if s.has_capture_volume() {
                                let (volume_min, volume_max) = s.get_capture_volume_range();
                                card.add_channel(
                                    id,
                                    name,
                                    false,
                                    s.has_capture_switch(),
                                    volume_min,
                                    volume_max,
                                );
                            } else {
                                if s.has_playback_volume() {
                                    let (volume_min, volume_max) = s.get_playback_volume_range();
                                    card.add_channel(
                                        id,
                                        name,
                                        true,
                                        s.has_playback_switch(),
                                        volume_min,
                                        volume_max,
                                    );
                                }
                            };
                        }

                        card.state = CardStatus::Active;
                    } else {
                        println!("Failed to enumerate card {} - {}", card.id, card.name());
                        card.state = CardStatus::EnumFailed;
                    }
                }
                CardStatus::Active => {
                    let mixer = Mixer::new(&format!("hw:{}", card.id), false).unwrap();

                    for (id, elem) in mixer.iter().enumerate() {
                        let selem = Selem::new(elem).unwrap();
                        match card.channels.get_mut(&(id as u32)) {
                            Some(channel) => {
                                if channel.dirty {
                                    if channel.has_switch {
                                        Self::set_muting(
                                            channel.is_playback,
                                            &selem,
                                            channel.switch,
                                        );
                                    }
                                    Self::set_volume(channel.is_playback, &selem, channel.volume);
                                    channel.dirty = false;
                                } else {
                                    if channel.has_switch {
                                        channel.switch =
                                            Self::get_muting(channel.is_playback, &selem);
                                    }
                                    channel.volume = Self::get_volume(channel.is_playback, &selem);
                                }
                            }
                            None => (),
                        }
                    }
                }
                CardStatus::EnumFailed => {
                    // we ignore this card
                }
                _ => {
                    panic!("unexpected card SM");
                }
            }
        }
    }

    fn attempt_playback_enumerate(&self, card: CardId) -> alsa::Result<Vec<SampleRate>> {
        // Open playback device
        let mut results = Vec::new();
        let pcm = PCM::new(&format!("hw:{}", card), Direction::Playback, false)?;
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_rate_resample(false).unwrap();
        for rate in SAMPLE_RATES.iter() {
            match hwp.test_rate(*rate) {
                Ok(()) => results.push(*rate),
                Err(_) => (),
            }
        }
        Ok(results)
    }

    fn attempt_capture_enumerate(&self, card: CardId) -> alsa::Result<Vec<SampleRate>> {
        // Open capture device
        let mut results = Vec::new();
        let pcm = PCM::new(&format!("hw:{}", card), Direction::Capture, false)?;
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_rate_resample(false).unwrap();
        for rate in SAMPLE_RATES.iter() {
            match hwp.test_rate(*rate) {
                Ok(()) => results.push(*rate),
                Err(_) => (),
            }
        }
        Ok(results)
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

// impl Card {
//     pub fn new(id: i32, name: String) -> Option<Self> {
//         let mut playback_rates:Vec<u32> = Vec::new();
//         let mut capture_rates:Vec<u32> = Vec::new();

//         match PCM::new(&format!("hw:{}", id), Direction::Playback, false) {
//         //match PCM::new("default", Direction::Playback, false) {
//             Ok(pcm) => {
//                 // Set hardware parameters: 44100 Hz / Mono / 16 bit
//                 let hwp = HwParams::any(&pcm).unwrap();
//                 println!("    Playback channels: {}, {}", hwp.get_channels_min().unwrap(),
//                 hwp.get_channels_max().unwrap());
//                 //,         hwp.get_channels().unwrap());
//                 hwp.set_rate_resample(false).unwrap();
//                 //for rate in SAMPLE_RATES.iter() {
//                 for rate in SAMPLE_RATES.iter() {
//                     match hwp.test_rate(*rate) {
//                         Ok(()) => {
//                             println!("        {}: Ok", rate);
//                             playback_rates.push(*rate);
//                         },
//                         Err(_) => (),
//                     };
//                 }
//             },
//             Err(e) => {
//                 println!("   Playback - cannot open card: {}", e);
//             }
//         }

//         match PCM::new(&format!("hw:{}", id), Direction::Capture, false) {
//         //match PCM::new("default", Direction::Playback, false) {
//             Ok(pcm) => {
//                 // Set hardware parameters: 44100 Hz / Mono / 16 bit
//                 let hwp = HwParams::any(&pcm).unwrap();
//                 println!("    Capture channels: {}, {}", hwp.get_channels_min().unwrap(),
//                 hwp.get_channels_max().unwrap());
//                 //,         hwp.get_channels().unwrap());
//                 hwp.set_rate_resample(false).unwrap();
//                 //for rate in SAMPLE_RATES.iter() {
//                 for rate in SAMPLE_RATES.iter() {
//                     match hwp.test_rate(*rate) {
//                         Ok(()) => {
//                             println!("        {}: Ok", rate);
//                             capture_rates.push(*rate);
//                         },
//                         Err(_) => (),
//                     };
//                 }
//             },
//             Err(e) => {
//                 println!("   Capture - cannot open card: {}", e);
//             }
//         }


// for a in ::alsa::card::Iter::new().map(|x| x.unwrap()) {
        //     // Open default playback device
        //     //&format!("hw:{}", a.get_index());
        //     println!("hw:{} {}", a.get_index(), a.get_name().unwrap());
        //     match PCM::new(&format!("hw:{}", a.get_index()), Direction::Playback, false) {
        //         //match PCM::new("default", Direction::Playback, false) {
        //         Ok(pcm) => {
        //             // Set hardware parameters: 44100 Hz / Mono / 16 bit
        //             let hwp = HwParams::any(&pcm).unwrap();
        //             println!(
        //                 "    Playback channels: {}, {}",
        //                 hwp.get_channels_min().unwrap(),
        //                 hwp.get_channels_max().unwrap()
        //             );
        //             //,         hwp.get_channels().unwrap());
        //             hwp.set_rate_resample(true).unwrap();
        //             //for rate in SAMPLE_RATES.iter() {
        //             for rate in SAMPLE_RATES.iter() {
        //                 match hwp.test_rate(*rate) {
        //                     Ok(()) => println!("        {}: Ok", rate),
        //                     Err(_) => (),
        //                 };
        //             }
        //         }
        //         Err(e) => {
        //             println!("   Playback - cannot open card: {}", e);
        //         }
        //     }

        //     match PCM::new(&format!("hw:{}", a.get_index()), Direction::Capture, false) {
        //         //match PCM::new("default", Direction::Playback, false) {
        //         Ok(pcm) => {
        //             // Set hardware parameters: 44100 Hz / Mono / 16 bit
        //             let hwp = HwParams::any(&pcm).unwrap();
        //             println!(
        //                 "    Capture channels: {}, {}",
        //                 hwp.get_channels_min().unwrap(),
        //                 hwp.get_channels_max().unwrap()
        //             );
        //             //,         hwp.get_channels().unwrap());
        //             hwp.set_rate_resample(true).unwrap();
        //             //for rate in SAMPLE_RATES.iter() {
        //             for rate in 1..40000000 {
        //                 match hwp.test_rate(rate) {
        //                     Ok(()) => println!("        {}: Ok", rate),
        //                     Err(_) => (),
        //                 };
        //             }
        //         }
        //         Err(e) => {
        //             println!("   Capture - cannot open card: {}", e);
        //         }
        //     }

        // use std::ffi::CString;
        // use alsa::hctl::HCtl;
        // let h = HCtl::open(&CString::new(format!("hw:{}", a.get_index())).unwrap(), false).unwrap();
        // h.load().unwrap();
        // for b in h.elem_iter() {
        //     use alsa::ctl::ElemIface;
        //     let id = b.get_id().unwrap();
        //     let name = id.get_name().unwrap();
        //     let value = b.read().unwrap();
        //     println!("hw:{} {} = {:?}", a.get_index(), &name, value);

        //     if !name.ends_with(" Jack") { continue; }
        //     if name.ends_with(" Phantom Jack") {
        //         println!("{} is always present", &name[..name.len()-13])
        //     }
        //     else { println!("{} is {}", &name[..name.len()-5],
        //         if b.read().unwrap().get_boolean(0).unwrap() { "plugged in" } else { "unplugged" })
        //     }
        // }
        // }