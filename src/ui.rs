//! All of the GTK stuff stuffed into a module so that the rest of the program can be designed sanely.
//! 
//! Don't expect me to document this module. It will change with every tiny change to the GUI.

use std::cell;
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;

use glib::signal::SignalHandlerId;
use gtk::prelude::*;
use gtk::Application;
use gtk::Builder;
use gtk::{
    Adjustment, Align, Button, CheckButton, Grid, Image, Label, LevelBar, Notebook, Orientation,
    PositionType, Scale, ScaleBuilder, Separator, Window,
};

use crate::mixer::MixerController;

use crate::jack::JackController;
use crate::model::{MixerChannel, Model, ModelInner, Port, PortGroup, Event};

use libappindicator::{AppIndicator, AppIndicatorStatus};

struct MixerHandle {
    card_id: i32,
    element_id: u32,
    mute: Option<(CheckButton, SignalHandlerId)>,
    volume: (Adjustment, SignalHandlerId),
}

pub struct MainDialog {
    state: Model,
    jack_controller: Rc<RefCell<JackController>>,
    alsa_controller: Rc<RefCell<MixerController>>,

    //builder: Builder,
    window: Window,
    xruns_label: Label,
    xruns_icon: Image,
    cpu_label: Label,
    cpu_meter: LevelBar,
    performance_rate: Label,
    performance_frames: Label,
    performance_latency: Label,
    tabs: Notebook,

    audio_matrix: Vec<(usize, usize, CheckButton, SignalHandlerId)>,
    midi_matrix: Vec<(usize, usize, CheckButton, SignalHandlerId)>,
    mixer_handles: Vec<MixerHandle>,
}

fn get_object<T>(builder: &Builder, name: &str) -> T
where
    T: gtk::prelude::IsA<glib::object::Object>,
{
    let o: T = builder
        .get_object(name)
        .expect(&format!("UI file does not contain {}", name));
    o
}

pub fn init_ui(
    state: Model,
    jack_controller: Rc<RefCell<JackController>>,
    alsa_controller: Rc<RefCell<MixerController>>,
) -> Rc<RefCell<MainDialog>> {
    // define the gtk application with a unique name and default parameters
    let _application = Application::new(Some("jackctl.segfault"), Default::default())
        .expect("Initialization failed");

    // this registers a closure (executing our setup_gui function)
    //that has to be run on a `activate` event, triggered when the UI is loaded
    //application.connect_activate(move |app| {
    //
    let glade_src = include_str!("jackctl.glade");

    // this builder provides access to all components of the defined ui
    let builder = Builder::from_string(glade_src);

    let this = MainDialog::new(builder, state, jack_controller, alsa_controller);

    let win_clone = this.clone();

    let mut indicator = AppIndicator::new("jackctl", "");
    indicator.set_status(AppIndicatorStatus::Active);
    let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    indicator.set_icon_theme_path(icon_path.to_str().unwrap());
    indicator.set_icon_full("jackctl-symbolic", "icon");
    let mut m = gtk::Menu::new();
    let mi = gtk::CheckMenuItem::with_label("exit");
    mi.connect_activate(|_| {
        gtk::main_quit();
    });
    m.append(&mi);
    let mi = gtk::CheckMenuItem::with_label("show");
    mi.connect_activate(move |_| {
        win_clone.borrow().show();
    });
    m.append(&mi);
    indicator.set_menu(&mut m);
    m.show_all();

    this
}

