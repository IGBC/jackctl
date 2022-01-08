//! New model abstraction

pub mod card;
pub mod events;
pub mod port;
pub mod settings;

use self::card::{Card, CardId, CardStatus, CardUsage};
use self::events::{
    HardwareCmd, HardwareEvent, JackCardAction, JackCmd, JackEvent, UiCmd, UiEvent,
};
use crate::rts::{hardware::HardwareHandle, jack::JackHandle};
use crate::ui::UiHandle;
use async_std::{channel, task};
use futures::FutureExt;
use settings::Settings;
use std::collections::HashMap;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug)]
pub struct Model {
    jack_handle: JackHandle,
    ui_handle: UiHandle,
    hw_handle: HardwareHandle,
    settings: Arc<Settings>,

    /// Card data and state map
    cards: BTreeMap<CardId, Card>,

    /// exit condtion bit
    done: bool,
}

impl Model {
    /// Initialise a new model tree
    pub fn start(
        jack_handle: JackHandle,
        ui_handle: UiHandle,
        hw_handle: HardwareHandle,
        settings: Arc<Settings>,
    ) {
        Self {
            jack_handle,
            ui_handle,
            hw_handle,
            settings,
            cards: Default::default(),
            done: false,
        }
        .dispatch()
    }

    fn dispatch(self) {
        task::spawn(async move {
            run(self).await;
            info!("Model run loop shut down");
        });
    }
}

async fn next_ctrlc(h: &channel::Receiver<()>) -> Option<()> {
    h.recv().await.ok()
}

#[instrument(skip(m), level = "debug")]
async fn run(mut m: Model) {
    let jack_handle = m.jack_handle.clone();
    let ui_handle = m.ui_handle.clone();
    let hw_handle = m.hw_handle.clone();
    let (tx, ctrlc_handle_rx) = channel::bounded::<()>(1);

    let _ = ctrlc::set_handler(move || {
        task::block_on(async {
            let _ = tx.send(()).await;
        });
    });

    while !m.done {
        let mut jack_event_poll = Box::pin(jack_handle.next_event().fuse());
        let mut ui_event_poll = Box::pin(ui_handle.next_event().fuse());
        let mut hw_event_poll = Box::pin(hw_handle.next_event().fuse());
        let mut ctlc_event_poll = Box::pin(next_ctrlc(&ctrlc_handle_rx).fuse());

        futures::select! {
            ev = jack_event_poll  => match ev {
                Some(ev) => handle_jack_ev(&mut m, ev).await,
                None => return,
            },
            ev = ui_event_poll  => match ev {
                Some(ev) => handle_ui_ev(&mut m, ev).await,
                None => return,
            },
            ev = hw_event_poll  => match ev {
                Some(ev) => handle_hw_ev(&mut m, ev).await,
                None => return,
            },
            _ = ctlc_event_poll => {
                info!("=== Recieved Ctrl-C, Shutting Down ===");
                end_program(&mut m).await;
            },
        }
    }
}

/// Events from the jack runtime
async fn handle_jack_ev(m: &mut Model, ev: JackEvent) {
    debug!("Handling jack event: {:?}", ev);
    use JackEvent::*;
    match ev {
        XRun => m.ui_handle.send_cmd(UiCmd::IncrementXRun).await,
        JackSettings(settings) => m.ui_handle.send_cmd(UiCmd::JackSettings(settings)).await,
        AddPort(port) => m.ui_handle.send_cmd(UiCmd::AddPort(port)).await,
        DelPort(id) => m.ui_handle.send_cmd(UiCmd::DelPort(id)).await,
        AddConnection(a, b) => m.ui_handle.send_cmd(UiCmd::AddConnection(a, b)).await,
        DelConnection(a, b) => m.ui_handle.send_cmd(UiCmd::DelConnection(a, b)).await,
    }
}

