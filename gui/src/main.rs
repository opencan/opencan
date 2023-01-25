#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;
use eframe::egui::{self, Slider};
use socketcan::CANSocket;

fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    #[cfg(target_os = "linux")]
    let can = Some(socketcan::CANSocket::open("can0")?);
    #[cfg(target_os = "macos")]
    let can = None;

    let gui = Gui { can_adapter: can };

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
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
    can_adapter: Option<CANSocket>,
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| ui.heading("OpenCAN Loves You"));
        });
    }
}
