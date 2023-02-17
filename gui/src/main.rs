#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;
use eframe::egui::{self};

fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let gui = Gui { count: 0, table: Table{striped: true,resizeable:true} };

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 1000.0)),
        ..Default::default()
    };

    eframe::run_native(
        "OpenCAN GUI",
        options,
        Box::new(|_cc| Box::new(gui)),
    );

    Ok(())
}

struct Gui {
    count: u32,
    table: Table,
}

/// Table with dynamic layout 
#[cfg_attr(feature = "serde", derive(serde::Deserialize, srde:: Serialize))]
pub struct Table{
    striped: bool,
    resizeable: bool,
}


impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui|{
                    ui.checkbox(&mut self.table.striped, "Striped");
                    ui.checkbox(&mut self.table.resizeable, "Resizeable Columns");
                });

            });

        });

        self.count += 1;
    }
}

