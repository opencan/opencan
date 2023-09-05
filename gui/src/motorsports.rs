use std::os::macos::raw;

use bitvec::prelude::*;
use eframe::egui::Ui;
use egui_extras::{Column, Table, TableBuilder};
use opencan_core::{CANMessage, CANSignalWithPosition};

use crate::decode;

pub struct Motorsports {
    pub vehicle_speed: f64,
    pub inv_dc_voltage: f64,
    pub inv_dc_current: f64,
    pub vcu_state: String,
    pub pch_state: String,
}

impl Default for Motorsports {
    fn default() -> Self {
        Self {
            vehicle_speed: -999.0,
            inv_dc_voltage: -99.0,
            inv_dc_current: -99.0,
            vcu_state: "??".to_string(),
            pch_state: "??".to_string(),
        }
    }
}

impl Motorsports {
    pub fn motorsports_gooey(&self, ui: &mut Ui) {
        use eframe::egui::*;

        ui.vertical_centered(|ui| {
            ui.label(RichText::new("-- SLIM JIM --").font(FontId::proportional(40.0)));
            ui.label(
                RichText::new(format!("{:.1}", self.vehicle_speed))
                    .font(FontId::proportional(100.0)),
            );
        });

        TopBottomPanel::bottom("bottom_bar").show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                TableBuilder::new(ui)
                    .column(Column::exact(120.).resizable(false))
                    .column(Column::exact(120.).resizable(false))
                    .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
                    .header(30., |mut header| {
                        header.col(|ui| {
                            ui.heading("VCU State");
                        });
                        header.col(|ui| {
                            ui.heading("PCH State");
                        });
                    })
                    .body(|mut body| {
                        body.row(40.0, |mut row| {
                            row.col(|ui| {
                                ui.label(
                                    RichText::new(&self.vcu_state).font(FontId::proportional(20.0)).strong(),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    RichText::new(&self.pch_state).font(FontId::proportional(20.0)).strong(),
                                );
                            });
                        })
                    });
                ui.separator();
                ui.push_id("lower bar", |ui| {
                    TableBuilder::new(ui)
                        .column(Column::exact(180.).resizable(false))
                        .column(Column::exact(180.).resizable(false))
                        .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
                        .header(30., |mut header| {
                            header.col(|ui| {
                                ui.heading("VDC (inv)");
                            });
                            header.col(|ui| {
                                ui.heading("IDC (inv)");
                            });
                        })
                        .body(|mut body| {
                            body.row(60.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(format!("{:.1} V", self.inv_dc_voltage))
                                            .font(FontId::proportional(40.0)),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(format!("{:.1} A", self.inv_dc_current))
                                            .font(FontId::proportional(40.0)),
                                    );
                                });
                            });
                        });
                });
            });
        });

        ui.ctx().request_repaint();
    }

    pub fn ingest(&mut self, msg: &CANMessage, data: &[u8]) {
        match msg.name.as_str() {
            "M165_Motor_Position_Info" => self.ingest_m165(msg, data),
            "M166_Current_Info" => self.ingest_m166(msg, data),
            "M167_Voltage_Info" => self.ingest_m167(msg, data),
            "VCU_Status" => self.ingest_vcu_status(msg, data),
            "PCH_Status" => self.ingest_pch_status(msg, data),
            _ => (),
        }
    }

    fn ingest_m165(&mut self, msg: &CANMessage, data: &[u8]) {
        const MOTOR_SPEED_TO_VEH_SPEED: f64 = 0.0146;
        for sigbit in &msg.signals {
            match sigbit.sig.name.as_str() {
                "D2_Motor_Speed" => {
                    self.vehicle_speed = MOTOR_SPEED_TO_VEH_SPEED * decode_signal_f64(sigbit, data);
                }
                _ => (),
            }
        }
    }

    fn ingest_m166(&mut self, msg: &CANMessage, data: &[u8]) {
        for sigbit in &msg.signals {
            match sigbit.sig.name.as_str() {
                "D4_DC_Bus_Current" => {
                    self.inv_dc_current = decode_signal_f64(sigbit, data);
                }
                _ => (),
            }
        }
    }

    fn ingest_m167(&mut self, msg: &CANMessage, data: &[u8]) {
        for sigbit in &msg.signals {
            match sigbit.sig.name.as_str() {
                "D1_DC_Bus_Voltage" => {
                    self.inv_dc_voltage = decode_signal_f64(sigbit, data);
                }
                _ => (),
            }
        }
    }

    fn ingest_vcu_status(&mut self, msg: &CANMessage, data: &[u8]) {
        for sigbit in &msg.signals {
            match sigbit.sig.name.as_str() {
                "VCU_state" => {
                    self.vcu_state = decode_signal_enumerated(sigbit, data);
                }
                _ => (),
            }
        }
    }

    fn ingest_pch_status(&mut self, msg: &CANMessage, data: &[u8]) {
        for sigbit in &msg.signals {
            match sigbit.sig.name.as_str() {
                "PCH_state" => {
                    self.pch_state = decode_signal_enumerated(sigbit, data);
                }
                _ => (),
            }
        }
    }
}

pub fn decode_signal_f64(sigbit: &CANSignalWithPosition, data: &[u8]) -> f64 {
    let bits = data.view_bits::<Lsb0>();

    match sigbit.sig.twos_complement {
        true => {
            let sigraw: i64 = bits[sigbit.start() as _..=sigbit.end() as _].load();
            (sigraw as f64 * sigbit.sig.scale.unwrap_or(1.)) + sigbit.sig.offset.unwrap_or(0.)
        }
        false => {
            let sigraw: u64 = bits[sigbit.start() as _..=sigbit.end() as _].load();
            (sigraw as f64 * sigbit.sig.scale.unwrap_or(1.)) + sigbit.sig.offset.unwrap_or(0.)
        }
    }
}

pub fn decode_signal_enumerated(sigbit: &CANSignalWithPosition, data: &[u8]) -> String {
    let bits = data.view_bits::<Lsb0>();

    let raw_u64 = match sigbit.sig.twos_complement {
        true => {
            let sigraw: i64 = bits[sigbit.start() as _..=sigbit.end() as _].load();
            sigraw as u64
        }
        false => {
            let sigraw: u64 = bits[sigbit.start() as _..=sigbit.end() as _].load();
            sigraw
        }
    };

    match sigbit.sig.enumerated_values.get_by_right(&raw_u64) {
        Some(n) => n.to_owned(),
        None => format!("(internal error)"),
    }
}
