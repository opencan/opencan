#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{collections::BTreeMap, process::exit, sync::mpsc};

use anyhow::Result;
use clap::Parser;
use eframe::egui::{self};
use opencan_core::{CANMessage, CANNetwork};
use perf_panel::PerfPanel;
use pycanrs::{PyCanBusType, PyCanInterface, PyCanMessage};

mod decode;
mod perf_panel;
mod rx_area;
mod status_bar;

#[derive(Parser)]
struct Args {
    /// Path to .yml network definitions file.
    #[clap(short)]
    yml: String,
}

struct Gui {
    rx_channel: mpsc::Receiver<PyCanMessage>,

    /// Message ID -> last data
    message_history: BTreeMap<u32, (CANMessage, RecievedMessage)>,

    row_heights: Vec<f32>,

    network: CANNetwork,

    interface: PyCanInterface,

    perf_panel: PerfPanel,
}

impl Gui {
    pub fn new(
        rx_channel: mpsc::Receiver<PyCanMessage>,
        network: CANNetwork,
        interface: PyCanInterface,
    ) -> Self {
        Self {
            rx_channel,
            message_history: BTreeMap::new(),
            row_heights: Vec::new(),
            network,
            interface,
            perf_panel: Default::default(),
        }
    }
}

struct RecievedMessage {
    pymsg: PyCanMessage,
    count: u32,
    last_timestamp: f64,
    cur_timestamp: f64,
}

fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    ctrlc::set_handler(|| {
        eprintln!("Caught ctrl-c. Bye!");
        exit(0);
    })?;

    let (tx, rx) = mpsc::channel();

    // Set up CAN listener
    let message_cb = move |msg: &PyCanMessage| tx.send(msg.clone()).unwrap();
    let error_cb = |_: &_| {
        eprintln!("Error from pycanrs!");
        std::process::exit(-1);
    };

    let can = PyCanInterface::new(PyCanBusType::Socketcand {
        host: "side".into(),
        channel: "vcan0".into(),
        port: 30001,
    })?;
    // let can = pycanrs::PyCanInterface::new(PyCanBusType::Slcan { bitrate: 500000, serial_port: "/dev/tty.usbmodem11201".into() } )?;
    can.register_rx_callback(message_cb, error_cb)?;

    let network = opencan_compose::compose(opencan_compose::Args {
        in_file: args.yml,
        dump_json: false,
        dump_python: false,
    })
    .unwrap();

    let gui = Gui::new(rx, network, can);

    let options = eframe::NativeOptions {
        resizable: false,
        ..Default::default()
    };

    eframe::run_native("Juice", options, Box::new(|_cc| Box::new(gui))).unwrap();

    Ok(())
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // drain messages from channel
        // todo: performance pinch point. we should probably not do this in the egui update loop.
        while let Ok(msg) = self.rx_channel.try_recv() {
            // try to update existing message in history, else insert new one
            if let Some((_, prev)) = self.message_history.get_mut(&msg.arbitration_id) {
                *prev = RecievedMessage {
                    count: prev.count + 1,
                    last_timestamp: prev.cur_timestamp,
                    cur_timestamp: msg.timestamp.unwrap(),
                    pymsg: msg,
                };
            } else {
                if let Some(oc) = self.message_id_to_opencan(msg.arbitration_id) {
                    self.message_history.insert(
                        msg.arbitration_id,
                        (
                            oc,
                            RecievedMessage {
                                count: 1,
                                last_timestamp: msg.timestamp.unwrap(),
                                cur_timestamp: msg.timestamp.unwrap(),
                                pymsg: msg,
                            },
                        ),
                    );
                    self.recalculate_row_heights();
                }
            }
        }

        self.perf_panel.maybe_show(ctx, frame);

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(1.);
            self.status_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.rx_area(ui);
        });

        frame.set_window_size(ctx.used_size());
    }
}
