/// Implements The UI logic for the ALSAMixer Style Sound Device Ctl interface.
use super::{pages::Pages, utils, UiRuntime};
use crate::model::card::{Card, CardId, CardStatus, MixerChannel};

use gtk::prelude::*;
use gtk::{Adjustment, Align, Orientation, PositionType, Scale, ScaleBuilder, Separator};

use async_std::sync::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct Mixer {
    cards: RwLock<HashMap<CardId, Card>>,
    dirty: AtomicBool,
    rt: UiRuntime,
}

impl Mixer {
    pub fn new(rt: UiRuntime) -> Self {
        println!("making mixer");
        Self {
            rt,
            dirty: AtomicBool::new(true),
            cards: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_card(&self, card: Card) {
        println!("adding card to mixer");
        self.cards.write().await.insert(card.id, card);
        self.dirty.fetch_or(true, Ordering::Relaxed);
    }

    pub async fn del_card(&self, id: CardId) {
        self.cards.write().await.remove(&id);
        self.dirty.fetch_or(true, Ordering::Relaxed);
    }

    pub async fn draw(&self, pages: &Pages) {
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }
        
        println!("drawing mixer");
        let cards = self.cards.read().await;

        let grid = utils::grid();
        //let mut handles = Vec::new();
        grid.set_hexpand(true);
        grid.set_vexpand(true);
        if cards.is_empty() {
            grid.attach(
                &utils::mixer_label("No controllable devices are detected.", false),0,0,1,1);
        } else {
            let mut x_pos = 0;
            // get the elements in order.
            let mut keys: Vec<&i32> = cards.keys().collect();
            keys.sort();
            for card in keys
                .iter()
                .map(|k| cards.get(*k).unwrap())
                //.filter(|x| x.state == CardStatus::Active)
            {
                let len = card.channels.len();
                if len == 0 {
                    grid.attach(
                        &utils::mixer_label(&card.name, false),
                        x_pos as i32,
                        3,
                        1,
                        1,
                    );
                    grid.attach(
                        &utils::mixer_label("Device Has No Controls", true),
                        x_pos as i32,
                        0,
                        1,
                        2,
                    );
                    x_pos += 1;
                } else {
                    grid.attach(
                        &utils::mixer_label(&card.name, false),
                        x_pos,
                        3,
                        len as i32,
                        1,
                    );

                    // get the card in order, for consistency with things like alsamixer.
                    let mut keys: Vec<&MixerChannel> = card.channels.values().collect();
                    keys.sort_by(|a, b| a.id.cmp(&b.id));

                    for channel in keys {
                        grid.attach(&utils::mixer_label(&channel.name, true), x_pos, 0, 1, 1);

                        let (scale, adjustment) = self.mixer_fader(card.id, channel);
                        grid.attach(&scale, x_pos, 1, 1, 1);

                        if channel.has_switch {
                            let cb =
                                self.mixer_checkbox(card.id, channel.id, channel.is_playback);
                            grid.attach(&cb, x_pos, 2, 1, 1);
                        };

                        x_pos += 1;

                        // let handle = MixerHandle {
                        //     card_id: card.id,
                        //     element_id: channel.id,
                        //     mute: cb_signal,
                        //     volume: (adjustment, scale_signal),
                        // };
                        // handles.push(handle);
                    }
                }
                grid.attach(&Separator::new(Orientation::Vertical), x_pos, 0, 1, 4);
                x_pos += 1;
            }
        }
        self.dirty.fetch_and(false, Ordering::Relaxed);
        pages.insert_horizontal("Mixer", &grid);
    }

    fn mixer_checkbox(
        &self,
        card_id: i32,
        channel: u32,
        output: bool,
    ) -> gtk::ToggleButton {
        let builder = gtk::ToggleButtonBuilder::new();
        let image = if output {
            gtk::Image::from_icon_name(Some("audio-volume-muted-symbolic"), gtk::IconSize::Button)
        } else {
            gtk::Image::from_icon_name(
                Some("microphone-sensitivity-muted-symbolic"),
                gtk::IconSize::Button,
            )
        };
        let button = builder
            .image(&image)
            .always_show_image(true)
            .image_position(gtk::PositionType::Bottom)
            .build();
        //button.set_active(model.connected_by_id(port1.id(), port2.id()));
        utils::margin(&button,5);
        button.set_halign(Align::Center);

        let model = self.rt.clone();

        let signal_id = button.connect_clicked(move |cb| {
            // model
            //     .lock()
            //     .unwrap()
            //     .get_pipe()
            //     .send(Event::SetMuting(card_id, channel, cb.get_active()))
            //     .unwrap();
        });
        button
    }

    fn mixer_fader(
        &self,
        card_id: i32,
        chan: &MixerChannel,
    ) -> (Scale, Adjustment) {
        let a = Adjustment::new(
            0.0,
            chan.volume_min as f64,
            chan.volume_max as f64,
            1.0,
            10.0,
            0.0,
        );

        let model = self.rt.clone();
        let chan_id = chan.id;

        // let signal = a.connect_value_changed(move |a| {
        //     // model
        //     //     .lock()
        //     //     .unwrap()
        //     //     .get_pipe()
        //     //     .send(Event::SetVolume(card_id, chan_id, a.get_value() as i64))
        //     //     .unwrap()
        // });

        let s = ScaleBuilder::new()
            .adjustment(&a)
            .orientation(Orientation::Vertical)
            .value_pos(PositionType::Bottom)
            .inverted(true)
            .hexpand(true)
            .height_request(200)
            .digits(0)
            .build();
        s.set_value_pos(PositionType::Bottom);
        (s, a)
    }
}
