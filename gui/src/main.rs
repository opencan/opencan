#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::{process::exit, sync::mpsc};

use anyhow::Result;
use eframe::{
    egui::{self},
    Theme,
};
use interface_picker::InterfacePicker;
use motorsports::Motorsports;
use opencan_core::CANNetwork;
use perf_panel::PerfPanel;
use pycanrs::{PyCanInterface, PyCanMessage};

mod decode;
mod interface_picker;
mod motorsports;
mod perf_panel;
// mod rx_area;
mod status_bar;

struct Interface {
    rx_channel: mpsc::Receiver<PyCanMessage>,
    interface: PyCanInterface,
}

struct Gui {
    interface: Option<Interface>,

    interface_picker: InterfacePicker,

    /// Message ID -> last data
    // message_history: BTreeMap<u32, (CANMessage, RecievedMessage)>,

    // row_heights: Vec<f32>,
    network: Option<CANNetwork>,

    perf_panel: PerfPanel,

    motorsports: Motorsports,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            interface: None,
            interface_picker: Default::default(),
            // message_history: BTreeMap::new(),
            // row_heights: Vec::new(),
            network: None,
            perf_panel: Default::default(),
            motorsports: Default::default(),
        }
    }

    pub fn assign_interface(&mut self, pycan_iface: PyCanInterface) -> Result<()> {
        let (tx, rx) = mpsc::channel();

        // Set up CAN listener
        let message_cb = move |msg: &PyCanMessage| tx.send(msg.clone()).unwrap();
        let error_cb = |_: &_| {
            eprintln!("Error from pycanrs!");
            std::process::exit(-1);
        };

        pycan_iface.register_rx_callback(message_cb, error_cb)?;

        self.interface = Some(Interface {
            rx_channel: rx,
            interface: pycan_iface,
        });

        Ok(())
    }
}

// struct RecievedMessage {
//     pymsg: PyCanMessage,
//     count: u32,
//     last_timestamp: f64,
//     cur_timestamp: f64,
// }

fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    ctrlc::set_handler(|| {
        eprintln!("Caught ctrl-c. Bye!");
        exit(0);
    })?;

    let mut gui = Gui::new();
    gui.network = Some(
        opencan_compose::compose(opencan_compose::Args {
            in_file: "/Users/dmezh/motorsports/can/can.yml".into(),
            dump_json: false,
            dump_python: false,
        })
        .unwrap(),
    );

    let options = eframe::NativeOptions {
        resizable: false,
        follow_system_theme: false,
        default_theme: Theme::Light,
        ..Default::default()
    };

    let bus = pycanrs::PyCanBusType::Socketcand {
        host: "side".into(),
        port: 30001,
        channel: "vcan0".into(),
    };

    let pycan_iface = pycanrs::PyCanInterface::new(bus).unwrap();

    gui.assign_interface(pycan_iface).unwrap();

    eframe::run_native("Juice", options, Box::new(|_cc| Box::new(gui))).unwrap();

    Ok(())
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.maybe_service_rx();

        self.perf_panel.maybe_show(ctx, frame);

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(1.);
            self.status_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.interface.is_none() {
                ui.label("No interface connected");
                self.interface_picker(ui);
            } else {
                self.motorsports.motorsports_gooey(ui);
            }
        });

        frame.set_window_size(ctx.used_size());
    }
}

impl Gui {
    fn maybe_service_rx(&mut self) {
        let Some(iface) = &self.interface else {
            return;
        };

        // drain messages from channel
        // todo: performance pinch point. we should probably not do this in the egui update loop.
        while let Ok(msg) = iface.rx_channel.try_recv() {
            // try to update existing message in history, else insert new one
            // if let Some((_, prev)) = self.message_history.get_mut(&msg.arbitration_id) {
            //     *prev = RecievedMessage {
            //         count: prev.count + 1,
            //         last_timestamp: prev.cur_timestamp,
            //         cur_timestamp: msg.timestamp.unwrap(),
            //         pymsg: msg,
            //     };
            // } else if let Some(oc) = self.message_id_to_opencan(msg.arbitration_id) {
            //     self.message_history.insert(
            //         msg.arbitration_id,
            //         (
            //             oc,
            //             RecievedMessage {
            //                 count: 1,
            //                 last_timestamp: msg.timestamp.unwrap(),
            //                 cur_timestamp: msg.timestamp.unwrap(),
            //                 pymsg: msg,
            //             },
            //         ),
            //     );
            //     self.row_heights = self.recalculate_row_heights();
            // }

            if let Some(oc) = self.message_id_to_opencan(msg.arbitration_id) {
                // motorsport
                self.motorsports.ingest(&oc, msg.data.unwrap().as_slice());
            }
        }
    }
}
