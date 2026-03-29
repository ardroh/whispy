use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Raw PCM after the input stream is stopped (still at device rate / channel layout).
pub struct RawCapture {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

pub struct AudioRecorder {
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
}

impl AudioRecorder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 0,
            channels: 0,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;

        let config = device.default_input_config()?;
        self.sample_rate = config.sample_rate();
        self.channels = config.channels();

        let buffer = self.buffer.clone();
        buffer.lock().unwrap().clear();

        let err_fn = |err: cpal::StreamError| {
            tracing::error!("Audio stream error: {}", err);
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = buffer.lock() {
                        buf.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => {
                let buffer = self.buffer.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buf) = buffer.lock() {
                            buf.extend(data.iter().map(|&s| s as f32 / 32768.0));
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::U16 => {
                let buffer = self.buffer.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buf) = buffer.lock() {
                            buf.extend(data.iter().map(|&s| (s as f32 / 32768.0) - 1.0));
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            _ => anyhow::bail!("Unsupported sample format"),
        };

        stream.play()?;
        self.stream = Some(stream);
        tracing::info!(
            "Recording started: {}Hz, {} channels",
            self.sample_rate,
            self.channels
        );
        Ok(())
    }

    /// Open the default input device briefly so macOS can prompt for microphone access.
    /// Drops the stream immediately; does not require captured samples.
    pub fn request_microphone_access(&mut self) -> Result<()> {
        self.start()?;
        self.stream.take();
        self.buffer.lock().unwrap().clear();
        self.sample_rate = 0;
        self.channels = 0;
        Ok(())
    }

    pub fn audio_levels(&self, num_bands: usize) -> Vec<f32> {
        let buf = self.buffer.lock().unwrap();
        // Use more samples for stable readings (~2400 samples ≈ 50ms at 48kHz)
        let samples_needed = num_bands * 120;
        if buf.len() < num_bands {
            return vec![0.0; num_bands];
        }
        let start = buf.len().saturating_sub(samples_needed);
        let recent = &buf[start..];
        let chunk_size = recent.len() / num_bands;
        if chunk_size == 0 {
            return vec![0.0; num_bands];
        }
        recent
            .chunks(chunk_size)
            .take(num_bands)
            .map(|chunk| {
                let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
                // Mic RMS is typically 0.01-0.05; scale aggressively for visibility
                (rms * 25.0).min(1.0)
            })
            .collect()
    }

    /// Stop the input stream and take captured samples. Prefer the **main thread** on macOS (CoreAudio).
    pub fn end_capture(&mut self) -> Result<RawCapture> {
        self.stream.take();

        let samples = {
            let mut buf = self.buffer.lock().unwrap();
            std::mem::take(&mut *buf)
        };

        if samples.is_empty() {
            anyhow::bail!("No audio recorded");
        }

        Ok(RawCapture {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }
}

pub fn raw_to_wav(raw: RawCapture) -> Result<Vec<u8>> {
    let mono = if raw.channels > 1 {
        mix_to_mono(&raw.samples, raw.channels)
    } else {
        raw.samples
    };

    let target_rate = 16000u32;
    let resampled = if raw.sample_rate != target_rate {
        resample(&mono, raw.sample_rate, target_rate)
    } else {
        mono
    };

    encode_wav(&resampled, target_rate)
}

fn mix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    let ch = channels as usize;
    samples
        .chunks(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = to_rate as f64 / from_rate as f64;
    let output_len = (samples.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 / ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let s0 = samples[idx.min(samples.len() - 1)];
        let s1 = samples[(idx + 1).min(samples.len() - 1)];
        output.push(s0 + (s1 - s0) * frac as f32);
    }

    output
}

fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        for &sample in samples {
            let s = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(s)?;
        }
        writer.finalize()?;
    }

    Ok(cursor.into_inner())
}
