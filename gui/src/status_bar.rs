use eframe::egui::Ui;

use crate::Gui;

impl Gui {
    pub fn status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Connected using {}",
                match &self.interface.bustype {
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
        });
    }
}
