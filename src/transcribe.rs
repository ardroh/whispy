use anyhow::{Context, Result};
use reqwest::multipart;

pub async fn transcribe(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    language: &str,
    wav_data: Vec<u8>,
) -> Result<String> {
    let duration_secs = estimate_wav_duration(&wav_data);
    let price = estimate_price(model, duration_secs);
    tracing::info!(
        "Transcribing {:.1}s of audio with model '{}' — estimated cost: ${:.4}",
        duration_secs,
        model,
        price
    );

    let file_part = multipart::Part::bytes(wav_data)
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let form = multipart::Form::new()
        .text("model", model.to_string())
        .text("response_format", "text")
        .text("language", language.to_string())
        .part("file", file_part);

    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .context("Failed to send request to Whisper API")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Whisper API error ({}): {}", status, body);
    }

    let text = response
        .text()
        .await
        .context("Failed to read response body")?;
    Ok(text.trim().to_string())
}

/// Estimate WAV duration in seconds from the raw bytes (assumes standard WAV header).
fn estimate_wav_duration(wav_data: &[u8]) -> f64 {
    if wav_data.len() < 44 {
        return 0.0;
    }
    // WAV header: bytes 24-27 = sample rate (u32 LE), bytes 34-35 = bits per sample (u16 LE),
    // bytes 22-23 = channels (u16 LE)
    let sample_rate = u32::from_le_bytes([wav_data[24], wav_data[25], wav_data[26], wav_data[27]]);
    let bits_per_sample =
        u16::from_le_bytes([wav_data[34], wav_data[35]]);
    let channels = u16::from_le_bytes([wav_data[22], wav_data[23]]);

    if sample_rate == 0 || bits_per_sample == 0 || channels == 0 {
        return 0.0;
    }

    let bytes_per_sample = bits_per_sample as u32 / 8;
    let data_bytes = (wav_data.len() as u32).saturating_sub(44);
    let total_samples = data_bytes / (bytes_per_sample * channels as u32);
    total_samples as f64 / sample_rate as f64
}

/// Estimate transcription cost in USD based on model and duration.
/// Prices as of 2025 (per minute):
///   whisper-1: $0.006/min
///   gpt-4o-transcribe: $0.006/min
///   gpt-4o-mini-transcribe: $0.003/min
fn estimate_price(model: &str, duration_secs: f64) -> f64 {
    let per_minute = match model {
        "gpt-4o-mini-transcribe" => 0.003,
        _ => 0.006, // whisper-1, gpt-4o-transcribe, and others
    };
    (duration_secs / 60.0) * per_minute
}
