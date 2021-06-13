use std::rc::Rc;
use std::cell::RefCell;
use std::env;
use std::path::Path;

use gtk::prelude::*;
use gtk::{Window, Label, Image, Button, LevelBar, Notebook, Grid, CheckButton, Align, Separator, Orientation};
use gtk::Builder;
use gtk::Application;
use glib::signal::SignalHandlerId;

use crate::model::{Model, Port, PortGroup};
use crate::jack::JackController;

use libappindicator::{AppIndicator, AppIndicatorStatus};

pub struct MainDialog {
    state: Model,
    controller: Rc<RefCell<JackController>>,

    //builder: Builder,
    window:  Window,
    xruns_label: Label,
    xruns_icon: Image,
    cpu_label: Label,
    cpu_meter: LevelBar,
    performance_rate: Label,
    performance_frames: Label,
    performance_latency: Label,
    tabs: Notebook,
    
    audio_matrix: Vec<Vec<(CheckButton, SignalHandlerId)>>,
    midi_matrix: Vec<Vec<(CheckButton, SignalHandlerId)>>,
}

fn get_object<T>(builder: &Builder, name: &str) -> T 
where T: gtk::prelude::IsA<glib::object::Object> {
    let o: T = builder.get_object(name).expect(&format!("UI file does not contain {}", name));
    o
}

