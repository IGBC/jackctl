use crate::{
    model2::port::{JackPortType, PortDirection},
    ui::{pages::Pages, utils},
};
use async_std::sync::RwLock;
use gtk::{prelude::*, Align, Orientation, Separator};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PortStateElement {
    is_hw: bool,
    port: String,
    id: JackPortType,
}

type PortStateMap = BTreeMap<String, BTreeSet<PortStateElement>>;
type PortState = RwLock<PortStateMap>;

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

pub struct AudioMatrix {
    _in: PortState,
    out: PortState,
    dirty: AtomicBool,
}

impl AudioMatrix {
    pub fn new() -> Self {
        Self {
            _in: Default::default(),
            out: Default::default(),
            dirty: AtomicBool::new(true),
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

    async fn is_empty(&self) -> bool {
        dbg!(self._in.read().await.is_empty()) || dbg!(self.out.read().await.is_empty())
    }

    /// Redraw this widget if it's dirty
    pub async fn draw(&self, pages: &Pages) {
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }

        // println!("Inputs: {:#?}", self._in.read().await);
        // println!("Outputs: {:#?}", self.out.read().await);

        let grid = utils::grid();
        if self.is_empty().await {
            let l = utils::grid_label("No ports are currently available", false);
            l.set_halign(Align::Center);
            grid.attach(&l, 0, 0, 1, 1);
        } else {
            let _in = self._in.read().await;
            let (num_in_clients, num_in_ports) = count_map(&_in);

            let out = self.out.read().await;
            let (num_out_clients, num_out_ports) = count_map(&out);

            // Number of clients (aka separators) + number of ports +
            // 2 labels (-1 because 0-indexed)
            let max_x: i32 = 2 + num_in_clients as i32 + num_in_ports as i32 - 1;
            let max_y: i32 = 2 + num_out_clients as i32 + num_out_ports as i32 - 1;

            // Draw input labels
            let mut curr_x = 1;
            _in.iter().enumerate().for_each(|(i, (client, set))| {
                let l = utils::grid_label(client, true);
                grid.attach(&l, curr_x, 0, set.len() as i32, 1);

                set.iter().enumerate().for_each(|(curr_x2, entry)| {
                    let l = utils::grid_label(&entry.port, true);
                    grid.attach(&l, curr_x2 as i32 + curr_x, 1, 1, 1);
                });

                if i < num_in_clients - 1 {
                    grid.attach(&Separator::new(Orientation::Vertical), curr_x, 0, 1, max_y);
                }
                curr_x += set.len() as i32 + 1;
            });

            // Draw output labels
            let mut curr_y = 1;
            out.iter().enumerate().for_each(|(i, (client, set))| {
                let l = utils::grid_label(client, false);
                grid.attach(&l, 0, curr_y, 1, set.len() as i32);

                set.iter().enumerate().for_each(|(curr_y2, entry)| {
                    let l = utils::grid_label(&entry.port, false);
                    grid.attach(&l, 1, curr_y2 as i32 + curr_y, 1, 1);
                });

                if i < num_in_clients - 1 {
                    grid.attach(&Separator::new(Orientation::Vertical), 0, curr_x, max_x, 1);
                }
                curr_y += set.len() as i32 + 1;
            });
        }

        self.dirty.fetch_and(false, Ordering::Relaxed);

        // Do magic things with grid
        pages.insert_scrolled("Matrix", dbg!(&grid));
    }

    // This function updates the matrix based on the current model
    // pub fn update(
    //     &mut self,
    //     pages: &mut Pages,
    //     inputs: &PortGroup,
    //     outputs: &PortGroup,
    // ) {
    //     let (grid, callbacks) = utils::generate_grid(jack, inputs, outputs);
    //     pages.remove_page("Matrix");
    //     pages.insert_scrolled("Matrix", &grid);
    //     self.inner = callbacks;
    // }
}
