use serde::{Deserialize, Deserializer, de::Error};
use serde_json::Value;

#[expect(
    clippy::cast_possible_truncation,
    reason = "the float is checked to be whole first"
)]
fn whole(value: &Value) -> Option<i64> {
    let number = value.as_number()?;
    number.as_i64().or_else(|| {
        let float = number.as_f64()?;
        (float.fract() == 0.0).then_some(float as i64)
    })
}

pub fn int<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<i64>,
{
    let value = Value::deserialize(deserializer)?;
    whole(&value)
        .and_then(|number| T::try_from(number).ok())
        .ok_or_else(|| D::Error::custom(format!("expected a whole number, got {value}")))
}

pub fn opt_int<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<i64>,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    whole(&value)
        .and_then(|number| T::try_from(number).ok())
        .map(Some)
        .ok_or_else(|| D::Error::custom(format!("expected a whole number, got {value}")))
}

pub trait Whole: TryFrom<i64> + Copy {
    const LOWEST: Self;
    const HIGHEST: Self;
}

impl Whole for u8 {
    const LOWEST: Self = Self::MIN;
    const HIGHEST: Self = Self::MAX;
}

impl Whole for u32 {
    const LOWEST: Self = Self::MIN;
    const HIGHEST: Self = Self::MAX;
}

fn saturate<T: Whole>(number: i64) -> T {
    T::try_from(number).unwrap_or(if number < 0 { T::LOWEST } else { T::HIGHEST })
}

pub fn saturating_int<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Whole,
{
    let value = Value::deserialize(deserializer)?;
    whole(&value)
        .map(saturate)
        .ok_or_else(|| D::Error::custom(format!("expected a whole number, got {value}")))
}

pub fn saturating_opt_int<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Whole,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    whole(&value)
        .map(|number| Some(saturate(number)))
        .ok_or_else(|| D::Error::custom(format!("expected a whole number, got {value}")))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize)]
    struct Probe {
        #[serde(default, deserialize_with = "super::int")]
        exact: u32,
        #[serde(default, deserialize_with = "super::saturating_int")]
        saturated: u8,
        #[serde(default, deserialize_with = "super::saturating_opt_int")]
        optional: Option<u8>,
    }

    fn probe(value: serde_json::Value) -> Result<Probe, serde_json::Error> {
        serde_json::from_value(value)
    }

    #[test]
    fn fractions_are_rejected_and_whole_floats_accepted() {
        assert!(probe(json!({ "exact": 1.5 })).ok().is_none());
        assert!(probe(json!({ "exact": -1 })).ok().is_none());
        assert_eq!(probe(json!({ "exact": 2.0 })).expect("parses").exact, 2);
    }

    #[test]
    fn huge_numbers_saturate_and_null_is_none() {
        let parsed = probe(json!({ "saturated": 1e20, "optional": null })).expect("parses");
        assert_eq!((parsed.saturated, parsed.optional), (u8::MAX, None));
        let negative = probe(json!({ "saturated": -1e20, "optional": 300 })).expect("parses");
        assert_eq!((negative.saturated, negative.optional), (0, Some(u8::MAX)));
        assert!(probe(json!({ "optional": 1.5 })).ok().is_none());
    }
}
