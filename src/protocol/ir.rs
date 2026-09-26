#![expect(
    clippy::float_arithmetic,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::suboptimal_flops,
    reason = "sample processing is floating-point arithmetic over sample counts"
)]
use std::{f64::consts::PI, fmt, io::Cursor, iter};

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

pub const SAMPLE_RATE: u32 = 48_000;
pub const LENGTH: usize = 24_000;
const PEAK: f32 = 0.89125;
const FADE: usize = 480;
const CUTOFF: f64 = 0.95;
const KERNEL_ZEROS: f64 = 16.0;
const MAX_KERNEL_HALF_WIDTH: f64 = 2048.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrError {
    Decode(String),
    Encode(String),
    Empty,
    Silent,
}

impl fmt::Display for IrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(detail) => write!(formatter, "not a readable WAV file: {detail}"),
            Self::Encode(detail) => write!(formatter, "could not write the WAV file: {detail}"),
            Self::Empty => formatter.write_str("the WAV file has no samples"),
            Self::Silent => formatter.write_str("the WAV file is silent"),
        }
    }
}

impl std::error::Error for IrError {}

pub fn convert(wav: &[u8]) -> Result<Vec<u8>, IrError> {
    let (samples, rate) = decode_mono(wav)?;
    if samples.is_empty() {
        return Err(IrError::Empty);
    }
    let mut shaped: Vec<f32> = resample(&samples, rate, SAMPLE_RATE)
        .chain(iter::repeat(0.0))
        .take(LENGTH)
        .collect();
    normalise(&mut shaped)?;
    fade_tail(&mut shaped);
    encode(&shaped)
}

fn decode_mono(wav: &[u8]) -> Result<(Vec<f32>, u32), IrError> {
    let mut reader =
        WavReader::new(Cursor::new(wav)).map_err(|error| IrError::Decode(error.to_string()))?;
    let spec = reader.spec();
    let channels = usize::from(spec.channels.max(1));
    let interleaved: Vec<f32> = match spec.sample_format {
        SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<Result<_, _>>()
            .map_err(|error| IrError::Decode(error.to_string()))?,
        SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            if !(1..=32).contains(&bits) {
                return Err(IrError::Decode(format!(
                    "{bits} bits per sample are not supported"
                )));
            }
            let full_scale: u64 = 1 << (bits - 1);
            let scale = full_scale as f32;
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<_, _>>()
                .map_err(|error| IrError::Decode(error.to_string()))?
        }
    };
    let mono = interleaved
        .chunks(channels)
        .map(|frame| {
            frame
                .iter()
                .map(|&sample| if sample.is_finite() { sample } else { 0.0 })
                .sum::<f32>()
                / channels as f32
        })
        .collect();
    Ok((mono, spec.sample_rate))
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> impl Iterator<Item = f32> + '_ {
    let ratio = if from_rate == 0 {
        1.0
    } else {
        f64::from(from_rate) / f64::from(to_rate)
    };
    let cutoff = if ratio > 1.0 { CUTOFF / ratio } else { 1.0 };
    let half_width = (KERNEL_ZEROS / cutoff).min(MAX_KERNEL_HALF_WIDTH);
    let length = (samples.len() as f64 / ratio).ceil() as usize;
    (0..length).map(move |index| {
        let center = index as f64 * ratio;
        let first = (center - half_width).ceil().max(0.0) as usize;
        let last = ((center + half_width).floor() as usize).min(samples.len().saturating_sub(1));
        let sum: f64 = samples
            .get(first..=last)
            .unwrap_or_default()
            .iter()
            .zip(first..)
            .map(|(&sample, position)| {
                let offset = center - position as f64;
                f64::from(sample) * cutoff * sinc(cutoff * offset) * blackman(offset / half_width)
            })
            .sum();
        sum as f32
    })
}

fn sinc(x: f64) -> f64 {
    if x.abs() < f64::EPSILON {
        1.0
    } else {
        let angle = PI * x;
        angle.sin() / angle
    }
}

fn blackman(position: f64) -> f64 {
    if position.abs() > 1.0 {
        return 0.0;
    }
    let angle = PI * position;
    0.42 + 0.5 * angle.cos() + 0.08 * (2.0 * angle).cos()
}

fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0, |peak: f32, sample| peak.max(sample.abs()))
}

fn normalise(samples: &mut [f32]) -> Result<(), IrError> {
    let peak = peak(samples);
    if peak <= f32::MIN_POSITIVE {
        return Err(IrError::Silent);
    }
    let gain = PEAK / peak;
    for sample in samples.iter_mut() {
        *sample *= gain;
    }
    Ok(())
}

