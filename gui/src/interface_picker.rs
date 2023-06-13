use eframe::egui::*;

use crate::Gui;

#[derive(Debug, PartialEq)]
enum SupportedInterface {
    Socketcand,
}

pub struct InterfacePicker {
    interface: Option<SupportedInterface>,
    host: String,
    channel: String,
    port: String,
}

impl Default for InterfacePicker {
    fn default() -> Self {
        Self {
            interface: None,
            host: String::new(),
            channel: String::new(),
            port: String::new(),
        }
    }
}

impl Gui {
    pub fn interface_picker(&mut self, ui: &mut Ui) {
        // dropdown to select interface type
        let label = match self.interface_picker.interface {
            Some(SupportedInterface::Socketcand) => "Socketcand",
            None => "Select interface type",
        };

        ui.vertical_centered(|ui| {
            ComboBox::from_label("Select interface type")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.interface_picker.interface,
                        Some(SupportedInterface::Socketcand),
                        "Socketcand",
                    );
                });

            let Some(iface) = self.interface_picker.interface.as_ref() else {
                    return;
                };

            match iface {
                SupportedInterface::Socketcand => {
                    ui.add(TextEdit::singleline(&mut self.interface_picker.host).hint_text("Host"));
                    ui.add(TextEdit::singleline(&mut self.interface_picker.port).hint_text("Port"));
                    ui.add(
                        TextEdit::singleline(&mut self.interface_picker.channel)
                            .hint_text("Channel"),
                    );
                }
            }

            if ui.button("Connect").clicked() {
                print!("{iface:?}");

                match iface {
                    SupportedInterface::Socketcand => {
                        let bus = pycanrs::PyCanBusType::Socketcand {
                            host: self.interface_picker.host.clone(),
                            port: self.interface_picker.port.parse().unwrap(), // todo: handle error
                            channel: self.interface_picker.channel.clone(),
                        };

                        let pycan_iface = pycanrs::PyCanInterface::new(bus).unwrap();
                        self.assign_interface(pycan_iface).unwrap();
                    }
                }
            }
        });
        // for socketcand, we need to know the host, port, and channel
    }
}
