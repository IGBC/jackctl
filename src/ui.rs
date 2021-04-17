use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{Window, Label, Image, Button, LevelBar, Notebook, Grid, CheckButton, Align};
use gtk::Builder;
use gtk::Application;

use crate::model::Model;
use crate::engine::Controller;

pub struct MainDialog {
    state: Model,
    controller: Rc<RefCell<Controller>>,

    builder: Builder,
    window:  Window,
    xruns_label: Label,
    xruns_icon: Image,
    cpu_label: Label,
    cpu_meter: LevelBar,
    performance_rate: Label,
    performance_frames: Label,
    performance_latency: Label,
    tabs: Notebook,
    matrix_tab: gtk::Box,
}

fn get_object<T>(builder: &Builder, name: &str) -> T 
where T: gtk::prelude::IsA<glib::object::Object> {
    let o: T = builder.get_object(name).expect(&format!("UI file does not contain {}", name));
    o
}

pub fn init_ui(state: Model, controller: Rc<RefCell<Controller>>) -> Rc<RefCell<MainDialog>> {
    // define the gtk application with a unique name and default parameters
    let application = Application::new(Some("The.name.goes.here"), Default::default())
    .expect("Initialization failed");

    // this registers a closure (executing our setup_gui function) 
    //that has to be run on a `activate` event, triggered when the UI is loaded
    //application.connect_activate(move |app| {
    //
    let glade_src = include_str!("jackctl.glade");
  
    // this builder provides access to all components of the defined ui
    let builder = Builder::from_string(glade_src);
    
    MainDialog::new(builder, state, controller)
}

impl MainDialog {
    pub fn new(builder: Builder, state: Model, controller: Rc<RefCell<Controller>>) -> Rc<RefCell<Self>> {
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
        let matrix_tab = get_object(&builder, "matrix.tabs.maindialog");

        // Save the bits we need
        let this = Rc::new(RefCell::new(MainDialog {
            state,
            controller,
            builder,
            window: window.clone(),
            xruns_label,
            xruns_icon,
            cpu_label,
            cpu_meter,
            performance_rate,
            performance_frames,
            performance_latency,
            tabs,
            matrix_tab,
        }));

        // hookup the update function
        let this_clone = this.clone();
        window.connect_draw(move |_,_|{this_clone.borrow().update_ui()});
        
        this
    }

    pub fn show(&self) {
        self.window.show_all();
    }

    pub fn update_ui(&self) -> gtk::Inhibit {
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

            if model.inputs().is_empty() || model.outputs().is_empty() {
                let l = grid_label("No Jack audio ports are currently available.", false);
                l.set_halign(Align::Center);
                self.tabs.remove_page(Some(0));
                self.tabs.insert_page(&l, Some(&Label::new(Some("Matrix"))), Some(0));
            } else {
                let grid = grid();
                let mut curr_x = 2;
                for g in model.inputs().iter() {
                    grid.attach(&grid_label(g.name(), true), curr_x, 0, g.len() as i32, 1);
                    
                    for n in g.iter() {
                        grid.attach(&grid_label(n, true), curr_x, 1, 1, 1);
                        curr_x += 1;
                    }
                }
    
                let mut curr_y = 2;
                for g in model.outputs().iter() {
                    grid.attach(&grid_label(g.name(), false), 0, curr_y, 1, g.len() as i32);
                    
                    for n in g.iter() {
                        grid.attach(&grid_label(n, false), 1, curr_y, 1, 1);
                        curr_y += 1;
                    }
                }

                let mut curr_x = 2;
                for g2 in model.inputs().iter() {
                    for n2 in g2.iter() {
                        let mut curr_y = 2;
                        for g1 in model.outputs().iter() {
                            for n1 in g1.iter() {
                                
                                let cb = self.grid_checkbox(g1.name(), n1, g2.name(), n2);
                                grid.attach(&cb, curr_x, curr_y, 1, 1);
                                
                                curr_y += 1;
                            }
                        }
                        curr_x += 1;
                    }
                }                
    
                self.tabs.remove_page(Some(0));
                self.tabs.insert_page(&grid, Some(&Label::new(Some("Matrix"))), Some(0));    
            }

            self.tabs.show_all();
            
            self.tabs.set_current_page(page);

        }

        gtk::Inhibit(false)
    }

    fn grid_checkbox(&self, group1: &str, port1: &str, group2: &str, port2: &str) -> CheckButton {
        let button = CheckButton::new();
        button.set_margin_top(5);
        button.set_margin_start(5);
        button.set_margin_bottom(5);
        button.set_margin_end(5);
        let clone = self.controller.clone();
        let group1_c = group1.to_owned();
        let group2_c = group2.to_owned();
        let port1_c = port1.to_owned();
        let port2_c = port2.to_owned();
        button.connect_toggled( move | cb | cb.set_active(clone.borrow().connect_ports("", "", "", "", cb.get_active())));
        button   
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