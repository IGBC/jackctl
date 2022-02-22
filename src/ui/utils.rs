use crate::{
    model::{events::UiEvent, port::JackPortType},
    ui::UiRuntime,
};
use glib::{object::IsA, SignalHandlerId};
use gtk::{
    prelude::*, Adjustment, Align, Application, Box, BoxExt, Builder, ButtonExt, CheckButton,
    ContainerExt, Dialog, DialogBuilder, DialogExt, DialogFlags, Grid, Label, LabelExt,
    Orientation, ResponseType, ScrolledWindow, ScrolledWindowExt, ToggleButtonExt, Viewport,
    ViewportExt, Widget, WidgetExt, Window,
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
    rt: UiRuntime,
    id1: JackPortType,
    id2: JackPortType,
) -> (CheckButton, SignalHandlerId) {
    let button = CheckButton::new();
    button.set_margin_top(5);
    button.set_margin_start(5);
    button.set_margin_bottom(5);
    button.set_margin_end(5);

    let signal_id = button.connect_clicked(move |cb| {
        let state = cb.get_active();
        rt.sender().send(UiEvent::SetConnection(id1, id2, state));
    });
    (button, signal_id)
}

pub(super) fn margin<P: IsA<Widget>>(widget: &P, margin: i32) {
    widget.set_margin_top(margin);
    widget.set_margin_start(margin);
    widget.set_margin_bottom(margin);
    widget.set_margin_end(margin);
}

pub(super) fn yes_no_dialog(
    app: &Application,
    parent: &Window,
) -> (Dialog, Label, Label, CheckButton) {
    let this = Dialog::with_buttons(
        Some("If you can read this, something broke :)"),
        Some(parent),
        DialogFlags::all(),
        &[("Yes", ResponseType::Yes), ("No", ResponseType::No)],
    );
    app.add_window(&this);
    this.set_modal(true);
    this.set_default_response(ResponseType::Yes);

    let vbox = this.get_content_area();
    vbox.set_orientation(Orientation::Vertical);
    vbox.set_margin_start(15);
    vbox.set_margin_end(15);
    vbox.set_margin_top(15);
    vbox.set_margin_bottom(5);
    vbox.set_spacing(5);

    let (l1, l2, cb) = card_query(&vbox);

    this.add(&vbox);
    this.resize(380, 235);
    (this, l1, l2, cb)
}

pub(super) fn card_query(vbox: &Box) -> (Label, Label, CheckButton) {
    let check = CheckButton::with_label("Remember my choice for this device");
    let label1 = Label::new(Some("If you can read this, something broke :)"));
    let label2 = Label::new(Some(
        // TODO: investigate text wrapping
        "Activating this device will add it to the JACK connection graph \
         for use with other JACK clients. Only one sound system may use the device at a time so it \
         will become unavailable to non JACK applications",
    ));
    label2.set_line_wrap(true);

    vbox.pack_start(&label1, true, false, 0);
    vbox.pack_start(&label2, true, false, 0);
    vbox.pack_start(&check, true, false, 0);

    (label1, label2, check)
}
