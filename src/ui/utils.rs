use crate::{model::port::JackPortType, ui::UiRuntime};
use glib::{object::IsA, SignalHandlerId};
use gtk::{
    prelude::BuilderExtManual, Adjustment, Align, Builder, ButtonExt, CheckButton, ContainerExt,
    Grid, Label, LabelExt, ScrolledWindow, ScrolledWindowExt, Viewport, ViewportExt, WidgetExt,
};

pub(super) fn get_object<T>(builder: &Builder, name: &str) -> T
where
    T: gtk::prelude::IsA<glib::object::Object>,
{
    builder
        .get_object(name)
        .expect(&format!("UI file does not contain {}", name))
}

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
    vp.set_shadow_type(gtk::ShadowType::Out);
    let sw = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
    sw.add(&vp);
    sw.set_shadow_type(gtk::ShadowType::EtchedIn);
    sw
}

pub(super) fn grid_label(text: &str, vertical: bool) -> Label {
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

pub(super) fn mixer_label(text: &str, vertical: bool) -> Label {
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

pub(super) fn grid_checkbox(
    _rt: UiRuntime,
    _id1: &JackPortType,
    _id2: &JackPortType,
) -> (CheckButton, SignalHandlerId) {
    let button = CheckButton::new();
    button.set_margin_top(5);
    button.set_margin_start(5);
    button.set_margin_bottom(5);
    button.set_margin_end(5);

    let signal_id = button.connect_clicked(move |cb| {
        // let state = cb.is_active();
        // rt.sender().send(UiEvent::SetConnection(id1, id2, true));
    });
    (button, signal_id)
}
