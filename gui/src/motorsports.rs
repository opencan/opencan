use bitvec::prelude::*;
use eframe::egui::Ui;
use opencan_core::{CANMessage, CANSignalWithPosition};

pub struct Motorsports {
    pub vehicle_speed: f64,
    pub inv_dc_voltage: f64,
}

impl Default for Motorsports {
    fn default() -> Self {
        Self { vehicle_speed: -999.0, inv_dc_voltage: -999.0 }
    }
}

impl Motorsports {
    pub fn motorsports_gooey(&self, ui: &mut Ui) {
        use eframe::egui::*;

        ui.vertical_centered(|ui| {
            ui.label(RichText::new("-- SLIM JIM --").font(FontId::proportional(40.0)));
            ui.label(
                RichText::new(format!("{:.1}", self.vehicle_speed)).font(FontId::proportional(100.0)),
            );
        });

        ui.ctx().request_repaint();
    }

    pub fn ingest(&mut self, msg: &CANMessage, data: &[u8]) {
        match msg.name.as_str() {
            "M165_Motor_Position_Info" => self.ingest_m165(msg, data),
            "M167_Voltage_Info" => self.ingest_m167(msg, data),
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
