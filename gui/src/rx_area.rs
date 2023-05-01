use eframe::egui::Ui;

use crate::Gui;

impl Gui {
    pub fn rx_area(&self, ui: &mut Ui) {
        use eframe::egui::*;
        use egui_extras::*;

        let mut repaint = false;

        ui.style_mut().override_text_style = Some(TextStyle::Monospace);
        ui.vertical(|ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
                .column(Column::auto().at_least(75.0).clip(false).resizable(false))
                .column(Column::auto().at_least(100.0).clip(false).resizable(false))
                .column(Column::auto().at_least(100.0).clip(false).resizable(false))
                .column(Column::auto().at_least(200.0).clip(false).resizable(false))
                .column(Column::auto().at_least(100.0).clip(false).resizable(false))
                .column(Column::auto().at_least(150.0).clip(false).resizable(false))
                .header(25.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("ID");
                    });
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Raw");
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
                    repaint = true;

                    body.heterogeneous_rows(
                        self.row_heights.iter().copied(),
                        |row_index, mut row| {
                            let (id, (opencan_msg, rx)) =
                                self.message_history.iter().nth(row_index).unwrap();

                            row.col(|ui| {
                                ui.label(format!("0x{id:X}"));
                            });
                            row.col(|ui| {
                                ui.label(&opencan_msg.name);
                            });
                            row.col(|ui| {
                                // bodge add spaces around text for column spacing
                                ui.label(if let Some(data) = &rx.pymsg.data {
                                    format!(" {data:02X?} ")
                                } else {
                                    " (empty message) ".into()
                                });
                            });
                            row.col(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    ui.label(if let Some(data) = &rx.pymsg.data {
                                        self.decode_message(opencan_msg, data)
                                    } else {
                                        "(empty message) ".into()
                                    });
                                });
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", rx.count));
                            });
                            row.col(|ui| {
                                ui.label(format!(
                                    "{:.0}", // todo moving average
                                    1000. * (rx.cur_timestamp - rx.last_timestamp)
                                ));
                            });
                        },
                    );
                });
        });

        if repaint {
            ui.ctx().request_repaint()
        }
    }

    pub fn recalculate_row_heights(&mut self) {
        self.row_heights = self
            .message_history
            .iter()
            .map(|(_id, (oc, _r))| 20. + 15. * (oc.signals.len() as f32))
            .collect();
    }
}
