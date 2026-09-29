mod audio_player;
mod gui;
mod rfft;

use std::sync::{Arc, Mutex};
use gui::VisualizerApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spectrum = Arc::new(Mutex::new(Vec::new()));
    let (viz_tx, viz_rx) = std::sync::mpsc::channel::<Vec<f32>>();

    let spectrum_for_fft = spectrum.clone();
    std::thread::spawn(move || {
        rfft::convert_to_rfft(viz_rx, spectrum_for_fft);
    });

    let path = "../03 - Rhymes Like Dimes.flac".to_string();
    // Keep `_stream` alive for the whole program — call this on the main
    // thread directly (not inside another spawn) since cpal::Stream isn't
    // guaranteed Send on every platform, and play_file already spawns its
    // own decoder thread internally.
    let _stream = audio_player::play_file(&path, viz_tx)?;

    eframe::run_native(
        "Audio Visualizer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(VisualizerApp::new( spectrum )) as Box<dyn eframe::App>)),
    )?;

    Ok(())
}
