use eframe::egui;
use std::sync::{Arc, Mutex};

pub type SharedSpectrum = Arc<Mutex<Vec<f32>>>;

pub struct VisualizerApp {
    pub spectrum: SharedSpectrum,
    smoothed: Vec<f32>,
}

impl VisualizerApp {
    pub fn new(spectrum: SharedSpectrum) -> Self {
        Self {
            spectrum,
            smoothed: Vec::new(),
        }
    }
}

const BAR_COUNT: usize = 64;
const ATTACK: f32 = 0.5;   // how fast bars rise toward a louder value
const RELEASE: f32 = 0.1;  // how fast bars fall toward a quieter value
const HEIGHT_SCALE: f32 = 0.6; // overall vertical scale, tune to taste

const DB_MIN: f32 = -50.0;
const DB_MAX: f32 = -8.0;

impl eframe::App for VisualizerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint();

        let magnitudes = self.spectrum.lock().unwrap().clone();
        if magnitudes.is_empty() {
            return;
        }

        if self.smoothed.len() != BAR_COUNT {
            self.smoothed = vec![0.0; BAR_COUNT];
        }

        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();
        let bar_width = rect.width() / BAR_COUNT as f32;

        let bin_count = magnitudes.len();
        let min_bin = 1.0_f32;
        let max_bin = (bin_count - 1).max(2) as f32;

        let mut raw_edges = Vec::with_capacity(BAR_COUNT + 1);
        for i in 0..=BAR_COUNT {
            let t = i as f32 / BAR_COUNT as f32;
            raw_edges.push(min_bin * (max_bin / min_bin).powf(t));
        }

        let mut edges = vec![0usize; BAR_COUNT + 1];
        edges[0] = 1; // start just after DC bin
        for i in 1..=BAR_COUNT {
            let candidate = raw_edges[i].round() as usize;
            edges[i] = candidate.max(edges[i - 1] + 1).min(bin_count - 1);
        }

        for i in 0..BAR_COUNT {
            let start = edges[i];
            let end = edges[i + 1].max(start + 1).min(bin_count);

            let avg_mag: f32 =
                magnitudes[start..end].iter().sum::<f32>() / (end - start) as f32;

            let db = 20.0 * avg_mag.max(1e-6).log10();
            let normalized = ((db - DB_MIN) / (DB_MAX - DB_MIN)).clamp(0.0, 1.0);
            let target_height = normalized * rect.height() * HEIGHT_SCALE;

            let rate = if target_height > self.smoothed[i] { ATTACK } else { RELEASE };
            self.smoothed[i] += (target_height - self.smoothed[i]) * rate;

            let x = rect.left() + i as f32 * bar_width;
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(x, rect.bottom() - self.smoothed[i]),
                egui::pos2(x + bar_width - 1.0, rect.bottom()),
            );

            painter.rect_filled(bar_rect, 0.0, egui::Color32::from_rgb(80, 200, 255));
        }
    }
}
