use super::settings::SettingsWindow;
use super::Questionaire;
use crate::{
    model::{
        card::Card,
        events::{JackSettings, UiCmd, UiEvent},
        port::{Port, PortType},
    },
    settings::Settings,
    ui::{
        about::About, card_query::CardQuery, matrix::Matrix, mixer::Mixer, pages::Pages, utils,
        UiRuntime,
    },
};
use async_std::task::block_on;
use atomptr::AtomPtr;
use gio::ApplicationExt;
use glib::Continue;
use gtk::{
    Application, Builder, Button, ButtonExt, GtkWindowExt, Label, LabelExt, LevelBar, LevelBarExt,
    ModelButton, WidgetExt, Window,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub(super) type CardQuestionaire = Arc<AtomPtr<Option<Questionaire<Card>>>>;

pub struct MainWindow {
    app: Application,
    inner: Window,
    rt: UiRuntime,
    settings: Arc<Settings>,
    labels: Arc<Labels>,
    pages: Pages,
    audio_matrix: Matrix,
    midi_matrix: Matrix,
    mixer: Mixer,
    cards: CardQuestionaire,
    settings_window: Arc<SettingsWindow>,
}

impl MainWindow {
    fn new(
        app: &Application,
        settings: Arc<Settings>,
        builder: &Builder,
        rt: UiRuntime,
    ) -> Arc<Self> {
        let inner = utils::get_object(builder, "maindialog");
        let labels = Labels::new(builder, &rt);
        let pages = Pages::new(
            builder,
            vec!["Audio Matrix", "MIDI Matrix", "Mixer", "Setup"],
        );

        let quit: ModelButton = utils::get_object(&builder, "quit.mainmenu");
        let rtt = rt.clone();
        quit.connect_clicked(move |_| rtt.sender().send(UiEvent::Shutdown));

        let settings_clone = settings.clone();
        let rtt = rt.clone();

        let this = MainWindow {
            audio_matrix: Matrix::new(rt.clone(), "Audio Matrix"),
            midi_matrix: Matrix::new(rt.clone(), "MIDI Matrix"),
            mixer: Mixer::new(rt.clone()),
            rt,
            inner,
            labels,
            pages,
            settings,
            app: app.clone(),
            cards: Default::default(),
            settings_window: SettingsWindow::new(settings_clone, rtt),
        };

        // hook up the main dialog
        let minimise: Button = utils::get_object(builder, "minimise.maindialog");
        let win = this.inner.clone();
        minimise.connect_clicked(move |_| win.hide());

        let arc = Arc::new(this);
        let arc_clone = arc.clone();

        let settings_button: Button = utils::get_object(&builder, "settingsButton");
        settings_button.connect_clicked(move |_| {
            trace!("Clicked settings button");
            arc_clone.settings_window.show();
        });

        arc
    }

    /// This function sets up a bunch of Gtk state and must be called!
    fn setup_application(self: &Arc<Self>, app: &Application, builder: Builder) {
        let this = Arc::clone(self);
        glib::timeout_add_local(200, move || {
            this.poll_updates();
            Continue(true)
        });

        // Setup about screen
        About::new(&builder).button(&builder);

        let this = Arc::clone(self);
        app.connect_startup(move |app| {
            this.inner.set_application(Some(app));
            block_on(async { this.setup_ui(app, &builder).await });
        });

        self.inner.show_all();
    }

    /// This function is called when Gtk application starts up
    ///
    /// Don't call it from outside this type!
    async fn setup_ui(&self, app: &Application, builder: &Builder) {
        // ==^-^== Initially draw all UI elements ==^-^==
        self.audio_matrix.draw(&self.settings, &self.pages).await;
        self.midi_matrix.draw(&self.settings, &self.pages).await;
        self.pages.show_all();
    }

    /// This function is called every frame by Gtk to poll for updates
    ///
    /// **Don't block this function!** Only handle a certain number of
    /// update events.
    fn poll_updates(self: &Arc<Self>) -> gtk::Inhibit {
        let evs = match self.rt.get_cmds(100) {
            Some(evs) => evs,
            None => return gtk::Inhibit(false),
        };

        block_on(async move {
            // ==^-^== First handle all update events ==^-^==
            for ev in evs {
                self.update(ev).await;
            }

            // ==^-^== Then redraw all dirty elements ==^-^==
            self.audio_matrix.draw(&self.settings, &self.pages).await;
            self.midi_matrix.draw(&self.settings, &self.pages).await;
            self.mixer.draw(&self.pages).await;
            self.pages.show_all();
        });

        gtk::Inhibit(false)
    }

    async fn update(self: &Arc<Self>, cmd: UiCmd) {
        match cmd {
            UiCmd::AddPort(Port {
                client_name,
                port_name,
                id,
                tt,
                dir,
                is_hw,
            }) => match tt {
                PortType::Audio => {
                    self.audio_matrix
                        .add_port(id, dir, is_hw, client_name, port_name)
                        .await
                }
                PortType::Midi => {
                    self.midi_matrix
                        .add_port(id, dir, is_hw, client_name, port_name)
                        .await
                }
                PortType::Unknown | _ => {
                    warn!("Unknown port type (if on pipewire, please report!)");
                }
            },
            UiCmd::DelPort(port_id) => {
                self.audio_matrix.rm_port(port_id).await;
                self.midi_matrix.rm_port(port_id).await;
            }

            UiCmd::IncrementXRun => self.labels.increment_xruns(),
            UiCmd::JackSettings(JackSettings {
                cpu_percentage,
                sample_rate,
                buffer_size,
                latency,
            }) => {
                self.labels.update_frames(buffer_size);
                self.labels.update_latency(latency);
                self.labels.update_rate(sample_rate);
                self.labels.update_cpu(cpu_percentage);
            }
            UiCmd::AskCard(card) => {
                trace!("Ask the user whether we should use {:?}", card);
                match **self.cards.get_ref() {
                    Some(ref q) => q.send(card),
                    None => {
                        let arc = Arc::clone(&self.cards);
                        let cards = CardQuery::new(arc, self.rt.clone(), &self.app, &self.inner);
                        cards.send(card);
                        self.cards.swap(Some(cards));
                    }
                }
            }
            UiCmd::AddConnection(a, b) => {
                self.audio_matrix.add_connection(b, a).await;
            }
            UiCmd::DelConnection(a, b) => {
                self.audio_matrix.rm_connection(b, a).await;
            }
            UiCmd::MuteChange(m) => {
                self.mixer.update_mute(m.card, m.channel, m.mute).await;
            }
            UiCmd::VolumeChange(v) => {
                self.mixer.update_volume(v.card, v.channel, v.volume).await;
            }
            UiCmd::AddCard(c) => {
                self.mixer.add_card(c).await;
            }
            UiCmd::DelCard(id) => {
                self.mixer.del_card(id).await;
            }
            UiCmd::YouDontHaveToGoHomeButYouCantStayHere => {
                self.app.quit();
            }
        }
    }

    pub fn get_inner(&self) -> Window {
        self.inner.clone()
    }
}

/// UI state for various labels in the UI
struct Labels {
    // xruns display
    xruns_label: Label,
    xruns_btn: Button,
    xruns_ctr: AtomicUsize,

    // cpu usage display
    cpu_label: Label,
    cpu_mtr: LevelBar,

    // Performance display
    perf_rate: Label,
    perf_frames: Label,
    perf_latency: Label,
}

impl Labels {
    fn new(builder: &Builder, _rt: &UiRuntime) -> Arc<Self> {
        let this = Arc::new(Self {
            // xruns ui state
            xruns_label: utils::get_object(builder, "label.xruns.maindialog"),
            xruns_btn: utils::get_object(builder, "button.xruns.maindialog"),
            xruns_ctr: AtomicUsize::new(0),

            // cpu labels
            cpu_label: utils::get_object(builder, "label.cpu.maindialog"),
            cpu_mtr: utils::get_object(builder, "meter.cpu.maindialog"),

            // jack performance stats
            perf_rate: utils::get_object(&builder, "samplerate.performance.maindialog"),
            perf_frames: utils::get_object(&builder, "wordsize.performance.maindialog"),
            perf_latency: utils::get_object(&builder, "latency.performance.maindialog"),
        });

        // Setup XRuns logic
        this.setup_tooltips();
        this.reset_xruns();
        let this_ = Arc::clone(&this);
        this.xruns_btn.connect_clicked(move |icon| {
            this_.reset_xruns();
            icon.hide();
        });

        this
    }

    fn setup_tooltips(self: &Arc<Self>) {
        self.cpu_label
            .set_tooltip_markup(Some("Jack CPU utilisation"));
        self.cpu_mtr
            .set_tooltip_markup(Some("Jack CPU utilisation"));
        self.perf_rate
            .set_tooltip_markup(Some("Jack sample rate (Hertz)"));
        self.perf_frames
            .set_tooltip_markup(Some("Sample buffer size (words)"));
        self.perf_latency
            .set_tooltip_markup(Some("Jack pipeline latency (milliseconds)"));
    }

    /// Reset the xruns counter and update the label
    fn reset_xruns(self: &Arc<Self>) {
        self.xruns_ctr.store(0, Ordering::Relaxed);
        self.xruns_label.set_markup(&format!("0 XRuns"));
        self.xruns_btn.hide();
    }

    /// Increment xruns counter and update the label
    fn increment_xruns(self: &Arc<Self>) {
        let val = self.xruns_ctr.fetch_add(1, Ordering::Relaxed);
        self.xruns_label.set_markup(&format!("{} XRuns", val + 1));
        self.xruns_btn.show();
    }

    fn update_cpu(self: &Arc<Self>, cpu: f32) {
        self.cpu_label.set_markup(&format!("{}%", cpu.trunc()));
        self.cpu_mtr.set_value(cpu as f64);
    }

    fn update_rate(self: &Arc<Self>, rate: u64) {
        self.perf_rate.set_markup(&format!("{}Hz", rate));
    }

    fn update_frames(self: &Arc<Self>, buf_size: u64) {
        self.perf_frames.set_markup(&format!("{}w", buf_size));
    }

    fn update_latency(self: &Arc<Self>, latency: f32) {
        self.perf_latency
            .set_markup(&format!("{}ms", latency.trunc()));
    }
}

pub(super) fn create(app: &Application, settings: Arc<Settings>, rt: UiRuntime) -> Arc<MainWindow> {
    let builder = Builder::from_resource("/net/jackctl/Jackctl/main.glade");
    let win = MainWindow::new(app, settings, &builder, rt);
    win.setup_application(&app, builder);
    win
}
