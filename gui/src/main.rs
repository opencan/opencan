#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{collections::BTreeMap, process::exit, sync::mpsc};

use anyhow::Result;
use eframe::egui::{self, TextStyle::Monospace};
use egui_extras::{Column, TableBuilder};
use pycanrs::{PyCanBusType, PyCanMessage};

struct Gui {
    messages: mpsc::Receiver<PyCanMessage>,

    /// Message ID -> last data
    message_history: BTreeMap<u32, RecievedMessage>,
}

struct RecievedMessage {
    msg: PyCanMessage,
    count: u32,
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

    let gui = Gui {
        messages: rx,
        message_history: BTreeMap::new(),
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
            let count = if let Some(prev) = self.message_history.get(&msg.arbitration_id) {
                prev.count
            } else {
                0
            };

            self.message_history
                .insert(msg.arbitration_id, RecievedMessage { msg, count: count + 1 });
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
                    .header(30.0, |mut header| {
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
                    })
                    .body(|body| {
                        let row_height = 40.0;
                        let num_rows = self.message_history.len();

                        body.rows(row_height, num_rows, |row_index, mut row| {
                            let (id, msg) = self.message_history.iter().nth(row_index).unwrap();

                            row.col(|ui| {
                                ui.label(format!("0x{id:X}"));
                            });
                            row.col(|ui| {
                                ui.label("Some message");
                            });
                            row.col(|ui| {
                                ui.label(
                                    if let Some(data) = &msg.msg.data {
                                        format!("{data:02X?}")
                                    } else {
                                        "(empty message)".into()
                                    }
                                );
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", msg.count));
                            });
                        })
                    });
            });
        });

        ctx.request_repaint();
    }
}
