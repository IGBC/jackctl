//! Jackctl GTK UI module

mod window;

use crate::model2::events::UiEvent;
use async_std::channel::Receiver;

pub struct UiHandle {
    inner: Receiver<UiEvent>,
}

impl UiHandle {
    pub async fn next_event(&self) -> Option<UiEvent> {
        self.inner.recv().await.ok()
    }
}

pub fn create_ui() {
    window::MainWindow {};
}

// mod about;
// mod matrix;
// mod midi;
// mod pages;
// mod utils;

// use about::About;
// use matrix::AudioMatrix;
// use midi::MidiMatrix;
// use pages::Pages;

// use crate::jack::JackController;
// use crate::model::{CardStatus, Event, MixerChannel, Model, ModelInner};
// use gio::prelude::*;
// use glib::signal::SignalHandlerId;
// use gtk::prelude::*;
// use gtk::{
//     Adjustment, Align, Application, Builder, Button, ButtonsType, DialogFlags, Grid, Label,
//     LevelBar, MessageDialog, MessageType, Orientation, PositionType, ResponseType, Scale,
//     ScaleBuilder, Separator, Window,
// };
// use libappindicator::{AppIndicator, AppIndicatorStatus};
// use std::cell::RefCell;
// use std::env;
// use std::path::Path;
// use std::rc::Rc;
// use std::sync::{Arc, Mutex};

// const STYLE: &str = include_str!("../jackctl.css");
// const GLADEFILE: &str = include_str!("../jackctl.glade");

// struct MixerHandle {
//     card_id: i32,
//     element_id: u32,
//     mute: Option<(gtk::ToggleButton, SignalHandlerId)>,
//     volume: (Adjustment, SignalHandlerId),
// }

// pub struct MainDialog {
//     state: Model,
//     jack_controller: Rc<RefCell<JackController>>,

//     builder: Builder,
//     window: Window,
//     xruns_label: Label,
//     xruns_icon: Button,
//     cpu_label: Label,
//     cpu_meter: LevelBar,
//     performance_rate: Label,
//     performance_frames: Label,
//     performance_latency: Label,

//     pages: Pages,

//     audio_matrix: AudioMatrix,
//     midi_matrix: MidiMatrix,

//     mixer_handles: Vec<MixerHandle>,

//     card_dialog: Arc<Mutex<Option<MessageDialog>>>,
// }

// pub fn init_ui(
//     state: Model,
//     jack_controller: Rc<RefCell<JackController>>,
// ) -> (Rc<RefCell<MainDialog>>, Application) {
//     let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

//     // define the gtk application with a unique name and default parameters
//     let application = Application::new(Some("jackctl.segfault"), Default::default())
//         .expect("Initialization failed");

//     //application.set_icon_theme_path(icon_path);

//     let this = MainDialog::new(state, jack_controller);
//     let win_clone = this.clone();
//     application.connect_startup(move |app| {
//         // The CSS "magic" happens here.
//         let provider = gtk::CssProvider::new();
//         provider
//             .load_from_data(STYLE.as_bytes())
//             .expect("Failed to load CSS");
//         // We give the CssProvided to the default screen so the CSS rules we added
//         // can be applied to our window.
//         gtk::StyleContext::add_provider_for_screen(
//             &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
//             &provider,
//             gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
//         );

//         win_clone.borrow_mut().build_ui(&app);
//     });

//     let win_clone = this.clone();

//     let mut indicator = AppIndicator::new("jackctl", "");
//     indicator.set_status(AppIndicatorStatus::Active);
//     indicator.set_icon_theme_path(icon_path.to_str().unwrap());
//     indicator.set_icon_full("jackctl-symbolic", "icon");
//     let mut m = gtk::Menu::new();
//     let mi = gtk::CheckMenuItem::with_label("exit");
//     let app_clone = application.clone();
//     mi.connect_activate(move |_| {
//         MainDialog::quit(&app_clone);
//     });
//     m.append(&mi);
//     let mi = gtk::CheckMenuItem::with_label("show");
//     mi.connect_activate(move |_| {
//         win_clone.borrow().show();
//     });
//     m.append(&mi);
//     indicator.set_menu(&mut m);
//     m.show_all();

