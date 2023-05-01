#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    collections::{BTreeMap, VecDeque},
    process::exit,
    sync::mpsc,
};

use anyhow::Result;
use eframe::egui;
use opencan_core::{CANMessage, CANNetwork};
use pycanrs::{PyCanBusType, PyCanInterface, PyCanMessage};

mod decode;
mod rx_area;
mod status_bar;

const CPU_HISTORY_WINDOW: usize = 20;

struct Gui {
    rx_channel: mpsc::Receiver<PyCanMessage>,

    /// Message ID -> last data
    message_history: BTreeMap<u32, (CANMessage, RecievedMessage)>,

    network: CANNetwork,

    interface: PyCanInterface,

    cpu_time_history: VecDeque<f32>,
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
            network,
            interface,
            cpu_time_history: VecDeque::with_capacity(CPU_HISTORY_WINDOW),
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

    let network =
        opencan_compose::compose_str(include_str!("../../../../motorsports/can/can.yml")).unwrap();

    let gui = Gui::new(rx, network, can);

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 1000.0)),
        ..Default::default()
    };

    eframe::run_native("Juice", options, Box::new(|_cc| Box::new(gui))).unwrap();

    Ok(())
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                self.message_history.insert(
                    msg.arbitration_id,
                    (
                        self.message_id_to_opencan(msg.arbitration_id),
                        RecievedMessage {
                            count: 1,
                            last_timestamp: msg.timestamp.unwrap(),
                            cur_timestamp: msg.timestamp.unwrap(),
                            pymsg: msg,
                        },
                    ),
                );
            }
        }

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.status_bar(ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.rx_area(ui);

            let history = &mut self.cpu_time_history;

            if let Some(t) = _frame.info().cpu_usage {
                if history.len() >= CPU_HISTORY_WINDOW {
                    history.rotate_right(1);
                    history[0] = t;
                } else {
                    history.push_front(t);
                }
            }

            let avg = 1000. * history.iter().sum::<f32>() / history.len() as f32;

            ui.label(format!(
                "Average CPU usage per frame (last {CPU_HISTORY_WINDOW} frames): {avg:.1} ms"
            ));
        });

        ctx.request_repaint();
    }
}