/// Events from the UI runtime
async fn handle_ui_ev(m: &mut Model, ev: UiEvent) {
    debug!("Handling UI event: {:?}", ev);
    use UiEvent::*;
    match ev {
        SetMuting(mute) => m.hw_handle.send_cmd(HardwareCmd::SetMixerMute(mute)).await,
        SetVolume(volume) => {
            m.hw_handle
                .send_cmd(HardwareCmd::SetMixerVolume(volume))
                .await
        }
        CardUsage { card, usage, store } if usage && store => {
            debug!("Use and store card {}", card.name);
            m.settings.w().cards().set_card_usage(&card.name, true);
            m.settings.sync();
            signal_jack_card(card, m).await;
        }
        CardUsage { card, usage, .. } if usage => {
            debug!("Use (but don't store!) card {}", card.name);
            signal_jack_card(card, m).await;
        }
        CardUsage { card, store, .. } if store => {
            debug!("Done use and store card {}", card.name);
            m.settings.w().cards().set_card_usage(&card.name, false);
            m.settings.sync();
        }
        CardUsage { card, .. } => {
            debug!("User doesn't want to use or store card {}", card.name);
        }
        SetConnection(input, output, connect) => {
            m.jack_handle
                .send_cmd(JackCmd::ConnectPorts {
                    input,
                    output,
                    connect,
                })
                .await
        }
        Shutdown => {
            info!("=== Recieved Shutdown Event ===");
            end_program(m).await;
        }
    }
}

async fn end_program(m: &mut Model) {
    m.jack_handle.send_cmd(JackCmd::Shutdown).await;
    m.jack_handle.close();
    m.hw_handle.send_cmd(HardwareCmd::Shutdown).await;
    m.hw_handle.close();
    info!("=== Sending Terminate Request ===");
    m.ui_handle
        .send_cmd(UiCmd::YouDontHaveToGoHomeButYouCantStayHere)
        .await;
    m.done = true;
}

/// Events from the hardware runtime
async fn handle_hw_ev(m: &mut Model, ev: HardwareEvent) {
    debug!("Handling HW event: {:?}", ev);
    use HardwareEvent::*;
    match ev {
        NewCardFound {
            id,
            name,
            capture,
            playback,
            mixerchannels,
        } => {
            let mut channels = HashMap::new();

            for c in mixerchannels.iter() {
                channels.insert(c.id.clone(), c.to_owned());
            }

            let card = Card {
                id,
                name: name.clone(),
                capture,
                playback,
                channels,
                client_handle: None,
                state: CardStatus::New,
            };

            m.cards.insert(id, card.clone());
            let usage = m.settings.r().cards().use_card(&name);

            match usage {
                CardUsage::Yes => {
                    debug!("Signal jack to use card {:?}", card);
                    signal_jack_card(card, m).await
                }
                CardUsage::No => {
                    debug!("Settings file told us not to use this card >:c");
                }
                CardUsage::AskUser => {
                    debug!("Send UI::AskUser command for card {:?}", card);
                    m.ui_handle.send_cmd(UiCmd::AskCard(card)).await;
                }
            }
        }
        DropCard { id } => {
            let card = m.cards.remove(&id).unwrap();
            match card.client_handle {
                Some(id) => {
                    debug!("Dropping card with ID {}", id);
                    let _ = m
                        .jack_handle
                        .send_card_action(JackCardAction::StopCard { id })
                        .await;

                    m.ui_handle.send_cmd(UiCmd::DelCard(id as i32));
                }
                None => {
                    error!("[Error]: Attempt to drop card that was never started, was there an error starting it?")
                }
            }
        }
        UpdateMixerVolume(volume) => {
            let c = m.cards.get_mut(&volume.card).unwrap();
            let chan = c.channels.get_mut(&volume.channel).unwrap();
            let oldv = chan.volume;
            m.ui_handle.send_cmd(UiCmd::VolumeChange(volume)).await;
        }
        UpdateMixerMute(mute) => {
            let c = m.cards.get_mut(&mute.card).unwrap();
            let chan = c.channels.get_mut(&mute.channel).unwrap();
            let oldm = chan.switch;
            m.ui_handle.send_cmd(UiCmd::MuteChange(mute)).await;
        }
    }
}

async fn signal_jack_card(card: Card, m: &mut Model) {
    let capture = card.capture().clone();
    let playback = card.playback().clone();

    if let (Some((r_in, n_in)), Some((r_out, n_out))) = (capture, playback) {
        if r_in != r_out {
            warn!("IN rate is not equal to OUT rate");
        }

        // Inform Jack here
        let client_handle = m
            .jack_handle
            .send_card_action(JackCardAction::StartCard {
                id: card.id.to_string(),
                name: card.name.clone(),
                rate: r_in, // FIXME: AAAAAAAAAAAAAAAAAAAAAAAH!
                in_ports: n_in,
                out_ports: n_out,
            })
            .await;
        match client_handle {
            Ok(h) => {
                m.cards.get_mut(&card.id).unwrap().client_handle = Some(h);
                m.ui_handle.send_cmd(UiCmd::AddCard(card)).await;
            }
            Err(e) => error!("Card {} Could not be started by jack: {}", card.id, e),
        }
    }
}