//     (this, application)
// }

// impl MainDialog {
//     pub fn new(state: Model, jack_controller: Rc<RefCell<JackController>>) -> Rc<RefCell<Self>> {
//         // this builder provides access to all components of the defined ui
//         let builder = Builder::from_string(GLADEFILE);

//         // Initialise the state:

//         // find the main dialog
//         let window: Window = utils::get_object(&builder, "maindialog");

//         // hook up the minimise button
//         let minimise: Button = utils::get_object(&builder, "minimise.maindialog");
//         let window_clone = window.clone();
//         minimise.connect_clicked(move |_| window_clone.hide());

//         // Setup xruns display
//         let xruns_label: Label = utils::get_object(&builder, "label.xruns.maindialog");
//         xruns_label.set_markup(&format!("{} XRuns", "N.D."));
//         let xruns_icon: Button = utils::get_object(&builder, "button.xruns.maindialog");
//         let state_clone = state.clone();
//         xruns_icon.connect_clicked(move |icon| {
//             state_clone
//                 .lock()
//                 .unwrap()
//                 .get_pipe()
//                 .send(Event::ResetXruns)
//                 .unwrap();
//             icon.hide();
//         });

//         // Setup CPU Meter
//         let cpu_label: Label = utils::get_object(&builder, "label.cpu.maindialog");
//         let cpu_meter: LevelBar = utils::get_object(&builder, "meter.cpu.maindialog");

//         // Setup Time status display
//         let performance_rate = utils::get_object(&builder, "samplerate.performance.maindialog");
//         let performance_frames = utils::get_object(&builder, "wordsize.performance.maindialog");
//         let performance_latency = utils::get_object(&builder, "latency.performance.maindialog");

//         // Setup page view abstraction
//         let pages = Pages::new(&builder, vec!["Matrix", "MIDI", "Mixer", "Tools"]);

//         // Setup about screen
//         About::new(&builder).button(&builder);

//         // Save the bits we need
//         let this = Rc::new(RefCell::new(MainDialog {
//             state,
//             jack_controller,
//             builder: builder.clone(),
//             window: window.clone(),
//             xruns_label,
//             xruns_icon,
//             cpu_label,
//             cpu_meter,
//             performance_rate,
//             performance_frames,
//             performance_latency,
//             pages,
//             audio_matrix: AudioMatrix::new(),
//             midi_matrix: MidiMatrix::new(),
//             mixer_handles: Vec::new(),
//             card_dialog: Arc::new(Mutex::new(None)),
//         }));

//         // hookup the update function
//         let this_clone = this.clone();

//         window.connect_draw(move |_, _| this_clone.borrow_mut().update_ui());

//         this
//     }

//     pub fn quit(app: &Application) {
//         eprintln!("Shutting down GTK");
//         app.quit();
//     }

//     pub fn build_ui(&mut self, app: &Application) {
//         eprintln!("Setting Window Application");
//         self.window.set_application(Some(app));

//         // Setup Main Menu
//         let quit: gtk::ModelButton = utils::get_object(&self.builder, "quit.mainmenu");
//         let app_clone = app.clone();
//         quit.connect_clicked(move |_| Self::quit(&app_clone));
//     }

//     pub fn show(&self) {
//         self.window.show_all();
//         self.window.present();
//     }

//     fn update_mixer(&self, model: &ModelInner) -> (Grid, Vec<MixerHandle>) {
//         let grid = utils::grid();
//         let mut handles = Vec::new();
//         grid.set_hexpand(true);
//         grid.set_vexpand(true);
//         if model.cards.is_empty() {
//             grid.attach(
//                 &utils::mixer_label("No controllable devices are detected.", false),
//                 0,
//                 0,
//                 1,
//                 1,
//             );
//             return (grid, handles);
//         }

