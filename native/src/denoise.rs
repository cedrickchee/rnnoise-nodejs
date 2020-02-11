// Preprocess sound data and invoke RNNoise
use neon::prelude::*;

use crate::rnnoise::{DenoiseState, FRAME_SIZE};

use std::fs::File;

use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};

// RNNoise assumes audio is 16-bit mono with a 48 KHz sample rate
const SAMPLE_RATE: u32 = 48_000;

pub fn suppress(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // Attempt to cast the first argument to a JsString. Then
    // get the value if cast is successul.
    let arg0 = cx.argument::<JsString>(0)?.value();
    let arg1 = cx.argument::<JsString>(1)?.value();

    let audio_file = File::open(&arg0).unwrap();
    let mut reader = Reader::new(audio_file).unwrap();
    let desc = reader.description();
    assert_eq!(
        1,
        desc.channel_count(),
        "The channel count is required to be one, at least for now"
    );

    // Obtain the buffer of samples
    let mut audio_buf: Vec<_> = if desc.sample_rate() == SAMPLE_RATE {
        reader.samples::<f32>().map(|s| s.unwrap()).collect()
    } else {
        // We need to interpolate to the target sample rate
        let interpolator = Linear::new([0f32], [0.0]);
        let conv = Converter::from_hz_to_hz(
            from_iter(reader.samples::<f32>().map(|s| [s.unwrap()])),
            interpolator,
            desc.sample_rate() as f64,
            SAMPLE_RATE as f64,
        );
        conv.until_exhausted().map(|v| v[0]).collect()
    };

    // println!("{} length: {}", &arg0, audio_buf.len());

    // The library requires each frame be exactly FRAME_SIZE, so we append
    // some zeros to be sure the final frame is sufficiently long.
    let padding = audio_buf.len() % FRAME_SIZE;
    if padding > 0 {
        let mut pad: Vec<f32> = vec![0.0; FRAME_SIZE - padding];
        audio_buf.append(&mut pad);
        // println!("padded audio file with {} characters", padding);
    }
    let mut denoised_buffer: Vec<f32> = vec![];
    let mut rnnoise = DenoiseState::new();
    let mut denoised_chunk: Vec<f32> = vec![0.0; FRAME_SIZE];
    let buffers = audio_buf[..].chunks(FRAME_SIZE);
    for buffer in buffers {
        rnnoise.process_frame_mut(&buffer, &mut denoised_chunk[..]);
        denoised_buffer.extend_from_slice(&mut denoised_chunk);
    }

    // println!("{} length: {}", &arg1, denoised_buffer.len());

    assert_eq!(audio_buf.len(), denoised_buffer.len());

    // Write denoised buffer into output file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let opt_wav_writer = hound::WavWriter::create(&arg1, spec);
    let mut wav_writer = opt_wav_writer.expect("failed to create wav file");
    denoised_buffer.iter().for_each(|i| {
        wav_writer
            .write_sample(*i)
            .expect("failed to write to wav file")
    });

    Ok(cx.number(denoised_buffer.len() as f64))
}
