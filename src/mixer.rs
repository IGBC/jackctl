use gtk::prelude::*;

use alsa::card::Iter as CardIter;
use alsa::mixer::{Mixer, Selem, SelemChannelId, SelemId};

use alsa::pcm::{Access, Format, HwParams, State, PCM};
use alsa::{Direction, ValueOr};

use std::cell::RefCell;
use std::rc::Rc;

use crate::model::{Model, CardStatus};

pub struct MixerController {
    model: Model,
}

const extended_sample_rates: [u32; 19] = [
    8000, 11025, 16000, 22050, 32000, 37800, 44056, 44100, 47250, 48000, 50000, 50400, 64000,
    88200, 96000, 176400, 192000, 352800, 384000,
];
const sample_rates: [u32; 9] = [
    8000, 16000, 32000, 44100, 48000, 88200, 96000, 176400, 192000,
];

impl MixerController {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
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
        //             //for rate in extended_sample_rates.iter() {
        //             for rate in extended_sample_rates.iter() {
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
        //             //for rate in extended_sample_rates.iter() {
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

        let this = Rc::new(RefCell::new(Self { model }));

        this.borrow_mut().update();
        let this_clone = this.clone();
        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().update();
            Continue(true)
        });

        this
    }

    fn update(&mut self) {
        let card_ids: Vec<i32> = self.model.borrow().cards.keys().map(|x| *x).collect();
        // first check for new cards
        for alsa_card in CardIter::new().map(|x| x.unwrap()) {
            if !card_ids.contains(&&alsa_card.get_index()) {
                self.model.borrow_mut().card_detected(alsa_card.get_index(), alsa_card.get_name().unwrap());
            }
        }

        for card in self.model.borrow_mut().cards.values_mut() {
            // todo map this into a proper match statement
            if card.state == crate::model::CardStatus::Unknown {
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
                    
                    for channel in mixer.iter() {
                        let s = Selem::new(channel).unwrap();
                        
                        let id = s.get_id();
                        let name = id.get_name().unwrap().to_string();
                        

                        if s.has_capture_volume() {
                            let (volume_min, volume_max) = s.get_capture_volume_range();
                            card.add_channel(id.get_index(),name, false, s.has_capture_switch(), volume_min, volume_max);
                        } else {
                            if s.has_playback_volume() {
                                let (volume_min, volume_max) = s.get_playback_volume_range();
                                card.add_channel(id.get_index(),name, false, s.has_playback_switch(), volume_min, volume_max);
                            }
                        };
                    }


                    card.state = CardStatus::Active;
                } else {
                    println!("Failed to enumerate card {} - {}", card.id, card.name());
                    card.state = CardStatus::EnumFailed;
                }
            }
        }
    } 

    fn attempt_playback_enumerate(&self, card: i32) -> alsa::Result<Vec<u32>> {
        // Open playback device
        let mut results = Vec::new();
        let pcm = PCM::new(&format!("hw:{}", card), Direction::Playback, false)?;
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_rate_resample(false).unwrap();
        for rate in extended_sample_rates.iter() {
            match hwp.test_rate(*rate) {
                Ok(()) => results.push(*rate),
                Err(_) => (),
            }
        }
        Ok(results)
    }

    fn attempt_capture_enumerate(&self, card: i32) -> alsa::Result<Vec<u32>> {
        // Open capture device
        let mut results = Vec::new();
        let pcm = PCM::new(&format!("hw:{}", card), Direction::Capture, false)?;
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_rate_resample(false).unwrap();
        for rate in extended_sample_rates.iter() {
            match hwp.test_rate(*rate) {
                Ok(()) => results.push(*rate),
                Err(_) => (),
            }
        }
        Ok(results)
    }

    fn pick_best_rate(& self, rates: &Vec<u32>) -> u32 {
        if rates.contains(&48000) {
            44100
        } else if rates.contains(&44100) {
            44100
        } else {
            *rates.last().unwrap()
        }
    }
}

// impl MixerModel {
//     pub fn new() -> Self {
//         let mut cards = Vec::new();
//         for card in CardIter::new().map(|x| x.unwrap()) {
//             let mixer = Mixer::new(&format!("hw:{}", card.get_index()), false).unwrap();
//             let mut channels = Vec::new();