//         let mut x_pos = 0;
//         // get the elements in order.
//         let mut keys: Vec<&i32> = model.cards.keys().collect();
//         keys.sort();
//         for card in keys
//             .iter()
//             .map(|k| model.cards.get(*k).unwrap())
//             .filter(|x| x.state == CardStatus::Active)
//         {
//             let len = card.len();
//             if len == 0 {
//                 grid.attach(
//                     &utils::mixer_label(card.name(), false),
//                     x_pos as i32,
//                     3,
//                     1,
//                     1,
//                 );
//                 grid.attach(
//                     &utils::mixer_label("Device Has No Controls", true),
//                     x_pos as i32,
//                     0,
//                     1,
//                     2,
//                 );
//                 x_pos += 1;
//             } else {
//                 grid.attach(
//                     &utils::mixer_label(card.name(), false),
//                     x_pos,
//                     3,
//                     len as i32,
//                     1,
//                 );

//                 // get the card in order, for consistency with things like alsamixer.
//                 let mut keys: Vec<&MixerChannel> = card.iter().collect();
//                 keys.sort_by(|a, b| a.id.cmp(&b.id));

//                 for channel in keys {
//                     grid.attach(
//                         &utils::mixer_label(channel.get_name(), true),
//                         x_pos,
//                         0,
//                         1,
//                         1,
//                     );

//                     let (scale, adjustment, scale_signal) = self.mixer_fader(card.id, channel);
//                     grid.attach(&scale, x_pos, 1, 1, 1);

//                     let cb_signal = if channel.has_switch {
//                         let (cb, cb_signal) =
//                             self.mixer_checkbox(card.id, channel.id, channel.is_playback);
//                         grid.attach(&cb, x_pos, 2, 1, 1);
//                         Some((cb, cb_signal))
//                     } else {
//                         None
//                     };
//                     x_pos += 1;

//                     let handle = MixerHandle {
//                         card_id: card.id,
//                         element_id: channel.id,
//                         mute: cb_signal,
//                         volume: (adjustment, scale_signal),
//                     };
//                     handles.push(handle);
//                 }
//             }
//             grid.attach(&Separator::new(Orientation::Vertical), x_pos, 0, 1, 4);
//             x_pos += 1;
//         }

//         (grid, handles)
//     }

//     pub fn update_ui(&mut self) -> gtk::Inhibit {
//         let mut model = self.state.lock().unwrap();
//         self.xruns_label
//             .set_markup(&format!("{} XRuns", model.xruns()));
//         if model.xruns() == 0 {
//             self.xruns_icon.hide();
//         } else {
//             self.xruns_icon.show_all();
//         }

//         self.cpu_label
//             .set_markup(&format!("{}%", model.cpu_percent.trunc()));
//         self.cpu_meter.set_value(model.cpu_percent as f64);

//         self.performance_rate
//             .set_markup(&format!("{}Hz", model.sample_rate));
//         self.performance_frames
//             .set_markup(&format!("{}w", model.buffer_size));
//         self.performance_latency
//             .set_markup(&format!("{}ms", model.latency));

//         if model.layout_dirty {
//             model.layout_dirty = false;
//             let page = self.pages.get_current();

//             // update Audio Matrix Tab
//             self.audio_matrix.update(
//                 &mut self.pages,
//                 &self.jack_controller,
//                 model.audio_outputs(),
//                 model.audio_inputs(),
//             );

//             self.midi_matrix.update(
//                 &mut self.pages,
//                 &self.jack_controller,
//                 model.midi_outputs(),
//                 model.midi_inputs(),
//             );

//             // update Mixer Tab
//             let (mixer_matrix, cb_vec) = self.update_mixer(&model);
//             self.pages.remove_page("Mixer");
//             self.pages.insert_horizontal("Mixer", &mixer_matrix);
//             self.mixer_handles = cb_vec;

//             self.pages.show_all();
//             self.pages.set_current(page);
//         }

//         for (i, j, item, handle) in self.audio_matrix.iter() {
//             item.block_signal(handle);
//             item.set_active(model.connected_by_id(*j, *i));
//             item.unblock_signal(handle);
//         }

//         for (j, i, item, handle) in self.midi_matrix.iter() {
//             item.block_signal(handle);
//             item.set_active(model.connected_by_id(*j, *i));
//             item.unblock_signal(handle);
//         }

