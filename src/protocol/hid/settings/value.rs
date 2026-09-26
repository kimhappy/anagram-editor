use serde_json::{Number, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub precision: usize,
    pub unit: &'static str,
    pub logarithmic: bool,
}

impl Bounds {
    #[must_use]
    pub const fn linear(
        min: f64,
        max: f64,
        step: f64,
        precision: usize,
        unit: &'static str,
    ) -> Self {
        Self {
            min,
            max,
            step,
            precision,
            unit,
            logarithmic: false,
        }
    }

    #[must_use]
    pub const fn logarithmic(min: f64, max: f64, step: f64, unit: &'static str) -> Self {
        Self {
            logarithmic: true,
            ..Self::linear(min, max, step, 0, unit)
        }
    }

    #[must_use]
    #[expect(clippy::float_arithmetic, reason = "snapping a value to its step")]
    pub fn snap(self, value: f64) -> f64 {
        let stepped = if self.step > 0.0 {
            ((value - self.min) / self.step)
                .round()
                .mul_add(self.step, self.min)
        } else {
            value
        };
        let digits = u32::try_from(self.precision).unwrap_or(0).min(9);
        let scale = f64::from(10_u32.pow(digits));
        ((stepped * scale).round() / scale).clamp(self.min, self.max)
    }

    #[must_use]
    pub const fn clamp(self, value: f64) -> f64 {
        if value.is_finite() {
            value.clamp(self.min, self.max)
        } else {
            self.min
        }
    }

    #[must_use]
    pub fn text(self, value: f64) -> String {
        let precision = self.precision;
        if self.unit.is_empty() {
            format!("{value:.precision$}")
        } else {
            format!("{value:.precision$} {}", self.unit)
        }
    }
}

#[must_use]
pub fn float(value: f64) -> Value {
    Number::from_f64(value).map_or(Value::Null, Value::Number)
}

#[cfg(test)]
mod tests {
    use super::Bounds;

    #[test]
    fn snapping_leaves_no_float_noise() {
        let gain = Bounds::linear(-18.0, 18.0, 0.1, 1, "dB");
        assert_eq!(gain.snap(3.0).to_string(), "3");
        assert_eq!(gain.snap(1.5).to_string(), "1.5");
        assert_eq!(gain.snap(-6.3).to_string(), "-6.3");
        assert_eq!(gain.snap(40.0).to_string(), "18");
    }
}
