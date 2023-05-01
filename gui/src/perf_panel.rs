use std::collections::VecDeque;

use eframe::egui::{self, Context};

pub const CPU_HISTORY_WINDOW: usize = 20;

pub struct PerfPanel {
    cpu_time_history: VecDeque<f32>,
    active: bool,
}

impl Default for PerfPanel {
    fn default() -> Self {
        Self {
            cpu_time_history: VecDeque::with_capacity(CPU_HISTORY_WINDOW),
            active: false,
        }
    }
}

impl PerfPanel {
    pub fn maybe_show(&mut self, ctx: &Context, frame: &eframe::Frame) {
        use egui::*;

        if ctx.input(|i| i.key_pressed(Key::End)) {
            self.active = true;
        }

        if ctx.input(|i| i.key_released(Key::End)) {
            self.active = false;
        }

        if self.active {
            TopBottomPanel::bottom("perf_panel").show(ctx, |ui| {
                let history = &mut self.cpu_time_history;

                if let Some(t) = frame.info().cpu_usage {
                    if history.len() >= CPU_HISTORY_WINDOW {
                        history.rotate_right(1);
                        history[0] = t;
                    } else {
                        history.push_front(t);
                    }
                }

                let avg = 1000. * history.iter().sum::<f32>() / history.len() as f32;

                ui.centered_and_justified(|ui| {
                    ui.label(format!("Average CPU usage per frame (last {CPU_HISTORY_WINDOW} frames): {avg:.1} ms"));
                });
            });
        }
    }
}
