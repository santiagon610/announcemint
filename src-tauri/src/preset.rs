//! Output presets: Ogg-only or WAV with configurable sample rate, bit depth, channels, endianness.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Output format for generated audio.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Ogg,
    Wav,
}

/// Endianness for WAV output.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputEndian {
    #[default]
    Little,
    Big,
}

/// A named preset defining how to convert Polly Ogg output to final files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputPreset {
    pub name: String,
    pub format: OutputFormat,
    #[serde(default)]
    pub endian: OutputEndian,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
}

impl OutputPreset {
    /// Built-in preset for two-way radio voice prompts: WAV, 8 kHz, 16-bit, mono, little endian.
    pub fn two_way_voice_prompt() -> Self {
        Self {
            name: "WAV: Two-Way Radio Voice Prompt".to_string(),
            format: OutputFormat::Wav,
            endian: OutputEndian::Little,
            sample_rate: 8000,
            bit_depth: 16,
            channels: 1,
        }
    }

    /// Ogg-only: no conversion, keep Polly output as-is.
    pub fn ogg_only() -> Self {
        Self {
            name: "OGG Vorbis".to_string(),
            format: OutputFormat::Ogg,
            endian: OutputEndian::Little,
            sample_rate: 22050, // typical Polly Ogg rate, unused when format is Ogg
            bit_depth: 16,
            channels: 1,
        }
    }

    /// All built-in presets.
    pub fn builtins() -> Vec<OutputPreset> {
        vec![Self::ogg_only(), Self::two_way_voice_prompt()]
    }
}

/// Convert Ogg bytes to WAV using the given preset. Returns the WAV bytes.
pub fn ogg_to_wav(ogg_bytes: &[u8], preset: &OutputPreset) -> Result<Vec<u8>, String> {
    if preset.format != OutputFormat::Wav {
        return Err("preset is not WAV".into());
    }
    let mut cursor = std::io::Cursor::new(ogg_bytes);
    let mut ogg_reader = lewton::inside_ogg::OggStreamReader::new(&mut cursor)
        .map_err(|e| format!("ogg decode: {}", e))?;
    let sample_rate_in = ogg_reader.ident_hdr.audio_sample_rate;
    let channels_in = ogg_reader.ident_hdr.audio_channels as usize;

    let mut pcm_i16: Vec<i16> = Vec::new();
    while let Some(packets) = ogg_reader.read_dec_packet().map_err(|e| e.to_string())? {
        for ch in packets {
            pcm_i16.extend(ch);
        }
    }

    if pcm_i16.is_empty() {
        return Err("empty ogg stream".into());
    }

    // Convert to mono if needed (average channels)
    let mono: Vec<i16> = if channels_in > 1 {
        pcm_i16
            .chunks(channels_in)
            .map(|c| {
                let sum: i32 = c.iter().map(|&s| s as i32).sum();
                (sum / channels_in as i32) as i16
            })
            .collect()
    } else {
        pcm_i16
    };

    // Resample to target rate if different
    let samples = if preset.sample_rate != sample_rate_in {
        resample_i16(&mono, sample_rate_in, preset.sample_rate)?
    } else {
        mono
    };

    // Write WAV to in-memory buffer
    let mut wav_buf = Vec::new();
    {
        let spec = hound::WavSpec {
            channels: preset.channels,
            sample_rate: preset.sample_rate,
            bits_per_sample: preset.bit_depth,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut wav_buf), spec)
            .map_err(|e| format!("wav writer: {}", e))?;
        for s in &samples {
            writer
                .write_sample(*s)
                .map_err(|e| format!("wav write: {}", e))?;
        }
        writer
            .finalize()
            .map_err(|e| format!("wav finalize: {}", e))?;
    }
    Ok(wav_buf)
}

/// Resample i16 PCM from one sample rate to another using rubato (f32 path).
fn resample_i16(samples: &[i16], rate_in: u32, rate_out: u32) -> Result<Vec<i16>, String> {
    if rate_in == rate_out {
        return Ok(samples.to_vec());
    }
    let f32_in: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();
    let f32_out = resample_f32(&f32_in, rate_in, rate_out)?;
    Ok(f32_out
        .iter()
        .map(|&f| {
            let clamped = f.clamp(-1.0, 1.0);
            (clamped * 32767.0) as i16
        })
        .collect())
}

fn resample_f32(samples: &[f32], rate_in: u32, rate_out: u32) -> Result<Vec<f32>, String> {
    use rubato::{FftFixedInOut, Resampler};
    let chunk_size = 1024;
    let mut resampler = FftFixedInOut::new(rate_in as usize, rate_out as usize, chunk_size, 1)
        .map_err(|e| format!("resampler: {}", e))?;
    let in_frames = resampler.input_frames_next();
    let mut out = Vec::new();
    let mut pos = 0;
    while pos + in_frames <= samples.len() {
        let chunk: Vec<f32> = samples[pos..pos + in_frames].to_vec();
        let waves_in = vec![chunk];
        let waves_out = resampler
            .process(&waves_in, None)
            .map_err(|e| format!("resample: {}", e))?;
        out.extend(waves_out.into_iter().next().unwrap_or_default());
        pos += in_frames;
    }
    if pos < samples.len() {
        let mut last_chunk = samples[pos..].to_vec();
        last_chunk.resize(in_frames, 0.0);
        let waves_in = vec![last_chunk];
        let waves_out = resampler
            .process_partial(Some(&waves_in), None)
            .map_err(|e| format!("resample partial: {}", e))?;
        out.extend(waves_out.into_iter().next().unwrap_or_default());
    }
    Ok(out)
}

/// Apply preset: for Ogg return the path as-is; for WAV convert Ogg to WAV and write to output_dir.
/// Calls progress with "converting" and "saving" (and "no_conversion" for Ogg).
pub async fn apply_preset(
    ogg_path: &Path,
    preset: &OutputPreset,
    output_dir: &Path,
    progress: Option<&dyn crate::progress::ProgressReporter>,
) -> Result<std::path::PathBuf, String> {
    let stem = ogg_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    match preset.format {
        OutputFormat::Ogg => {
            if let Some(r) = progress {
                r.report("no_conversion");
                r.report("saving");
            }
            Ok(ogg_path.to_path_buf())
        }
        OutputFormat::Wav => {
            if let Some(r) = progress {
                r.report("converting");
            }
            let ogg_bytes = tokio::fs::read(ogg_path).await.map_err(|e| e.to_string())?;
            let wav_bytes = ogg_to_wav(&ogg_bytes, preset)?;
            let dest = output_dir.join(format!("{}.wav", stem));
            if let Some(r) = progress {
                r.report("saving");
            }
            tokio::fs::write(&dest, wav_bytes)
                .await
                .map_err(|e| e.to_string())?;
            Ok(dest)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OutputFormat, OutputPreset};

    #[test]
    fn test_builtins() {
        let builtins = OutputPreset::builtins();
        assert!(builtins.len() >= 2);
        assert_eq!(builtins[0].name, "Ogg only");
        assert_eq!(builtins[0].format, OutputFormat::Ogg);
        let two_way = OutputPreset::two_way_voice_prompt();
        assert_eq!(two_way.sample_rate, 8000);
        assert_eq!(two_way.bit_depth, 16);
        assert_eq!(two_way.channels, 1);
    }
}
