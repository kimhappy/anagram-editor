#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unit {
    None,
    Seconds,
    Millis,
    Decibels,
    Percent,
    Hertz,
    Kilohertz,
    Megahertz,
    Cents,
    Semitones,
    Other(String),
}

impl Unit {
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        match trimmed.to_ascii_lowercase().as_str() {
            "" => Self::None,
            "s" => Self::Seconds,
            "ms" => Self::Millis,
            "db" => Self::Decibels,
            "%" | "pc" => Self::Percent,
            "hz" => Self::Hertz,
            "khz" => Self::Kilohertz,
            "mhz" => Self::Megahertz,
            "ct" | "cent" => Self::Cents,
            "semi" | "semitone12tet" => Self::Semitones,
            _ => Self::Other(trimmed.to_owned()),
        }
    }

    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::None => "",
            Self::Seconds => "s",
            Self::Millis => "ms",
            Self::Decibels => "dB",
            Self::Percent => "%",
            Self::Hertz => "Hz",
            Self::Kilohertz => "kHz",
            Self::Megahertz => "MHz",
            Self::Cents => "ct",
            Self::Semitones => "semi",
            Self::Other(label) => label,
        }
    }

    const fn shows_kilohertz(&self, value: f64) -> bool {
        matches!(self, Self::Hertz) && value.abs() >= KILO
    }
}

const KILO: f64 = 1000.0;
const BARE_KILO_LIMIT: f64 = 100.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub number: String,
    pub unit: String,
}

#[must_use]
#[expect(clippy::float_arithmetic, reason = "hertz shown as kilohertz")]
pub fn reading(unit: &Unit, value: f64, precision: usize) -> Reading {
    if unit.shows_kilohertz(value) {
        return Reading {
            number: format!("{:.2}", value / KILO),
            unit: Unit::Kilohertz.label().to_owned(),
        };
    }
    let precision = if *unit == Unit::Decibels {
        1
    } else {
        precision
    };
    Reading {
        number: format!("{value:.precision$}"),
        unit: unit.label().to_owned(),
    }
}

#[must_use]
#[expect(clippy::float_arithmetic, reason = "kilohertz typed back as hertz")]
pub fn parse_typed(unit: &Unit, typed: &str, shown: f64) -> Option<f64> {
    let lowered = typed.trim().to_ascii_lowercase();
    let has_suffix = lowered.ends_with(|character: char| character.is_ascii_alphabetic());
    let digits: String = lowered
        .trim_end_matches(|character: char| character.is_ascii_alphabetic() || character == '%')
        .trim()
        .to_owned();
    let number = digits
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())?;
    let reads_as_shown =
        !has_suffix && unit.shows_kilohertz(shown) && number.abs() < BARE_KILO_LIMIT;
    let is_kilo = lowered.ends_with("khz") || lowered.ends_with('k') || reads_as_shown;
    Some(if matches!(unit, Unit::Hertz) && is_kilo {
        number * KILO
    } else {
        number
    })
}

#[cfg(test)]
mod tests {
    use super::{Reading, Unit, parse_typed, reading};

    fn read(unit: &str, value: f64, precision: usize) -> Reading {
        reading(&Unit::parse(unit), value, precision)
    }

    #[test]
    fn units_read_like_the_device_table() {
        assert_eq!(
            read("db", -3.0, 2),
            Reading {
                number: "-3.0".to_owned(),
                unit: "dB".to_owned()
            }
        );
        assert_eq!(read("hz", 999.0, 0).unit, "Hz");
        assert_eq!(
            read("hz", 1000.0, 0),
            Reading {
                number: "1.00".to_owned(),
                unit: "kHz".to_owned()
            }
        );
        assert_eq!(read("hz", 12500.0, 0).number, "12.50");
        assert_eq!(read("pc", 50.0, 0).unit, "%");
        assert_eq!(read("semi", 2.0, 0).unit, "semi");
        assert_eq!(read("ct", 5.0, 0).unit, "ct");
        assert_eq!(read("s", 1.5, 1).unit, "s");
        assert_eq!(read("furlong", 1.0, 0).unit, "furlong");
    }

    #[test]
    fn typed_kilohertz_turn_back_into_hertz() {
        let hertz = Unit::Hertz;
        assert_eq!(parse_typed(&hertz, "2.5", 2000.0), Some(2500.0));
        assert_eq!(parse_typed(&hertz, "800", 2000.0), Some(800.0));
        assert_eq!(parse_typed(&hertz, "800 Hz", 2000.0), Some(800.0));
        assert_eq!(parse_typed(&hertz, "3k", 200.0), Some(3000.0));
        assert_eq!(parse_typed(&hertz, "3 kHz", 200.0), Some(3000.0));
        assert_eq!(parse_typed(&hertz, "300", 200.0), Some(300.0));
        assert_eq!(parse_typed(&Unit::Decibels, "-6 dB", 0.0), Some(-6.0));
    }
}
