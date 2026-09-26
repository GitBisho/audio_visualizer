use realfft::RealFftPlanner;

pub fn convert_to_rfft(rx: std::sync::mpsc::Receiver<Vec<f32>>) {
    let mut real_planner = RealFftPlanner::<f32>::new();

    let r2c = real_planner.plan_fft_forward(1024);
    let mut input = r2c.make_input_vec();
    let mut output = r2c.make_output_vec();
    while let Ok(mut mono) = rx.recv() {

        while mono.len() >= 1024 {
            let chunk: Vec<f32> = mono.drain(..1024).collect();
            input.copy_from_slice(&chunk);

            if let Err(e) = r2c.process(&mut input, &mut output) {
                eprintln!("FFT error: {e}");
                continue;
            }
            if r2c.process(&mut input, &mut output).is_ok() {
                let magnitudes: Vec<f32> = output.iter().map(|c| c.norm()).collect();

                //Send magnitudes to visualizer
            }
        }
    }
}
