use realfft::RealFftPlanner;
use std::sync::{Arc, Mutex};

pub fn convert_to_rfft(rx: std::sync::mpsc::Receiver<Vec<f32>>, ui_view: Arc<Mutex<Vec<f32>>>) {
    let mut real_planner = RealFftPlanner::<f32>::new();
    let r2c = real_planner.plan_fft_forward(1024);
    let mut input = r2c.make_input_vec();
    let mut output = r2c.make_output_vec();

    // persists across recv() calls so leftover samples that don't fill a
    // full 1024-chunk aren't dropped between packets
    let mut scratch: Vec<f32> = Vec::new();

    while let Ok(incoming) = rx.recv() {
        scratch.extend_from_slice(&incoming);

        while scratch.len() >= 1024 {
            let mut chunk: Vec<f32> = scratch.drain(..1024).collect();
            apply_hann_window(&mut chunk);

            input.copy_from_slice(&chunk);

            match r2c.process(&mut input, &mut output) {
                Ok(()) => {
                    let magnitudes: Vec<f32> = output.iter().map(|c| c.norm() / 512.0).collect();
                    if let Ok(mut guard) = ui_view.lock() {
                        if let Some(&max) = magnitudes.iter().cloned().reduce(f32::max).as_ref() {
                        println!("max magnitude: {}", max);
                        }
                        *guard = magnitudes;
                    }
                }
                Err(e) => eprintln!("FFT error: {e}"),
            }
        }
    }
}

fn apply_hann_window(samples: &mut [f32]) {
    let n = samples.len();
    for (i, s) in samples.iter_mut().enumerate() {
        let w = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (n as f32 - 1.0)).cos();
        *s *= w;
    }
}
