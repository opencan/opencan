#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;
use eframe::egui::{self};

fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let gui = Gui { count: 0 };

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 500.0)),
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
    count: u32
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| ui.heading("OpenCAN Loves You"));
            ui.vertical_centered_justified(|ui| example_plot(ui));
        });

        self.count += 1;
    }
}

fn example_plot(ui: &mut egui::Ui) -> egui::Response {
    use egui::plot::{Line, PlotPoints};
    let n = 128;
    let line_points: PlotPoints = (0..=n)
        .map(|i| {
            let x = egui::remap(i as f64, 0.0..=n as f64, -2.0..=2.0);
            [x, (4.0 - (x * x)).sqrt()]
        })
        .collect();
    let line_points2: PlotPoints = (0..=n)
        .map(|i| {
            let x = egui::remap(i as f64, 0.0..=n as f64, -2.0..=2.0);
            [x, -(4.0 - (x * x)).sqrt()]
        })
        .collect();
    let line = Line::new(line_points);
    let line2 = Line::new(line_points2);
    egui::plot::Plot::new("example_plot")
        .height(300.0)
        .data_aspect(1.0)
        .auto_bounds_x()
        .auto_bounds_y()
        .show(ui, |plot_ui| {
            plot_ui.line(line);
            plot_ui.line(line2);
        })
        .response
}