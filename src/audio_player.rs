pub mod audio_player {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use ringbuf::{HeapRb, HeapCons, HeapProd};
    use ringbuf::traits::{Consumer, Producer, Split};

    pub fn play_file(path: String) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
        let rb = HeapRb::new(48000 * 2 * 2);
        let (producer, consumer) = rb.split();

        std::thread::spawn(move || {
            if let Err(e) = decode_file(&path, producer) {
                eprintln!("{}", e);
            }
        });

        let stream = init_cpal(consumer)?;
        stream.play()?;

        Ok(stream)
    }

    pub fn init_cpal(mut consumer: HeapCons<f32>) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available"); 
        let supported_config = device.default_output_config().expect("Didn't find default output config");
        let config = supported_config.into();
        let stream = device.build_output_stream(config,
            move |data: &mut [f32], _| {
                for sample in data.iter_mut() {
                    *sample = consumer.try_pop().unwrap_or(0.0); 
                }
            },
            move |err| eprintln!("{}", err),
            None,
        )?;
             

        Ok(stream)
    }
   
    use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::errors::Error as SymphoniaError;
    fn decode_file(path: &String, mut producer: HeapProd<f32>) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(&path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let probed = symphonia::default::get_probe().format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let mut format = probed.format;
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .unwrap();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;
        let mut sample_buf: Option<SampleBuffer<f32>> = None;
        let track_id = track.id;
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                    { break; }, 
                Err(e) => return Err(e.into()),
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    if sample_buf.is_none() {
                        let spec = *audio_buf.spec();
                        let duration = audio_buf.capacity() as u64;
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);
                        let samples: &[f32] = buf.samples();
                        for &s in samples {
                            while producer.try_push(s).is_err() {
                                std::thread::sleep(std::time::Duration::from_millis(1));
                            }
                        }
                    }
                }
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Err(e.into()),
            }

        };
        Ok(())
    }
}