impl MainDialog {
    pub fn new(
        builder: Builder,
        state: Model,
        jack_controller: Rc<RefCell<JackController>>,
        alsa_controller: Rc<RefCell<MixerController>>,
    ) -> Rc<RefCell<Self>> {
        // Initialise the state:

        // find the main dialog
        let window: Window = get_object(&builder, "maindialog");

        // hook up the minimise button
        let minimise: Button = get_object(&builder, "minimise.maindialog");
        let window_clone = window.clone();
        minimise.connect_clicked(move |_| window_clone.hide());

        // Setup xruns display
        let xruns_label: Label = get_object(&builder, "label.xruns.maindialog");
        xruns_label.set_markup(&format!("{} XRuns", "N.D."));
        let xruns_icon: Image = get_object(&builder, "icon.xruns.maindialog");
        xruns_icon.set_from_icon_name(Some("dialog-warning-symbolic"), gtk::IconSize::Button);
        //xruns_icon.connect_clicked(move |icon| { state.borrow().reset_xruns(); icon.hide();})

        // Setup CPU Meter
        let cpu_label: Label = get_object(&builder, "label.cpu.maindialog");
        let cpu_meter: LevelBar = get_object(&builder, "meter.cpu.maindialog");

        // Setup Time status display
        let performance_rate = get_object(&builder, "samplerate.performance.maindialog");
        let performance_frames = get_object(&builder, "wordsize.performance.maindialog");
        let performance_latency = get_object(&builder, "latency.performance.maindialog");

        // Setup notebook view
        let tabs = get_object(&builder, "tabs.maindialog");

        // Setup about screen
        let about: Window = get_object(&builder, "aboutdialog");

        // Save the bits we need
        let this = Rc::new(RefCell::new(MainDialog {
            state,
            jack_controller,
            alsa_controller,
            //builder,
            window: window.clone(),
            xruns_label,
            xruns_icon,
            cpu_label,
            cpu_meter,
            performance_rate,
            performance_frames,
            performance_latency,
            tabs,
            audio_matrix: Vec::new(),
            midi_matrix: Vec::new(),
            mixer_handles: Vec::new(),
        }));

        // hookup the update function
        let this_clone = this.clone();

        // Setup Main Menu
        let quit: gtk::ModelButton = get_object(&builder, "quit.mainmenu");
        quit.connect_clicked(|_| gtk::main_quit());
        let aboutbutton: gtk::ModelButton = get_object(&builder, "about.mainmenu");
        aboutbutton.connect_clicked(move |_| this_clone.borrow().show());

        // hookup the update function
        let this_clone = this.clone();

        window.connect_draw(move |_, _| this_clone.borrow_mut().update_ui());

        this
    }

    pub fn show(&self) {
        self.window.show_all();
        self.window.present();
    }

    fn update_matrix(
        &self,
        inputs: &PortGroup,
        outputs: &PortGroup,
    ) -> (Grid, Vec<(usize, usize, CheckButton, SignalHandlerId)>) {
        let grid = grid();

        if inputs.is_empty() || outputs.is_empty() {
            let l = grid_label("No ports are currently available.", false);
            l.set_halign(Align::Center);
            grid.attach(&l, 0, 0, 1, 1);

            (grid, Vec::new())
        } else {
            let i_groups = inputs.no_groups();
            let o_groups = outputs.no_groups();
            let n_audio_inputs = inputs.len();
            let n_audio_outputs = outputs.len();
            let max_x: i32 = 2 + i_groups as i32 + n_audio_inputs as i32 - 1;
            let max_y: i32 = 2 + o_groups as i32 + n_audio_outputs as i32 - 1;

            let mut curr_x = 2;
            for (i, g) in inputs.iter().enumerate() {
                let l = grid_label(g.name(), true);
                l.set_line_wrap(true);
                grid.attach(&l, curr_x, 0, g.len() as i32, 1);

                for n in g.iter() {
                    grid.attach(&grid_label(n.name(), true), curr_x, 1, 1, 1);
                    curr_x += 1;
                }

                if i < i_groups - 1 {
                    grid.attach(&Separator::new(Orientation::Vertical), curr_x, 0, 1, max_y);
                    curr_x += 1;
                }
            }

            let mut curr_y = 2;
            for (i, g) in outputs.iter().enumerate() {
                let l = grid_label(g.name(), false);
                l.set_line_wrap(true);
                grid.attach(&l, 0, curr_y, 1, g.len() as i32);

                for n in g.iter() {
                    grid.attach(&grid_label(n.name(), false), 1, curr_y, 1, 1);
                    curr_y += 1;
                }

                if i < o_groups - 1 {
                    grid.attach(
                        &Separator::new(Orientation::Horizontal),
                        0,
                        curr_y,
                        max_x,
                        1,
                    );
                    curr_y += 1;
                }
            }

            let mut curr_x = 2;
            let mut handles = Vec::new();
            for g2 in inputs.iter() {
                for n2 in g2.iter() {
                    let mut curr_y = 2;
                    for g1 in outputs.iter() {
                        for n1 in g1.iter() {
                            let (cb, handler) = self.grid_checkbox(n2, n1);
                            grid.attach(&cb, curr_x, curr_y, 1, 1);

                            handles.push((n1.id(), n2.id(), cb, handler));
                            curr_y += 1;
                        }
                        // skip over the separator;
                        curr_y += 1;
                    }
                    curr_x += 1;
                }
                // skip over the separator
                curr_x += 1;
            }

            (grid, handles)
        }
    }

