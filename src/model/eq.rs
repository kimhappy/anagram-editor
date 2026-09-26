#![expect(
    clippy::float_arithmetic,
    clippy::suboptimal_flops,
    reason = "filter responses are floating-point arithmetic"
)]
use std::f64::consts::{LN_2, PI};

use super::catalog::BlockModel;

pub const SAMPLE_RATE: f64 = 48_000.0;
pub const MIN_FREQ: f64 = 20.0;
pub const MAX_FREQ: f64 = 20_000.0;
pub const GAIN_RANGE: f64 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    LowShelf,
    Peak,
    HighShelf,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqPoint {
    pub shape: Shape,
    pub freq: f64,
    pub gain: f64,
    pub width: f64,
}

struct Biquad {
    zeros: [f64; 3],
    poles: [f64; 3],
}

impl Biquad {
    fn magnitude_db(&self, freq: f64) -> f64 {
        let omega = 2.0 * PI * freq / SAMPLE_RATE;
        let squared = |[c0, c1, c2]: [f64; 3]| {
            let real = c0 + c1 * omega.cos() + c2 * (2.0 * omega).cos();
            let imaginary = c1 * omega.sin() + c2 * (2.0 * omega).sin();
            real * real + imaginary * imaginary
        };
        10.0 * (squared(self.zeros) / squared(self.poles)).log10()
    }
}

#[must_use]
pub fn bandwidth_octaves(width: f64) -> f64 {
    (width * 6.0 - 4.0).exp2()
}

#[must_use]
pub fn shelf_q(width: f64) -> f64 {
    ((1.0 - width) * 6.0 - 4.0).exp2() * 0.444_444 + 0.212_899
}

impl EqPoint {
    fn biquad(&self) -> Biquad {
        let amplitude = 10_f64.powf(self.gain / 40.0);
        let omega = 2.0 * PI * self.freq.clamp(1.0, SAMPLE_RATE * 0.49) / SAMPLE_RATE;
        let (sin, cos) = omega.sin_cos();
        match self.shape {
            Shape::Peak => {
                let alpha = sin * (LN_2 / 2.0 * bandwidth_octaves(self.width) * omega / sin).sinh();
                Biquad {
                    zeros: [1.0 + alpha * amplitude, -2.0 * cos, 1.0 - alpha * amplitude],
                    poles: [1.0 + alpha / amplitude, -2.0 * cos, 1.0 - alpha / amplitude],
                }
            }
            Shape::LowShelf | Shape::HighShelf => {
                let lift = 2.0 * amplitude.sqrt() * sin / (2.0 * shelf_q(self.width));
                let (plus, minus) = (amplitude + 1.0, amplitude - 1.0);
                let direction = if self.shape == Shape::LowShelf {
                    -1.0
                } else {
                    1.0
                };
                Biquad {
                    zeros: [
                        amplitude * (plus + direction * minus * cos + lift),
                        -2.0 * direction * amplitude * (minus + direction * plus * cos),
                        amplitude * (plus + direction * minus * cos - lift),
                    ],
                    poles: [
                        plus - direction * minus * cos + lift,
                        2.0 * direction * (minus - direction * plus * cos),
                        plus - direction * minus * cos - lift,
                    ],
                }
            }
        }
    }
}

#[must_use]
pub fn response_db(points: &[EqPoint], freq: f64) -> f64 {
    points
        .iter()
        .map(|point| point.biquad().magnitude_db(freq))
        .sum()
}

#[must_use]
pub fn freq_at(fraction: f64) -> f64 {
    MIN_FREQ * (MAX_FREQ / MIN_FREQ).powf(fraction.clamp(0.0, 1.0))
}

#[must_use]
pub fn fraction_of(freq: f64) -> f64 {
    ((freq / MIN_FREQ).ln() / (MAX_FREQ / MIN_FREQ).ln()).clamp(0.0, 1.0)
}

#[must_use]
pub fn gain_at(fraction: f64) -> f64 {
    GAIN_RANGE - fraction.clamp(0.0, 1.0) * 2.0 * GAIN_RANGE
}

#[must_use]
pub fn gain_fraction(gain: f64) -> f64 {
    ((GAIN_RANGE - gain) / (2.0 * GAIN_RANGE)).clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BandParams {
    pub label: &'static str,
    pub shape: Shape,
    pub freq: usize,
    pub gain: usize,
    pub width: usize,
    pub enable: Option<usize>,
}

pub const BAND_FIELDS: [&str; 3] = ["Freq", "Gain", "Width"];

const PARAMETRIC_BANDS: [(&str, Shape, [&str; 4]); 6] = [
    (
        "Low",
        Shape::LowShelf,
        ["LSfreq", "LSgain", "LSwidth", "LSsec"],
    ),
    ("1", Shape::Peak, ["freq1", "gain1", "width1", "sec1"]),
    ("2", Shape::Peak, ["freq2", "gain2", "width2", "sec2"]),
    ("3", Shape::Peak, ["freq3", "gain3", "width3", "sec3"]),
    ("4", Shape::Peak, ["freq4", "gain4", "width4", "sec4"]),
    (
        "High",
        Shape::HighShelf,
        ["HSfreq", "HSgain", "HSwidth", "HSsec"],
    ),
];

#[must_use]
pub fn parametric_bands(model: &BlockModel) -> Option<Vec<BandParams>> {
    PARAMETRIC_BANDS
        .iter()
        .map(|&(label, shape, [freq, gain, width, enable])| {
            Some(BandParams {
                label,
                shape,
                freq: model.param_index(freq)?,
                gain: model.param_index(gain)?,
                width: model.param_index(width)?,
                enable: model.param_index(enable),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{EqPoint, Shape, bandwidth_octaves, freq_at, gain_at, response_db};

    fn close(found: f64, expected: f64) -> bool {
        (found - expected).abs() < 0.05
    }

    #[test]
    fn a_peak_reaches_its_gain_at_its_centre_and_fades_away_from_it() {
        let peak = EqPoint {
            shape: Shape::Peak,
            freq: 1000.0,
            gain: 6.0,
            width: 0.5,
        };
        assert!(close(response_db(&[peak], 1000.0), 6.0));
        assert!(response_db(&[peak], 100.0).abs() < 0.1);
        assert!(close(bandwidth_octaves(0.5), 0.5));
    }

    #[test]
    fn shelves_hold_their_gain_beyond_the_corner() {
        let low = EqPoint {
            shape: Shape::LowShelf,
            freq: 200.0,
            gain: -9.0,
            width: 0.33,
        };
        let high = EqPoint {
            shape: Shape::HighShelf,
            freq: 4000.0,
            gain: 4.0,
            width: 0.33,
        };
        assert!(close(response_db(&[low], 20.0), -9.0));
        assert!(response_db(&[low], 10_000.0).abs() < 0.1);
        assert!(close(response_db(&[high], 20_000.0), 4.0));
        assert!(response_db(&[high], 50.0).abs() < 0.1);
        assert!(close(response_db(&[low, high], 20.0), -9.0));
    }

    #[test]
    fn the_axes_map_to_fractions_and_back() {
        assert!(close(freq_at(0.0), 20.0));
        assert!(close(freq_at(1.0), 20_000.0));
        assert!(close(freq_at(0.5).log10(), 632.455_f64.log10()));
        assert!(close(gain_at(0.25), 9.0));
    }
}
