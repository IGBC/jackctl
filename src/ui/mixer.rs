/// Implements The UI logic for the ALSAMixer Style Sound Device Ctl interface.
use super::{pages::Pages, utils, UiRuntime};
use crate::model::card::{Card, CardId, ChannelId, MixerChannel, Volume};
use crate::model::events::{MuteCmd, UiEvent, VolumeCmd};

use glib::SignalHandlerId;
use gtk::prelude::*;
use gtk::{
    Adjustment, Align, Orientation, PositionType, Scale, ScaleBuilder, Separator, ToggleButton,
};

use async_std::sync::RwLock;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

struct MixerHandle {
    mute_button: Option<ToggleButton>,
    mute_handle: Option<SignalHandlerId>,
    scale_handle: SignalHandlerId,
    volume_setting: Adjustment,
}

pub(super) struct Mixer {
    cards: RwLock<HashMap<CardId, Card>>,
    handles: RwLock<BTreeMap<(CardId, ChannelId), MixerHandle>>,
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
            handles: RwLock::new(BTreeMap::new()),
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

    pub async fn update_volume(&self, id: CardId, channel: ChannelId, volume: Volume) {
        self.update_parameter(id, channel, |handle| {
            handle.volume_setting.set_value(volume as f64);
        })
        .await;
    }

    pub async fn update_mute(&self, id: CardId, channel: ChannelId, mute: bool) {
        self.update_parameter(id, channel, |handle| {
            if handle.mute_button.is_some() {
                let signal = handle.mute_handle.as_ref().unwrap();
                let button = handle.mute_button.as_ref().unwrap();
                button.block_signal(signal);
                button.set_active(mute);
                button.unblock_signal(signal);
            } else {
                println!("attempting to set mute on channel that doesn't have one");
            }
        })
        .await;
    }

    async fn update_parameter(&self, id: CardId, channel: ChannelId, cb: impl Fn(&MixerHandle)) {
        match self.handles.read().await.get(&(id, channel)) {
            Some(handle) => {
                cb(handle);
            }
            None => eprintln!("Attempt to set parameter on missing mixer thing"),
        }
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
                &utils::mixer_label("No controllable devices are detected.", false),
                0,
                0,
                1,
                1,
            );
        } else {
            let mut x_pos = 0;
            // get the elements in order.
            let mut keys: Vec<&i32> = cards.keys().collect();
            keys.sort();
            for card in keys.iter().map(|k| cards.get(*k).unwrap())
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

                        let (scale, volume_setting, scale_handle) =
                            self.mixer_fader(card.id, channel);
                        grid.attach(&scale, x_pos, 1, 1, 1);

                        let (mute_handle, mute_button) = if channel.has_switch {
                            let (cb, handle) =
                                self.mixer_checkbox(card.id, channel.id.clone(), channel.is_playback);
                            grid.attach(&cb, x_pos, 2, 1, 1);
                            (Some(handle), Some(cb))
                        } else {
                            (None, None)
                        };

                        x_pos += 1;

                        let handle = MixerHandle {
                            mute_handle,
                            mute_button,
                            volume_setting,
                            scale_handle,
                        };
                        self.handles
                            .write()
                            .await
                            .insert((card.id, channel.id.clone()), handle);
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
        channel: ChannelId,
        output: bool,
    ) -> (gtk::ToggleButton, SignalHandlerId) {
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
        utils::margin(&button, 5);
        button.set_halign(Align::Center);

        let model = self.rt.clone();

        let signal_id = button.connect_clicked(move |cb| {
            model.sender().send(UiEvent::SetMuting(MuteCmd {
                card: card_id,
                channel: channel.clone(),
                mute: cb.get_active(),
            }));
        });
        (button, signal_id)
    }

    fn mixer_fader(
        &self,
        card_id: i32,
        chan: &MixerChannel,
    ) -> (Scale, Adjustment, SignalHandlerId) {
        let a = Adjustment::new(
            0.0,
            chan.volume_min as f64,
            chan.volume_max as f64,
            1.0,
            10.0,
            0.0,
        );

        let model = self.rt.clone();

        let channel = chan.id.clone();
        let signal = a.connect_value_changed(move |a| {
            model.sender().send(UiEvent::SetVolume(VolumeCmd {
                card: card_id,
                channel: channel.clone(),
                volume: a.get_value() as i64,
            }));
        });

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
        (s, a, signal)
    }
}