//             for channel in mixer.iter() {
//                 let s = Selem::new(channel).unwrap();
//                 if s.has_capture_volume() {
//                     let (volume_min, volume_max) = s.get_capture_volume_range();
//                     channels.push(MixerChannel {
//                         id: s.get_id(),
//                         is_playback: false,
//                         has_switch: s.has_capture_switch(),
//                         volume_max,
//                         volume_min,
//                     });
//                 } else {
//                     if s.has_playback_volume() {
//                         let (volume_min, volume_max) = s.get_playback_volume_range();
//                         channels.push(MixerChannel {
//                             id: s.get_id(),
//                             is_playback: true,
//                             has_switch: s.has_playback_switch(),
//                             volume_max,
//                             volume_min,
//                         });
//                     }
//                 };
//             }

//             match Card::new(card.get_index(), card.get_name().unwrap()) {
//                 Some(c) => cards.push(c),
//                 None => (),
//             }
//             // cards.push(Card {
//             //     id: card.get_index(),
//             //     name: card.get_name().unwrap(),
//             //     channels,
//             // })
//         }
//         Self { cards }
//     }

//     pub fn update(&mut self) {
//         for card in CardIter::new().map(|x| x.unwrap()) {
//             let id = card.get_index();
//             if match self.cards.get(id) {
//                 Some(busy) =>
//             }
//     }

//     pub fn empty() -> Self {
//         Self { cards: Vec::new() }
//     }

//     pub fn iter(&self) -> std::slice::Iter<'_, Card> {
//         self.cards.iter()
//     }
// }

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
//                 //for rate in extended_sample_rates.iter() {
//                 for rate in extended_sample_rates.iter() {
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
//                 //for rate in extended_sample_rates.iter() {
//                 for rate in extended_sample_rates.iter() {
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
//         let inputs = if capture_rates.is_empty() {
//             None
//         } else {
//             if capture_rates.contains(&48000) {
//                 Some(48000)
//             } else {
//                 if capture_rates.contains(&44100) {
//                     Some(44100)
//                 } else {
//                     None
//                 }
//             }
//         };

//         let outputs = if playback_rates.is_empty() {
//             None
//         } else {
//             if playback_rates.contains(&48000) {
//                 Some(48000)
//             } else {
//                 if playback_rates.contains(&44100) {
//                     Some(44100)
//                 } else {
//                     None
//                 }
//             }
//         };

//         if inputs.is_none() && outputs.is_none() {
//             None
//         } else {
//             Some(Card {
//                 id,
//                 name,
//                 inputs,
//                 outputs,
//                 channels: Vec::new(),
//             })
//         }
//     }

//     pub fn name(&self) -> &str {
//         &self.name
//     }

//     pub fn len(&self) -> usize {
//         self.channels.len()
//     }

//     pub fn iter(&self) -> std::slice::Iter<'_, MixerChannel> {
//         self.channels.iter()
//     }

//     pub fn get_volume(&self, channel: &MixerChannel) -> i64 {
//         let mixer = Mixer::new(&format!("hw:{}", self.id), false).unwrap();
//         let element = mixer.find_selem(&channel.id).unwrap();
//         if channel.is_playback {
//             element
//                 .get_playback_volume(SelemChannelId::FrontLeft)
//                 .unwrap()
//         } else {
//             element
//                 .get_capture_volume(SelemChannelId::FrontLeft)
//                 .unwrap()
//         }
//     }

//     pub fn get_muting(&self, channel: &MixerChannel) -> bool {
//         let mixer = Mixer::new(&format!("hw:{}", self.id), false).unwrap();
//         let element = mixer.find_selem(&channel.id).unwrap();
//         let val = if channel.is_playback {
//             element
//                 .get_playback_switch(SelemChannelId::FrontLeft)
//                 .unwrap()
//         } else {
//             element
//                 .get_capture_switch(SelemChannelId::FrontLeft)
//                 .unwrap()
//         };
//         val == 0
//     }

//     pub fn set_volume(&self, channel: &MixerChannel, volume: i64) {
//         let mixer = Mixer::new(&format!("hw:{}", self.id), false).unwrap();
//         let element = mixer.find_selem(&channel.id).unwrap();
//         if channel.is_playback {
//             element.set_playback_volume_all(volume).unwrap();
//         } else {
//             element
//                 .set_capture_volume(SelemChannelId::FrontLeft, volume)
//                 .unwrap();
//             element
//                 .set_capture_volume(SelemChannelId::FrontRight, volume)
//                 .unwrap();
//         }
//     }

//     pub fn set_muting(&self, channel: &MixerChannel, mute: bool) {
//         let mixer = Mixer::new(&format!("hw:{}", self.id), false).unwrap();
//         let element = mixer.find_selem(&channel.id).unwrap();
//         let value = match mute {
//             true => 0,
//             false => 1,
//         };
//         if channel.is_playback {
//             element.set_playback_switch_all(value).unwrap()
//         } else {
//             element.set_capture_switch_all(value).unwrap()
//         }
//     }
// }
