use crate::cb_channel::{self, ReturningReceiver, ReturningSender};
use crate::model::card::{CardConfig, ChannelCount, MixerChannel, SampleRate, Volume};
use crate::model::events::{HardwareCardAction, HardwareCmd, HardwareEvent, MuteCmd, VolumeCmd};
use alsa::card::Card;
use alsa::card::Iter as CardIter;
use alsa::mixer::{Elem, Mixer, Selem, SelemChannelId, SelemId};
use alsa::pcm::{HwParams, PCM};
use alsa::Direction;
use async_std::{
    channel::{bounded, Receiver, Sender},
    sync::RwLock,
    task,
};
use std::collections::hash_map::HashMap;
use std::sync::Arc;

pub type CardId = i32;
pub type ChannelId = (u32, String); //this is basically a selemID;

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

#[derive(Clone)]
pub struct AlsaHandle {
    /// Send commands to the ALSA runtime
    cmd_tx: Sender<HardwareCmd>,
    /// Receive events from the ALSA runtime
    event_rx: Receiver<HardwareEvent>,
    /// Send card actions to ALSA runtime with blocking ACK
    card_tx: ReturningSender<HardwareCardAction, ()>,
}

impl AlsaHandle {
    pub async fn next_event(&self) -> Option<HardwareEvent> {
        self.event_rx.recv().await.ok()
    }

    // TODO: trait this
    pub async fn send_cmd(&self, cmd: HardwareCmd) {
        if let Err(_) = self.cmd_tx.send(cmd).await {
            println!("Failed to send CMD to hardware runtime!");
        }
    }

    pub fn close(&self) {
        self.cmd_tx.close();
        self.event_rx.close();
        self.card_tx.close();
    }
}

pub struct AlsaController {
    /// Send commands to the ALSA runtime
    cmd_rx: Receiver<HardwareCmd>,
    /// Receive events from the ALSA runtime
    event_tx: Sender<HardwareEvent>,
    /// Send card actions to ALSA runtime with blocking ACK
    card_rx: ReturningReceiver<HardwareCardAction, ()>,
    /// Cards we have already seen, for keeping track of enumeration
    known_cards: RwLock<HashMap<CardId, bool>>,
}

fn extract_selem(id: &SelemId) -> ChannelId {
    let index = id.get_index();
    let name = id.get_name().expect("could not get selemid name").to_owned();
    (index, name)
}

impl AlsaHandle {
    pub fn new() -> Self {
        // Open the channels
        let (event_tx, event_rx) = bounded(4);
        let (cmd_tx, cmd_rx) = bounded(4);
        let (card_tx, card_rx) = cb_channel::bounded(4);

        Arc::new(AlsaController {
            cmd_rx,
            card_rx,
            event_tx,
            known_cards: RwLock::new(HashMap::new()),
        })
        .bootstrap();

        Self {
            event_rx,
            cmd_tx,
            card_tx,
        }
    }
}

impl AlsaController {
    fn bootstrap(self: &Arc<Self>) {
        println!("bootstrapping the alsas...");
        {
            let rt = Arc::clone(self);
            task::spawn(async move { rt.do_cmd().await });
        }
        {
            let rt = Arc::clone(self);
            task::spawn(async move { rt.handle_events().await });
        }
        // {
        //     let rt = Arc::clone(&self);
        //     task::spawn(async move { rt.respond_card().await });
        // }
    }

    fn print_mixer_hw(card: CardId) {
        println!("============== Mixer for Card {} =================", card);
        let mixer = Mixer::new(&format!("hw:{}", card), false).unwrap();
        for (id, elem) in mixer.iter().enumerate() {
            let sid = Selem::new(elem).map(|s|{s.get_id()}).map(
            |id|{
                let i = id.get_index().clone();
                let s = id.get_name().map(|v|{v.to_owned()}).map_err(|e|{e.to_string()}).clone();
                (i, s)
            });
             
            println!("{}: elem={:?}, selemId={:?}", id, elem, sid);
        }
        println!("============== End of Mixer =====================");
    }