    fn update_mixer(&self, model: &cell::RefMut<ModelInner>) -> (Grid, Vec<MixerHandle>) {
        let grid = grid();
        let mut handles = Vec::new();
        grid.set_hexpand(true);
        grid.set_vexpand(true);
        if model.cards.is_empty() {
            grid.attach(&mixer_label("No controllable devices are detected.", false), 0,0,1,1);
            return (grid, handles);
        }

        let mut x_pos = 0;
        // get the elements in order.
        let mut keys: Vec<&i32> = model.cards.keys().collect();
        keys.sort();
        for card in keys.iter().map(|k| model.cards.get(*k).unwrap()) {
            let len = card.len();
            if len == 0 {
                grid.attach(&mixer_label(card.name(), false), x_pos as i32, 3, 1, 1);
                grid.attach(
                    &mixer_label("Device Has No Controls", true),
                    x_pos as i32,
                    0,
                    1,
                    2,
                );
                x_pos += 1;
            } else {
                grid.attach(&mixer_label(card.name(), false), x_pos, 3, len as i32, 1);

                // get the card in order, for consistency with things like alsamixer.
                let mut keys: Vec<&MixerChannel> = card.iter().collect();
                keys.sort_by(|a, b| a.id.cmp(&b.id));

                for channel in keys {
                    grid.attach(&mixer_label(channel.get_name(), true), x_pos, 0, 1, 1);

                    let (scale, adjustment, scale_signal) = self.mixer_fader(card.id, channel);
                    grid.attach(&scale, x_pos, 1, 1, 1);

                    let cb_signal = if channel.has_switch {
                        let (cb, cb_signal) = self.mixer_checkbox(card.id, channel.id);
                        grid.attach(&cb, x_pos, 2, 1, 1);
                        Some((cb, cb_signal))
                    } else {
                        None
                    };
                    x_pos += 1;

                    let handle = MixerHandle {
                        card_id: card.id,
                        element_id: channel.id,
                        mute: cb_signal,
                        volume: (adjustment, scale_signal),
                    };
                    handles.push(handle);
                }
            }
            grid.attach(&Separator::new(Orientation::Vertical), x_pos, 0, 1, 4);
            x_pos += 1;
        }

        (grid, handles)
    }

