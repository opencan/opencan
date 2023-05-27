use eframe::{
    egui::{self, Ui},
    emath::Align::Center,
};

use crate::Gui;

impl Gui {
    pub fn status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            match &self.interface {
                Some(iface) => {
                    ui.label(format!(
                        "Connected using {}",
                        match &iface.interface.bustype {
                            pycanrs::PyCanBusType::Gsusb { usb_channel, .. } =>
                                format!("gs_usb at {usb_channel}"),
                            pycanrs::PyCanBusType::Slcan { serial_port, .. } =>
                                format!("slcan at {serial_port}"),
                            pycanrs::PyCanBusType::Socketcan { channel } =>
                                format!("socketcan at {channel} "),
                            pycanrs::PyCanBusType::Socketcand {
                                host,
                                channel,
                                port,
                            } => format!("socketcand on {host}:{port}, channel {channel}"),
                        }
                    ));
                }
                None => {
                    ui.label("Not connected");
                }
            }

            ui.with_layout(egui::Layout::right_to_left(Center), |ui| {
                if ui.button("Select .yml").clicked() {
                    if let Some(yml) = rfd::FileDialog::new().pick_file() {
                        let network = opencan_compose::compose(opencan_compose::Args {
                            in_file: yml.to_string_lossy().into(),
                            dump_json: false,
                            dump_python: false,
                        })
                        .unwrap();

                        self.network = Some(network);
                    }
                }
            })
        });
    }
}
