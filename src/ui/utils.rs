use crate::{
    jack::JackController,
    model::{Port, PortGroup},
};
use glib::{object::IsA, SignalHandlerId};
use gtk::{
    Adjustment, Align, ButtonExt, CheckButton, ContainerExt, Grid, GridExt, Label, LabelExt,
    Orientation, ScrolledWindow, ScrolledWindowExt, Separator, ToggleButtonExt, Viewport,
    ViewportExt, WidgetExt,
};
use std::{cell::RefCell, rc::Rc};

pub(super) fn grid() -> Grid {
    let grid = Grid::new();
    grid.set_margin_top(5);
    grid.set_margin_start(5);
    grid.set_margin_bottom(5);
    grid.set_margin_end(5);
    grid.set_valign(Align::Center);
    grid.set_halign(Align::Center);
    grid
}

pub(super) fn wrap_scroll<P: IsA<gtk::Widget>>(widget: &P) -> ScrolledWindow {
    let vp = Viewport::new::<Adjustment, Adjustment>(None, None);
    vp.add(widget);
    // vp.set_margin_top(10);
    // vp.set_margin_start(10);
    // vp.set_margin_bottom(10);
    // vp.set_margin_end(10);
    vp.set_shadow_type(gtk::ShadowType::Out);
    let sw = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
    sw.add(&vp);
    sw.set_shadow_type(gtk::ShadowType::EtchedIn);
    sw
}

pub(crate) fn grid_label(text: &str, vertical: bool) -> Label {
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
        l.set_justify(gtk::Justification::Right);
        l.set_xalign(1.0);
    }
    l
}

pub(crate) fn mixer_label(text: &str, vertical: bool) -> Label {
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

pub(crate) fn grid_checkbox(
    jack: &Rc<RefCell<JackController>>,
    port1: &Port,
    port2: &Port,
) -> (CheckButton, SignalHandlerId) {
    let button = CheckButton::new();
    button.set_margin_top(5);
    button.set_margin_start(5);
    button.set_margin_bottom(5);
    button.set_margin_end(5);
    let clone = jack.clone();
    let id1 = port1.id();
    let id2 = port2.id();
    let signal_id = button.connect_clicked(move |cb| {
        clone.borrow().connect_ports(id1, id2, cb.get_active());
    });
    (button, signal_id)
}

pub(crate) fn generate_grid(
    jack: &Rc<RefCell<JackController>>,
    inputs: &PortGroup,
    outputs: &PortGroup,
) -> (Grid, Vec<(u32, u32, CheckButton, SignalHandlerId)>) {
    let grid = grid();

    if inputs.is_empty() || outputs.is_empty() {
        let l = &grid_label("No ports are currently available.", false);
        l.set_halign(Align::Center);
        grid.attach(l, 0, 0, 1, 1);

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
            // Don't re-enable this, it causes a spacing bug
            // TODO: Manual Word Wrapping.
            //l.set_line_wrap(true);
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
                        let (cb, handler) = grid_checkbox(&jack, n2, n1);
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