    pub fn update_ui(&mut self) -> gtk::Inhibit {
        let mut model = self.state.borrow_mut();
        self.xruns_label
            .set_markup(&format!("{} XRuns", model.xruns()));
        if model.xruns() == 0 {
            self.xruns_icon.hide();
        } else {
            self.xruns_icon.show_all();
        }

        self.cpu_label
            .set_markup(&format!("{}%", model.cpu_percent.trunc()));
        self.cpu_meter.set_value(model.cpu_percent as f64);

        self.performance_rate
            .set_markup(&format!("{}Hz", model.sample_rate));
        self.performance_frames
            .set_markup(&format!("{}w", model.buffer_size));
        self.performance_latency
            .set_markup(&format!("{}ms", model.latency));

        if model.layout_dirty {
            model.layout_dirty = false;
            let page = self.tabs.get_current_page();

            // update Audio Matrix Tab
            let (audio_matrix, cb_vec) =
                self.update_matrix(model.audio_outputs(), model.audio_inputs());
            self.tabs.remove_page(Some(0));
            self.tabs
                .insert_page(&audio_matrix, Some(&Label::new(Some("Matrix"))), Some(0));
            self.audio_matrix = cb_vec;

            // update Midi Matrix Tab
            let (midi_matrix, cb_vec) =
                self.update_matrix(model.midi_outputs(), model.midi_inputs());
            self.tabs.remove_page(Some(1));
            self.tabs
                .insert_page(&midi_matrix, Some(&Label::new(Some("MIDI"))), Some(1));
            self.midi_matrix = cb_vec;

            // update Mixer Tab
            let (mixer_matrix, cb_vec) = self.update_mixer(&model);
            self.tabs.remove_page(Some(2));
            self.tabs
                .insert_page(&mixer_matrix, Some(&Label::new(Some("Mixer"))), Some(2));
            self.mixer_handles = cb_vec;

            self.tabs.show_all();

            self.tabs.set_current_page(page);
        }

        for (i, j, item, handle) in self.audio_matrix.iter() {
            item.block_signal(handle);
            item.set_active(model.connected_by_id(*j, *i));
            item.unblock_signal(handle);
        }

        for (j, i, item, handle) in self.midi_matrix.iter() {
            item.block_signal(handle);
            item.set_active(model.connected_by_id(*j, *i));
            item.unblock_signal(handle);
        }

        for element in self.mixer_handles.iter() {
            let (item, handle) = &element.volume;
            item.block_signal(handle);
            item.set_value(model.get_volume(element.card_id, element.element_id) as f64);
            item.unblock_signal(handle);
            if element.mute.is_some() {
                let (item, handle) = &element.mute.as_ref().unwrap();
                item.block_signal(handle);
                item.set_active(model.get_muting(element.card_id, element.element_id));
                item.unblock_signal(handle);
            }
        }

        gtk::Inhibit(false)
    }

    fn grid_checkbox(&self, port1: &Port, port2: &Port) -> (CheckButton, SignalHandlerId) {
        let button = CheckButton::new();
        button.set_margin_top(5);
        button.set_margin_start(5);
        button.set_margin_bottom(5);
        button.set_margin_end(5);
        let clone = self.jack_controller.clone();
        let id1 = port1.id();
        let id2 = port2.id();
        let signal_id = button.connect_clicked(move |cb| {
            clone.borrow().connect_ports(id1, id2, cb.get_active());
        });
        (button, signal_id)
    }

    fn mixer_checkbox(&self, card_id: i32, channel: u32) -> (CheckButton, SignalHandlerId) {
        let button = CheckButton::new();
        //button.set_active(model.connected_by_id(port1.id(), port2.id()));
        button.set_margin_top(5);
        button.set_margin_start(5);
        button.set_margin_bottom(5);
        button.set_margin_end(5);

        let model = self.state.clone();

        let signal_id = button.connect_clicked(move |cb| {
            model
                .borrow_mut()
                .update(Event::SetMuting(card_id, channel, cb.get_active()));
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

        let model = self.state.clone();
        let chan_id = chan.id;

        let signal = a.connect_value_changed(move |a| {
            model
                .borrow_mut()
                .update(Event::SetVolume(card_id, chan_id, a.get_value() as i64))
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

fn grid_label(text: &str, vertical: bool) -> Label {
    let l = Label::new(Some(text));
    l.set_margin_top(5);
    l.set_margin_start(5);
    l.set_margin_bottom(5);
    l.set_margin_end(5);
    if vertical {
        l.set_angle(90.0);
        l.set_valign(Align::End);
    } else {
        l.set_halign(Align::End);
    }
    l
}

fn mixer_label(text: &str, vertical: bool) -> Label {
    let l = Label::new(Some(text));
    l.set_margin_top(5);
    l.set_margin_start(5);
    l.set_margin_bottom(5);
    l.set_margin_end(5);
    if vertical {
        l.set_angle(90.0);
        l.set_valign(Align::End);
    } else {
        l.set_halign(Align::Center);
    }
    l
}

fn grid() -> Grid {
    let grid = Grid::new();
    grid.set_margin_top(5);
    grid.set_margin_start(5);
    grid.set_margin_bottom(5);
    grid.set_margin_end(5);
    grid.set_valign(Align::Center);
    grid.set_halign(Align::Center);
    grid
}
