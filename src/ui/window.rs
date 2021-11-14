use crate::{
    model2::{
        events::{JackSettings, UiCmd},
        port::{Port, PortType},
    },
    ui::{about::About, matrix::AudioMatrix, pages::Pages, utils, UiRuntime, STYLE},
};
use async_std::task::block_on;
use gio::ApplicationExt;
use glib::Continue;
use gtk::{
    Application, Builder, Button, ButtonExt, CssProviderExt, GtkWindowExt, Label, LabelExt,
    LevelBar, LevelBarExt, ModelButton, WidgetExt, Window,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub struct MainWindow {
    inner: Window,
    rt: UiRuntime,
    labels: Arc<Labels>,
    pages: Pages,
    audio_matrix: AudioMatrix,
}

impl MainWindow {
    fn new(builder: &Builder, rt: UiRuntime) -> Arc<Self> {
        let inner = utils::get_object(builder, "maindialog");
        let labels = Labels::new(builder, &rt);
        let pages = Pages::new(builder, vec!["Matrix", "MIDI", "Mixer", "Setup"]);

        let this = MainWindow {
            rt,
            inner,
            labels,
            pages,
            audio_matrix: AudioMatrix::new(),
        };

        // hook up the main dialog
        let minimise: Button = utils::get_object(builder, "minimise.maindialog");
        let win = this.inner.clone();
        minimise.connect_clicked(move |_| win.hide());

        Arc::new(this)
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
            // The CSS "magic" happens here.
            let provider = gtk::CssProvider::new();
            provider
                .load_from_data(STYLE.as_bytes())
                .expect("Failed to load CSS");

            // We give the CssProvided to the default screen so the CSS rules we added
            // can be applied to our window.
            gtk::StyleContext::add_provider_for_screen(
                &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            this.inner.set_application(Some(app));
            block_on(async { this.setup_ui(app, &builder).await });
        });

        self.inner.show_all();
    }

    /// This function is called when Gtk application starts up
    ///
    /// Don't call it from outside this type!
    async fn setup_ui(&self, app: &Application, builder: &Builder) {
        let quit: ModelButton = utils::get_object(&builder, "quit.mainmenu");
        let app = app.clone();
        quit.connect_clicked(move |_| app.quit());

        // ==^-^== Initially draw all UI elements ==^-^==
        self.audio_matrix.draw().await;
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
            self.audio_matrix.draw().await;
        });

        gtk::Inhibit(false)
    }

    async fn update(self: &Arc<Self>, cmd: UiCmd) {
        // TODO: handle more than one event at a time
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
                    panic!("WE HIT THIS CODE POINT HURRAY!!!!");
                    self.audio_matrix
                        .add_port(id, dir, is_hw, client_name, port_name)
                        .await
                }
                // PortType::Midi => self.midi_matrix.add_port(id, input, client_name, port_name),
                PortType::Unknown | _ => {
                    println!("Unknown port type!");
                }
            },
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
            _ => {}
        }
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

pub(super) fn create(builder: Builder, rt: UiRuntime) -> (Arc<MainWindow>, Application) {
    let app = Application::new(Some("jackctl.segfault"), Default::default())
        .expect("Failed to initialise Gtk application!");
    let win = MainWindow::new(&builder, rt);
    win.setup_application(&app, builder);
    (win, app)
}
