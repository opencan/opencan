#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{sync::mpsc, ops::Deref, error, time::Duration};

use anyhow::Result;
use eframe::egui::{self};
use egui_extras::{Column, TableBuilder};
use pycanrs::{PyCanMessage, PyCanBusType};

struct Gui {
    messages: mpsc::Receiver<PyCanMessage>,
    last_message: Option<PyCanMessage>,
}

fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let (tx, rx) = mpsc::channel();

    // Set up CAN listener
    let message_cb = move |msg: &PyCanMessage| {
        tx.send(msg.clone()).unwrap()
    };
    let error_cb = |_: &_| {
        eprintln!("Error from pycanrs!");
        std::process::exit(-1);
    };

    // let can = pycanrs::PyCanInterface::new(PyCanBusType::Socketcand { host: "side".into(), channel: "vcan0".into(), port: 30000 })?;
    let can = pycanrs::PyCanInterface::new(PyCanBusType::Slcan { bitrate: 500000, serial_port: "/dev/tty.usbmodem11201".into() } )?;
    can.register_rx_callback(message_cb, error_cb)?;

    let gui = Gui { messages: rx, last_message: None };

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 1000.0)),
        ..Default::default()
    };

    eframe::run_native("OpenCAN GUI", options, Box::new(|_cc| Box::new(gui))).unwrap();

    Ok(())
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .column(Column::auto().at_least(30.0).clip(true).resizable(true))
                    .column(Column::auto().at_least(50.0).clip(true).resizable(true))
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("ID");
                        });
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Message");
                        });
                    })
                    .body(|body| {
                        let row_height = 40.0;
                        let num_rows = 1;
                        body.rows(row_height, num_rows, |_row_index, mut row| {
                            let msg = self.messages.recv_timeout(Duration::ZERO);
                            if let Ok(m) = msg { self.last_message = Some(m) };

                            let label = if let Some(m) = &self.last_message {
                                format!("{m}")
                            } else {
                                format!("(none yet)")
                            };

                            row.col(|ui| {
                                ui.label(label);
                            });
                        });
                    });
            });
        });

        ctx.request_repaint();
    }
}
