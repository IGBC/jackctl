use crate::{
    model::port::{JackPortType, PortDirection},
    settings::{IoOrder, Settings},
    ui::{pages::Pages, utils, UiRuntime},
};
use async_std::sync::RwLock;
use glib::{ObjectExt, ObjectType, SignalHandlerId};
use gtk::{prelude::*, Align, CheckButton, Orientation, Separator};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PortStateElement {
    is_hw: bool,
    port: String,
    id: JackPortType,
}

type PortStateMap = BTreeMap<String, BTreeSet<PortStateElement>>;
type CallbackMap = BTreeMap<(JackPortType, JackPortType), (CheckButton, SignalHandlerId)>;
type Locked<T> = RwLock<T>;

/// Count the number of clients and ports across all clients
///
/// This function exists because we love it very much and value its
/// contribution to the project <3
fn count_map(map: &PortStateMap) -> (usize, usize) {
    (
        map.len(),
        map.iter().fold(0, |acc, (_, set)| acc + set.len()),
    )
}

pub(super) struct Matrix {
    _in: Locked<PortStateMap>,
    out: Locked<PortStateMap>,
    callbacks: Locked<CallbackMap>,
    dirty: AtomicBool,
    rt: UiRuntime,
    page: &'static str,
}

impl Matrix {
    pub fn new(rt: UiRuntime, page: &'static str) -> Self {
        Self {
            _in: Default::default(),
            out: Default::default(),
            dirty: AtomicBool::new(true),
            callbacks: Default::default(),
            rt,
            page,
        }
    }

    /// Add a new port to this audio matrix
    pub async fn add_port(
        &self,
        id: JackPortType,
        dir: PortDirection,
        is_hw: bool,
        client: String,
        port: String,
    ) {
        let mut state = match dir {
            PortDirection::Input => self._in.write().await,
            PortDirection::Output => self.out.write().await,
        };

        state
            .entry(client)
            .or_default()
            .insert(PortStateElement { is_hw, port, id });
        self.dirty.fetch_or(true, Ordering::Relaxed);
    }

    /// Add a connection between two portsg
    pub async fn add_connection(&self, a: JackPortType, b: JackPortType) {
        self.toggle_state(a, b, true).await;
    }

    /// Remove a connection between two ports
    pub async fn rm_connection(&self, a: JackPortType, b: JackPortType) {
        self.toggle_state(a, b, false).await;
    }

    async fn toggle_state(&self, a: JackPortType, b: JackPortType, state: bool) {
        let mut cb_map = self.callbacks.write().await;
        if let Some((btn, cb)) = cb_map.get_mut(&(a, b)) {
            btn.block_signal(cb);
            btn.set_active(state);
            btn.unblock_signal(cb);
        }
    }

    async fn is_empty(&self) -> bool {
        dbg!(self._in.read().await.is_empty()) || dbg!(self.out.read().await.is_empty())
    }

    /// Redraw this widget if it's dirty
    pub async fn draw(&self, settings: &Arc<Settings>, pages: &Pages) {
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }

        let grid = utils::grid();
        if self.is_empty().await {
            let l = utils::grid_label("No ports are currently available", false);
            l.set_halign(Align::Center);
            grid.attach(&l, 0, 0, 1, 1);
        } else {
            // Depending on the app settings assign vertical and
            // horizontal state to inputs and outputs
            let (vert, horz) = match settings.r().app().io_order {
                IoOrder::VerticalInputs => (self._in.read().await, self.out.read().await),
                IoOrder::HorizontalInputs => (self.out.read().await, self._in.read().await),
            };

            let (num_vert_clients, num_vert_ports) = count_map(&vert);
            let (num_horz_clients, num_horz_ports) = count_map(&horz);

            // Number of clients (aka separators) + number of ports +
            // 2 labels (-1 because 0-indexed)
            let max_x: i32 = 2 + num_vert_clients as i32 + num_vert_ports as i32 - 1;
            let max_y: i32 = 2 + num_horz_clients as i32 + num_horz_ports as i32 - 1;

            // Draw vertical labels
            let mut curr_x = 2;
            vert.iter().enumerate().for_each(|(i, (client, set))| {
                let l = utils::grid_label(client, true);
                grid.attach(&l, curr_x, 0, set.len() as i32, 1);

                set.iter().enumerate().for_each(|(curr_x2, entry)| {
                    let l = utils::grid_label(&entry.port, true);
                    grid.attach(&l, curr_x2 as i32 + curr_x, 1, 1, 1);
                });

                if i < num_vert_clients - 1 {
                    grid.attach(&Separator::new(Orientation::Vertical), curr_x, 0, 1, max_y);
                }
                curr_x += set.len() as i32 + 1;
            });

            // Draw horizontal labels
            let mut curr_y = 2;
            horz.iter().enumerate().for_each(|(i, (client, set))| {
                let l = utils::grid_label(client, false);
                grid.attach(&l, 0, curr_y, 1, set.len() as i32);

                set.iter().enumerate().for_each(|(curr_y2, entry)| {
                    let l = utils::grid_label(&entry.port, false);
                    grid.attach(&l, 1, curr_y2 as i32 + curr_y, 1, 1);
                });

                if i < num_vert_clients - 1 {
                    grid.attach(&Separator::new(Orientation::Vertical), 0, curr_x, max_x, 1);
                }
                curr_y += set.len() as i32 + 1;
            });

            // Draw checkboxes
            let mut curr_x = 2;
            let base_x = 2;
            let mut curr_y = 2;
            let mut cb_map = self.callbacks.write().await;

            // Iterate over the horizontal clients list
            horz.iter().enumerate().for_each(|(_i, (client_x, set_x))| {
                // Then iterate over the horizontal ports for the current client
                set_x.iter().for_each(
                    |PortStateElement {
                         id: id_x,
                         port: port_x,
                         ..
                     }| {
                        curr_x = base_x;
                        // For each horizontal port, iterate over the vertical clients list
                        vert.iter().enumerate().for_each(|(_j, (client_y, set_y))| {
                            // Then iterate over the vertical ports of the current client
                            set_y.iter().for_each(
                                |PortStateElement {
                                     id: id_y,
                                     port: port_y,
                                     ..
                                 }| {
                                    let (cb, id) =
                                        utils::grid_checkbox(self.rt.clone(), *id_x, *id_y);
                                    cb.set_tooltip_text(Some(&format!(
                                        "{}:{} x {}:{}",
                                        client_x, port_x, client_y, port_y,
                                    )));

                                    grid.attach(&cb, curr_x, curr_y, 1, 1);
                                    cb_map.insert((*id_x, *id_y), (cb, id));
                                    curr_x += 1;
                                },
                            );

                            // Skip a col because of the divider
                            curr_x += 1;
                        });

                        curr_y += 1;
                    },
                );

                // Skip a row because of the divider
                curr_y += 1;
            });

            drop(cb_map);
        }

        self.dirty.fetch_and(false, Ordering::Relaxed);

        // Do magic things with grid
        pages.insert_scrolled(self.page, &grid);
    }
}