fn fade_tail(samples: &mut [f32]) {
    let length = samples.len();
    let fade = FADE.min(length);
    samples
        .iter_mut()
        .skip(length - fade)
        .enumerate()
        .for_each(|(index, sample)| *sample *= 1.0 - (index + 1) as f32 / fade as f32);
}

fn encode(samples: &[f32]) -> Result<Vec<u8>, IrError> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut cursor = Cursor::new(Vec::new());
    let mut writer =
        WavWriter::new(&mut cursor, spec).map_err(|error| IrError::Encode(error.to_string()))?;
    samples
        .iter()
        .try_for_each(|&sample| writer.write_sample(sample))
        .map_err(|error| IrError::Encode(error.to_string()))?;
    writer
        .finalize()
        .map_err(|error| IrError::Encode(error.to_string()))?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use std::{f64::consts::TAU, io::Cursor};

    use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

    use super::{IrError, LENGTH, SAMPLE_RATE, convert, resample};

    fn stereo_pcm(rate: u32, frames: usize) -> Vec<u8> {
        let spec = WavSpec {
            channels: 2,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = WavWriter::new(&mut cursor, spec).expect("writer");
        for index in 0..frames {
            let value = if index == 0 { 8000 } else { 0 };
            writer.write_sample::<i16>(value).expect("sample");
            writer.write_sample::<i16>(value).expect("sample");
        }
        writer.finalize().expect("finalise");
        cursor.into_inner()
    }

    #[test]
    fn output_is_mono_float_48k_with_24000_samples() {
        let converted = convert(&stereo_pcm(44_100, 1000)).expect("converts");
        let reader = WavReader::new(Cursor::new(converted)).expect("reads back");
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, SAMPLE_RATE);
        assert_eq!(spec.sample_format, SampleFormat::Float);
        assert_eq!(reader.len() as usize, LENGTH);
        let samples: Vec<f32> = reader
            .into_samples()
            .map(|sample| sample.expect("sample"))
            .collect();
        let peak = samples
            .iter()
            .fold(0.0, |peak: f32, sample| peak.max(sample.abs()));
        assert!((peak - 0.89125).abs() < 1e-3);
        assert_eq!(samples.last(), Some(&0.0));
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(matches!(convert(b"not a wav"), Err(IrError::Decode(_))));
    }

    fn mono_float(samples: &[f32]) -> Vec<u8> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = WavWriter::new(&mut cursor, spec).expect("writer");
        for &sample in samples {
            writer.write_sample(sample).expect("sample");
        }
        writer.finalize().expect("finalise");
        cursor.into_inner()
    }

    #[test]
    fn non_finite_samples_become_silence_instead_of_ruining_the_ir() {
        let converted =
            convert(&mono_float(&[0.5, f32::NAN, f32::INFINITY, 0.25])).expect("converts");
        let reader = WavReader::new(Cursor::new(converted)).expect("reads back");
        assert!(
            reader
                .into_samples::<f32>()
                .all(|sample| sample.expect("sample").is_finite())
        );
    }

    #[test]
    fn content_past_the_ir_length_counts_as_silence() {
        let mut samples = vec![0.0; LENGTH + 100];
        samples.push(1.0);
        assert_eq!(convert(&mono_float(&samples)), Err(IrError::Silent));
    }

    #[test]
    fn silence_is_rejected() {
        assert_eq!(convert(&mono_float(&[0.0; 64])), Err(IrError::Silent));
        assert_eq!(convert(&mono_float(&[f32::NAN; 4])), Err(IrError::Silent));
        assert_eq!(
            convert(&mono_float(&[f32::MIN_POSITIVE / 4.0; 64])),
            Err(IrError::Silent)
        );
    }

    fn tone(frequency: f64, rate: u32, length: usize) -> Vec<f32> {
        (0..length)
            .map(|index| (TAU * frequency * index as f64 / f64::from(rate)).sin() as f32)
            .collect()
    }

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
    }

    #[test]
    fn downsampling_removes_what_the_new_rate_cannot_hold() {
        let passed: Vec<f32> = resample(&tone(1000.0, 96_000, 9600), 96_000, 48_000).collect();
        let aliased: Vec<f32> = resample(&tone(30_000.0, 96_000, 9600), 96_000, 48_000).collect();
        let middle = 200..4600;
        assert!(rms(passed.get(middle.clone()).expect("middle")) > 0.65);
        assert!(rms(aliased.get(middle).expect("middle")) < 0.01);
    }

    #[test]
    fn the_same_rate_passes_samples_through() {
        let input = tone(440.0, 48_000, 100);
        let output: Vec<f32> = resample(&input, 48_000, 48_000).collect();
        assert!(
            input
                .iter()
                .zip(&output)
                .all(|(before, after)| (before - after).abs() < 1e-6)
        );
    }
}