pub fn init_ui(state: Model, controller: Rc<RefCell<JackController>>) -> Rc<RefCell<MainDialog>> {
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
    
    let this = MainDialog::new(builder, state, controller);
    
    let win_clone = this.clone();

    let mut indicator = AppIndicator::new("jackctl", "");
    indicator.set_status(AppIndicatorStatus::Active);
    let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    indicator.set_icon_theme_path(icon_path.to_str().unwrap());
    indicator.set_icon_full("rust-logo-64x64-blk", "icon");
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
    pub fn new(builder: Builder, state: Model, controller: Rc<RefCell<JackController>>) -> Rc<RefCell<Self>> {
        // Initialise the state:

        // find the main dialog
        let window: Window = get_object(&builder, "maindialog");
        
        // hook up the minimise button
        let minimise: Button = get_object(&builder, "minimise.maindialog");
        let window_clone = window.clone();
        minimise.connect_clicked(move |_| {window_clone.hide()});

        // Setup xruns display
        let xruns_label: Label = get_object(&builder, "label.xruns.maindialog");
        xruns_label.set_markup(&format!("{} XRuns", "N.D."));
        let xruns_icon: Image = get_object(&builder, "icon.xruns.maindialog");
        xruns_icon.set_from_icon_name(Some("dialog-warning-symbolic"), gtk::IconSize::Button);

        // Setup CPU Meter
        let cpu_label: Label = get_object(&builder, "label.cpu.maindialog");
        let cpu_meter: LevelBar = get_object(&builder, "meter.cpu.maindialog");

        // Setup Time status display
        let performance_rate =    get_object(&builder, "samplerate.performance.maindialog");
        let performance_frames =  get_object(&builder, "wordsize.performance.maindialog");
        let performance_latency = get_object(&builder, "latency.performance.maindialog");

        // Setup notebook view
        let tabs = get_object(&builder, "tabs.maindialog");
        
        // Setup about screen
        let about:Window = get_object(&builder, "aboutdialog");

        // Setup Main Menu
        let quit: gtk::ModelButton = get_object(&builder, "quit.mainmenu");
        quit.connect_clicked(|_| gtk::main_quit());
        let aboutbutton: gtk::ModelButton = get_object(&builder, "about.mainmenu");
        aboutbutton.connect_clicked( move |_| about.show_all());


        // Save the bits we need
        let this = Rc::new(RefCell::new(MainDialog {
            state,
            controller,
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
        }));

        // hookup the update function
        let this_clone = this.clone();
        window.connect_draw(move |_,_|{this_clone.borrow_mut().update_ui()});
        
        this
    }

    pub fn show(&self) {
        self.window.show_all();
        self.window.present();
    }

    fn update_matrix(&self, inputs: &PortGroup, outputs: &PortGroup) -> (Grid, Vec<Vec<(CheckButton, SignalHandlerId)>>) {
        let grid = grid();
        
        if inputs.is_empty() || outputs.is_empty() {
        
            let l = grid_label("No ports are currently available.", false);
            l.set_halign(Align::Center);
            grid.attach(&l,0,0,1,1);

            (grid,Vec::new())

        } else {
            
            let i_groups = inputs.no_groups();
            let o_groups = outputs.no_groups();
            let n_audio_inputs = inputs.len();
            let n_audio_outputs = outputs.len();
            let max_x: i32 = 2 + i_groups as i32 + n_audio_inputs as i32 - 1;
            let max_y: i32 = 2 + o_groups as i32 + n_audio_outputs as i32 - 1;

            let mut curr_x = 2;
            for (i, g) in inputs.iter().enumerate() {
                grid.attach(&grid_label(g.name(), true), curr_x, 0, g.len() as i32, 1);
                
                for n in g.iter() {
                    grid.attach(&grid_label(n.name(), true), curr_x, 1, 1, 1);
                    curr_x += 1;
                }

                if i < i_groups -1 {
                    grid.attach(&Separator::new(Orientation::Vertical), curr_x, 0, 1, max_y);
                    curr_x += 1;
                }
            }

            let mut curr_y = 2;
            for (i, g) in outputs.iter().enumerate() {
                grid.attach(&grid_label(g.name(), false), 0, curr_y, 1, g.len() as i32);
                
                for n in g.iter() {
                    grid.attach(&grid_label(n.name(), false), 1, curr_y, 1, 1);
                    curr_y += 1;
                }

                if i < o_groups -1 {
                    grid.attach(&Separator::new(Orientation::Horizontal), 0, curr_y, max_x, 1);
                    curr_y += 1;
                }
            }

            let mut curr_x = 2;
            let mut x_vec = Vec::new();
            for g2 in inputs.iter() {
                for n2 in g2.iter() {
                    let mut curr_y = 2;
                    let mut y_vec = Vec::new();
                    for g1 in outputs.iter() {
                        for n1 in g1.iter() {
                            
                            let (cb, handler) = self.grid_checkbox(n1, n2);
                            grid.attach(&cb, curr_x, curr_y, 1, 1);
                            
                            y_vec.push((cb, handler));
                            curr_y += 1;
                        }
                        // skip over the separator;
                        curr_y += 1;
                    }
                    x_vec.push(y_vec);
                    curr_x += 1;
                }
                // skip over the separator
                curr_x += 1;
            }       
            
            (grid, x_vec)    
        }
    }

    pub fn update_ui(&mut self) -> gtk::Inhibit {
        let mut model = self.state.borrow_mut();
        self.xruns_label.set_markup(&format!("{} XRuns", model.xruns()));
        
        self.cpu_label.set_markup(&format!("{}%", model.cpu_percent.trunc()));
        self.cpu_meter.set_value(model.cpu_percent as f64);
        
        self.performance_rate.set_markup(&format!("{}Hz", model.sample_rate));
        self.performance_frames.set_markup(&format!("{}w", model.buffer_size));
        self.performance_latency.set_markup(&format!("{}ms", model.latency));

        if model.layout_dirty {
            model.layout_dirty = false;
            let page = self.tabs.get_current_page();

            // update Audio Matrix Tab
            let (audio_matrix, cb_vec) = self.update_matrix(model.audio_inputs(), model.audio_outputs());
            self.tabs.remove_page(Some(0));
            self.tabs.insert_page(&audio_matrix, Some(&Label::new(Some("Matrix"))), Some(0));
            self.audio_matrix = cb_vec;

            // update Midi Matrix Tab
            let (midi_matrix, cb_vec) = self.update_matrix(model.midi_inputs(), model.midi_outputs());
            self.tabs.remove_page(Some(1));
            self.tabs.insert_page(&midi_matrix, Some(&Label::new(Some("MIDI"))), Some(1));
            self.midi_matrix = cb_vec;

            self.tabs.show_all();
            
            self.tabs.set_current_page(page);

        }

        for (i, col) in self.audio_matrix.iter().enumerate() {
            for (j, (item, handle)) in col.iter().enumerate() {
                item.block_signal(handle);
                item.set_active(model.connected_by_id(j, i));
                item.unblock_signal(handle); 
            }
        } 

        for (i, col) in self.midi_matrix.iter().enumerate() {
            for (j, (item, handle)) in col.iter().enumerate() {
                item.block_signal(handle);
                item.set_active(model.connected_by_id(j+1000, i+1000));
                item.unblock_signal(handle); 
            }
        }

        gtk::Inhibit(false)
    }

    fn grid_checkbox(&self, port1: &Port, port2: &Port, /*model: &ModelInner*/) -> (CheckButton, SignalHandlerId) {
        let button = CheckButton::new();
        //button.set_active(model.connected_by_id(port1.id(), port2.id()));
        button.set_margin_top(5);
        button.set_margin_start(5);
        button.set_margin_bottom(5);
        button.set_margin_end(5);
        let clone = self.controller.clone();
        let id1 = port1.id();
        let id2 = port2.id();
        let signal_id = button.connect_clicked( move | cb | { 
            clone.borrow().connect_ports(id1, id2, cb.get_active()); 
        });
        (button, signal_id)   
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