//         for element in self.mixer_handles.iter() {
//             let (item, handle) = &element.volume;
//             item.block_signal(handle);
//             item.set_value(model.get_volume(element.card_id, element.element_id) as f64);
//             item.unblock_signal(handle);
//             if element.mute.is_some() {
//                 let (item, handle) = &element.mute.as_ref().unwrap();
//                 item.block_signal(handle);
//                 item.set_active(model.get_muting(element.card_id, element.element_id));
//                 item.unblock_signal(handle);
//             }
//         }

//         for card in model.cards.values() {
//             if card.state == CardStatus::New {
//                 if self.card_dialog.lock().unwrap().is_none() {
//                     eprintln!("asking for card {}", card.id);
//                     let dialog = MessageDialog::new(
//                         Some(&self.window),
//                         DialogFlags::empty(),
//                         MessageType::Question,
//                         ButtonsType::YesNo,
//                         &format!("Use Card \"{}\"?", card.name()),
//                     );

//                     let model = self.state.clone();
//                     let dialog_ref = self.card_dialog.clone();
//                     let id_clone = card.id.clone();

//                     dialog.connect_response(move |dialog, response| {
//                         eprintln!("response recieved");
//                         match response {
//                             ResponseType::Yes => {
//                                 model
//                                     .lock()
//                                     .unwrap()
//                                     .get_pipe()
//                                     .send(Event::UseCard(id_clone))
//                                     .unwrap();
//                             }
//                             ResponseType::No => {
//                                 model
//                                     .lock()
//                                     .unwrap()
//                                     .get_pipe()
//                                     .send(Event::DontUseCard(id_clone))
//                                     .unwrap();
//                             }
//                             _ => panic!("Unexpected Message Response"),
//                         }
//                         dialog.hide();
//                         let _ = dialog_ref.lock().unwrap().take();
//                     });

//                     dialog.set_modal(true);
//                     dialog.show_all();

//                     self.card_dialog.lock().unwrap().replace(dialog);
//                 }
//             }
//         }

//         gtk::Inhibit(false)
//     }

//     fn mixer_checkbox(
//         &self,
//         card_id: i32,
//         channel: u32,
//         output: bool,
//     ) -> (gtk::ToggleButton, SignalHandlerId) {
//         let builder = gtk::ToggleButtonBuilder::new();
//         let image = if output {
//             gtk::Image::from_icon_name(Some("audio-volume-muted-symbolic"), gtk::IconSize::Button)
//         } else {
//             gtk::Image::from_icon_name(
//                 Some("microphone-sensitivity-muted-symbolic"),
//                 gtk::IconSize::Button,
//             )
//         };
//         let button = builder
//             .image(&image)
//             .always_show_image(true)
//             .image_position(gtk::PositionType::Bottom)
//             .build();
//         //button.set_active(model.connected_by_id(port1.id(), port2.id()));
//         button.set_margin_top(5);
//         button.set_margin_start(5);
//         button.set_margin_bottom(5);
//         button.set_margin_end(5);
//         button.set_halign(Align::Center);

//         let model = self.state.clone();

//         let signal_id = button.connect_clicked(move |cb| {
//             model
//                 .lock()
//                 .unwrap()
//                 .get_pipe()
//                 .send(Event::SetMuting(card_id, channel, cb.get_active()))
//                 .unwrap();
//         });
//         (button, signal_id)
//     }

//     fn mixer_fader(
//         &self,
//         card_id: i32,
//         chan: &MixerChannel,
//     ) -> (Scale, Adjustment, SignalHandlerId) {
//         let a = Adjustment::new(
//             0.0,
//             chan.volume_min as f64,
//             chan.volume_max as f64,
//             1.0,
//             10.0,
//             0.0,
//         );

//         let model = self.state.clone();
//         let chan_id = chan.id;

//         let signal = a.connect_value_changed(move |a| {
//             model
//                 .lock()
//                 .unwrap()
//                 .get_pipe()
//                 .send(Event::SetVolume(card_id, chan_id, a.get_value() as i64))
//                 .unwrap()
//         });

//         let s = ScaleBuilder::new()
//             .adjustment(&a)
//             .orientation(Orientation::Vertical)
//             .value_pos(PositionType::Bottom)
//             .inverted(true)
//             .hexpand(true)
//             .height_request(200)
//             .digits(0)
//             .build();
//         s.set_value_pos(PositionType::Bottom);
//         (s, a, signal)
//     }
// }
