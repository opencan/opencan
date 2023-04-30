#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{collections::BTreeMap, process::exit, sync::mpsc};

use anyhow::Result;
use eframe::egui::{self, TextStyle::Monospace};
use egui_extras::{Column, TableBuilder};
use opencan_core::{CANMessage, CANNetwork};
use pycanrs::{PyCanBusType, PyCanMessage};

mod decode;

struct Gui {
    messages: mpsc::Receiver<PyCanMessage>,

    /// Message ID -> last data
    message_history: BTreeMap<u32, RecievedMessage>,

    network: CANNetwork,
}

struct RecievedMessage {
    pymsg: PyCanMessage,
    opencan_msg: CANMessage,
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

    let can = pycanrs::PyCanInterface::new(PyCanBusType::Socketcand {
        host: "side".into(),
        channel: "vcan0".into(),
        port: 30001,
    })?;
    // let can = pycanrs::PyCanInterface::new(PyCanBusType::Slcan { bitrate: 500000, serial_port: "/dev/tty.usbmodem11201".into() } )?;
    can.register_rx_callback(message_cb, error_cb)?;

    let network =
        opencan_compose::compose_str(include_str!("../../../../motorsports/can/can.yml")).unwrap();

    let gui = Gui {
        messages: rx,
        message_history: BTreeMap::new(),
        network,
    };

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
        while let Ok(msg) = self.messages.try_recv() {
            let (count, last_time, last_opencan) =
                // todo use get_mut rather than remove and insert; this is a mess
                if let Some(prev) = self.message_history.remove(&msg.arbitration_id) {
                    (prev.count, prev.cur_timestamp, prev.opencan_msg)
                } else {
                    (0, msg.timestamp.unwrap(), self.message_id_to_opencan(msg.arbitration_id))
                };

            self.message_history.insert(
                msg.arbitration_id,
                RecievedMessage {
                    count: count + 1,
                    last_timestamp: last_time,
                    cur_timestamp: msg.timestamp.unwrap(),
                    opencan_msg: last_opencan,
                    pymsg: msg,
                },
            );
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().override_text_style = Some(Monospace);
            ui.vertical(|ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .column(Column::auto().at_least(100.0).clip(true).resizable(true))
                    .column(Column::auto().at_least(100.0).clip(true).resizable(true))
                    .column(Column::auto().at_least(200.0).clip(true).resizable(true))
                    .column(Column::auto().at_least(100.0).clip(true).resizable(true))
                    .column(Column::auto().at_least(150.0).clip(true).resizable(true))
                    .header(25.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("ID");
                        });
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Message");
                        });
                        header.col(|ui| {
                            ui.heading("Count");
                        });
                        header.col(|ui| {
                            ui.heading("Cycle time (ms)");
                        });
                    })
                    .body(|body| {
                        let row_height = 100.0;
                        let num_rows = self.message_history.len();

                        body.rows(row_height, num_rows, |row_index, mut row| {
                            let (id, msg) = self.message_history.iter().nth(row_index).unwrap();

                            row.col(|ui| {
                                ui.label(format!("0x{id:X}"));
                            });
                            row.col(|ui| {
                                ui.label(&msg.opencan_msg.name);
                            });
                            row.col(|ui| {
                                ui.label(if let Some(data) = &msg.pymsg.data {
                                    format!(
                                        "{data:02X?}, aka: {}",
                                        self.decode_message(&msg.opencan_msg, data)
                                    )
                                } else {
                                    "(empty message)".into()
                                });
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", msg.count));
                            });
                            row.col(|ui| {
                                ui.label(format!(
                                    "{:.0}",
                                    1000. * (msg.cur_timestamp - msg.last_timestamp)
                                ));
                            });
                        })
                    });
            });
        });

        ctx.request_repaint();
    }
}
