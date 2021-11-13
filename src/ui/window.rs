use crate::{
    model2::events::UiEvent,
    ui::{utils, UiRuntime},
};
use gtk::{Application, Builder, Button, ButtonExt, Label, LabelExt, LevelBar, WidgetExt, Window};
use std::sync::Arc;

pub struct MainWindow {
    inner: Window,
    rt: UiRuntime,
    labels: Labels,
}

impl MainWindow {
    fn new(builder: &Builder, rt: UiRuntime) -> Arc<Self> {
        let inner = utils::get_object(builder, "maindialog");
        let labels = Labels::new(builder, &rt);
        let this = MainWindow { rt, inner, labels };

        // hook up the main dialog
        let minimise: Button = utils::get_object(builder, "minimise.maindialog");
        let win = this.inner.clone();
        minimise.connect_clicked(move |_| win.hide());

        Arc::new(this)
    }

    fn setup_draw_hook(self: &Arc<Self>) {
        let this = Arc::clone(self);
        self.inner.connect_draw(move |_, _| this.poll_updates());
    }

    pub fn show(&self) {
        self.inner.show();
    }

    /// This function is called every frame by Gtk to poll for updates
    ///
    /// **Don't block this function!** Only handle a certain number of
    /// update events.
    fn poll_updates(self: &Arc<Self>) -> gtk::Inhibit {
        let ev = match self.rt.rx_cmd.try_recv() {
            Ok(ev) => ev,
            _ => return gtk::Inhibit(false),
        };

        match ev {
            _ => {}
        }

        gtk::Inhibit(false)
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

pub(super) fn create(builder: &Builder, rt: UiRuntime) -> (Arc<MainWindow>, Application) {
    let app = Application::new(Some("jackctl.segfault"), Default::default())
        .expect("Failed to initialise Gtk application!");
    let win = MainWindow::new(builder, rt);
    win.setup_draw_hook();
    (win, app)
}
