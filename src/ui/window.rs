use crate::{
    model2::{
        events::{UiCmd, UiEvent},
        port::PortType,
    },
    ui::{matrix::AudioMatrix, pages::Pages, utils, UiRuntime, STYLE},
};
use async_std::task::block_on;
use gio::ApplicationExt;
use gtk::{
    Application, Builder, Button, ButtonExt, CssProviderExt, GtkWindowExt, Label, LabelExt,
    LevelBar, ModelButton, WidgetExt, Window,
};
use std::sync::Arc;

pub struct MainWindow {
    inner: Window,
    rt: UiRuntime,
    labels: Labels,
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
        self.inner.connect_draw(move |_, _| this.poll_updates());

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
            this.setup_ui(app, &builder);
        });

        self.inner.show();
    }

    /// This function is called when Gtk application starts up
    ///
    /// Don't call it from outside this type!
    fn setup_ui(&self, app: &Application, builder: &Builder) {
        let quit: ModelButton = utils::get_object(&builder, "quit.mainmenu");
        let app = app.clone();
        quit.connect_clicked(move |_| app.quit());
    }

    /// This function is called every frame by Gtk to poll for updates
    ///
    /// **Don't block this function!** Only handle a certain number of
    /// update events.
    fn poll_updates(self: &Arc<Self>) -> gtk::Inhibit {
        // TODO: make this less dumb
        let ev = match self.rt.rx_cmd.try_recv() {
            Ok(ev) => ev,
            _ => return gtk::Inhibit(false),
        };

        block_on(async move { self.update(ev).await });

        gtk::Inhibit(false)
    }

    async fn update(self: &Arc<Self>, cmd: UiCmd) {
        // TODO: handle more than one event at a time
        match cmd {
            UiCmd::AddPort {
                id,
                tt,
                is_hw,
                input,
                client_name,
                port_name,
            } => match tt {
                PortType::Audio => {
                    self.audio_matrix
                        .add_port(id, input, is_hw, client_name, port_name)
                        .await
                }
                // PortType::Midi => self.midi_matrix.add_port(id, input, client_name, port_name),
                PortType::Unknown | _ => {
                    println!("Unknown port type!");
                }
            },
            _ => {}
        }

        // ==^-^== Redraw all dirty elements ==^-^==
        self.audio_matrix.draw().await;
    }
}

/// UI state for various labels in the UI
struct Labels {
    // xruns display
    xruns_label: Label,
    xruns_btn: Button,

    // cpu usage display
    cpu_label: Label,
    cpu_mtr: LevelBar,

    // Performance display
    perf_rate: Label,
    perf_frames: Label,
    perf_latency: Label,
}

impl Labels {
    fn new(builder: &Builder, rt: &UiRuntime) -> Self {
        let this = Self {
            xruns_label: utils::get_object(builder, "label.xruns.maindialog"),
            xruns_btn: utils::get_object(builder, "button.xruns.maindialog"),
            cpu_label: utils::get_object(builder, "label.cpu.maindialog"),
            cpu_mtr: utils::get_object(builder, "meter.cpu.maindialog"),
            perf_rate: utils::get_object(&builder, "samplerate.performance.maindialog"),
            perf_frames: utils::get_object(&builder, "wordsize.performance.maindialog"),
            perf_latency: utils::get_object(&builder, "latency.performance.maindialog"),
        };

        // Setup various ui callbacks
        this.xruns_label.set_markup(&format!("{} XRuns", "N.D."));
        let ev_tx = rt.sender();
        this.xruns_btn.connect_clicked(move |icon| {
            ev_tx.clone().send(UiEvent::ResetXruns);
            icon.hide();
        });

        this
    }
}

pub(super) fn create(builder: Builder, rt: UiRuntime) -> (Arc<MainWindow>, Application) {
    let app = Application::new(Some("jackctl.segfault"), Default::default())
        .expect("Failed to initialise Gtk application!");
    let win = MainWindow::new(&builder, rt);
    win.setup_application(&app, builder);
    (win, app)
}