    async fn do_cmd(self: Arc<Self>) {
        while let Ok(event) = self.cmd_rx.recv().await {
            match event {
                HardwareCmd::SetMixerVolume(volume) => {
                    let mixer = Mixer::new(&format!("hw:{}", volume.card), false).unwrap();
                    let selemid = SelemId::new(&volume.channel.1, volume.channel.0);
                    let selem = mixer.find_selem(&selemid).unwrap();
                    let playback = selem.has_playback_volume();

                    Self::set_volume(playback, &selem, volume.volume);
                }

                HardwareCmd::SetMixerMute(mute) => {
                    let mixer = Mixer::new(&format!("hw:{}", mute.card), false).unwrap();
                    let selemid = SelemId::new(&mute.channel.1, mute.channel.0);
                    let selem = mixer.find_selem(&selemid).unwrap();
                    let playback = selem.has_playback_switch();

                    Self::set_muting(playback, &selem, mute.mute);
                }

                HardwareCmd::Shutdown => {
                    // Exit the event loop now.
                    break;
                }
            }
        }
    }

    async fn handle_events(self: Arc<Self>) {
        loop {
            if self.cmd_rx.is_closed() {
                break;
            }

            // Enumerate known cards and update known_cards cache
            self.update_card_cache().await;

            // ???
            let mut events = vec![];
            let mut dropped_cards = vec![];
            let mut cards = self.known_cards.write().await;

            for (card, is_ok) in cards.iter() {
                // if this card was enumerated correctly
                if *is_ok {
                    match Mixer::new(&format!("hw:{}", card), false) {
                        Ok(mixer) => {
                            for (elem_id, elem) in mixer.iter().enumerate() {
                                let selem = Selem::new(elem).unwrap();
                                self.get_channel_volumes(card, selem, &mut events);
                            }
                        }
                        Err(e) => {
                            // OK card is gone, we expected this eventually;
                            eprintln!("card{}: {}", card, e);
                            dropped_cards.push(*card);
                        }
                    }
                }
            }

            for card in dropped_cards {
                cards.remove(&card);
                eprintln!("card{}: sending drop card event", card);
                events.push(HardwareEvent::DropCard { id: card });
            }

            for e in events.into_iter() {
                self.event_tx.send(e).await;
            }

            // this rate limits updates to the mixers, we don't need to update the volumes
            // at 100 FPS
            task::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn get_channel_volumes(
        self: &Arc<Self>,
        card: &CardId,
        selem: Selem<'_>,
        events: &mut Vec<HardwareEvent>,
    ) {
        if selem.has_capture_volume() {
            if Self::has_switch(&selem) {
                let mute = Self::get_muting(false, &selem);
                events.push(HardwareEvent::UpdateMixerMute(MuteCmd {
                    card: *card,
                    channel: extract_selem(&selem.get_id()),
                    mute,
                }));
            }

            let volume = Self::get_volume(false, &selem);
            events.push(HardwareEvent::UpdateMixerVolume(VolumeCmd {
                card: *card,
                channel: extract_selem(&selem.get_id()),
                volume,
            }));
        } else {
            if selem.has_playback_volume() {
                if Self::has_switch(&selem) {
                    let mute = Self::get_muting(true, &selem);
                    events.push(HardwareEvent::UpdateMixerMute(MuteCmd {
                        card: *card,
                        channel: extract_selem(&selem.get_id()),
                        mute,
                    }));
                }

                let volume = Self::get_volume(true, &selem);
                events.push(HardwareEvent::UpdateMixerVolume(VolumeCmd {
                    card: *card,
                    channel: extract_selem(&selem.get_id()),
                    volume,
                }));
            }
        }
    }

    async fn update_card_cache(self: &Arc<Self>) {
        // todo poll alsa for shit
        for card in CardIter::new().map(|x| x.unwrap()) {
            let id = card.get_index();

            let mut cards = self.known_cards.write().await;

            if !cards.contains_key(&id) {
                // if we have not seen this card before then we enumerate it
                match Self::enumerate_card(id) {
                    Ok(Some((capture, playback, mixerchannels, name))) => {
                        match self
                            .event_tx
                            .send(HardwareEvent::NewCardFound {
                                id,
                                name,
                                capture,
                                playback,
                                mixerchannels,
                            })
                            .await
                        {
                            Ok(_) => (),
                            Err(e) => {
                                panic!("FATAL ERROR: ALSA Event tx - {}", e);
                            }
                        }
                        cards.insert(id, true);
                    }
                    Ok(None) => {
                        eprintln!("Error: Card {} had no playback or capture channels", id);
                        cards.insert(id, false);
                    }
                    Err(e) => {
                        cards.insert(id, false);
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    }

    /// The goggles do nothing...
    async fn respond_card(self: Arc<Self>) {
        while let Ok(event) = self.card_rx.recv().await {
            match event {
                (e, m) => {
                    println!("{:?}", (e, m));
                    todo!("Implement Card Events");
                }
            }
        }
    }

    fn enumerate_card(
        id: CardId,
    ) -> Result<
        Option<(
            Option<CardConfig>,
            Option<CardConfig>,
            Vec<MixerChannel>,
            String,
        )>,
        alsa::Error,
    > {
        let inputs = match Self::attempt_capture_enumerate(id) {
            Ok((rates, channels)) => {
                let rate = Self::pick_best_rate(&rates);
                Some(CardConfig {
                    sample_rate: rate,
                    channels,
                })
            }
            _ => None,
        };

        let outputs = match Self::attempt_playback_enumerate(id) {
            Ok((rates, channels)) => {
                let rate = Self::pick_best_rate(&rates);
                Some(CardConfig {
                    sample_rate: rate,
                    channels,
                })
            }
            _ => None,
        };

        let card = Card::new(id);
        let name = card.get_name()?;

        if inputs.is_some() || outputs.is_some() {
            // this is the old mixer enumeration code, but we're only running it once.
            // pray that cards do not dynamically change their mixer interfaces.
            let mixer = Mixer::new(&format!("hw:{}", id), false)?;

            let mut channels: Vec<MixerChannel> = Vec::new();

            for (mixer_id, channel) in mixer.iter().enumerate() {
                let mixer_id = mixer_id as u32;
                let s = Selem::new(channel).unwrap();

                let name = s.get_id().get_name()?.to_string();
                println!("Card {}, id {}, name: {}", id, mixer_id, name);

                if s.has_capture_volume() {
                    let (volume_min, volume_max) = s.get_capture_volume_range();
                    let has_switch = s.has_capture_switch();
                    let switch = if has_switch {
                        Self::get_muting(false, &s)
                    } else {
                        false
                    };

                    let volume = Self::get_volume(false, &s);

                    let mc = MixerChannel {
                        id: extract_selem(&s.get_id()),
                        dirty: false,
                        name,
                        is_playback: false,
                        has_switch,
                        volume_min,
                        volume_max,
                        volume,
                        switch,
                    };
                    channels.push(mc);
                } else {
                    if s.has_playback_volume() {
                        let (volume_min, volume_max) = s.get_playback_volume_range();
                        let has_switch = s.has_playback_switch();
                        let switch = if has_switch {
                            Self::get_muting(true, &s)
                        } else {
                            false
                        };

                        let volume = Self::get_volume(true, &s);

                        let mc = MixerChannel {
                            id: extract_selem(&s.get_id()),
                            dirty: false,
                            name,
                            is_playback: true,
                            has_switch,
                            volume_min,
                            volume_max,
                            volume,
                            switch,
                        };
                        channels.push(mc);
                    }
                };
            }

            Ok(Some((inputs, outputs, channels, name)))
        } else {
            println!("Failed to enumerate card {} - {} has no channels", id, name,);
            Ok(None)
        }
    }

    fn attempt_playback_enumerate(card: CardId) -> alsa::Result<(Vec<SampleRate>, ChannelCount)> {
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

    fn attempt_capture_enumerate(card: CardId) -> alsa::Result<(Vec<SampleRate>, ChannelCount)> {
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

    fn pick_best_rate(rates: &Vec<SampleRate>) -> SampleRate {
        println!("WARNING: rate picking is not correctly implemented");
        if rates.contains(&48000) {
            48000
        } else if rates.contains(&44100) {
            44100
        } else {
            *rates.last().unwrap()
        }
    }

    fn is_playback(channel: &Selem) -> bool {
        // Our default is capture masks playback
        if channel.has_capture_volume() {
            false
        } else {
            if channel.has_playback_volume() {
                true
            } else {
                panic!("Channel is both neither capture nor playback, you figure it out")
            }
        }
    }

    fn has_switch(channel: &Selem) -> bool {
        if Self::is_playback(channel) {
            channel.has_playback_switch()
        } else {
            channel.has_capture_switch()
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
