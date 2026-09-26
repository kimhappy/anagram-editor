use leptos::prelude::*;

const TRACK_STEPS: f64 = 1000.0;

#[component]
pub fn Slider(
    bounds: (f64, f64, f64),
    #[prop(into)] value: Signal<f64>,
    #[prop(into)] on_input: Callback<f64>,
    #[prop(into, optional)] on_commit: Option<Callback<f64>>,
    #[prop(optional)] logarithmic: bool,
    #[prop(into, optional)] label: String,
    #[prop(into, optional)] value_text: Option<Callback<f64, String>>,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let (min, max, step) = bounds;
    let is_log = logarithmic && min > 0.0 && max > min;
    let to_track = move |level: f64| {
        if is_log {
            log_position(level, min, max)
        } else {
            level
        }
    };
    let from_track = move |position: f64| {
        if is_log {
            log_level(position, min, max)
        } else {
            position
        }
    };
    let (track_min, track_max, track_step) = if is_log {
        (0.0, TRACK_STEPS, 1.0)
    } else {
        (min, max, step)
    };
    let parse = move |event: &leptos::ev::Event| {
        event_target_value(event)
            .parse::<f64>()
            .ok()
            .map(from_track)
    };
    view! {
        <input
            type="range"
            class="slider"
            disabled=disabled
            aria-label=label
            aria-valuetext=move || value_text.map(|text| text.run(value.get()))
            min=track_min.to_string()
            max=track_max.to_string()
            step=track_step.to_string()
            style=move || format!("--fill: {}%", fill_percent(to_track(value.get()), track_min, track_max))
            prop:value=move || to_track(value.get()).to_string()
            on:input=move |event| {
                if let Some(parsed) = parse(&event) {
                    on_input.run(parsed);
                }
            }
            on:change=move |event| {
                if let (Some(commit), Some(parsed)) = (on_commit, parse(&event)) {
                    commit.run(parsed);
                }
            }
        />
    }
}

#[expect(clippy::float_arithmetic, reason = "logarithmic mapping of the range")]
fn log_position(level: f64, min: f64, max: f64) -> f64 {
    ((level.max(min) / min).log(max / min) * TRACK_STEPS).round()
}

#[expect(clippy::float_arithmetic, reason = "logarithmic mapping of the range")]
fn log_level(position: f64, min: f64, max: f64) -> f64 {
    min * (max / min).powf(position / TRACK_STEPS)
}

#[expect(
    clippy::float_arithmetic,
    reason = "the filled share of the track is a ratio of the value range"
)]
fn fill_percent(value: f64, min: f64, max: f64) -> f64 {
    if max > min {
        ((value - min) / (max - min) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{fill_percent, log_level, log_position};

    #[test]
    fn log_mapping_round_trips_within_a_track_step() {
        let (min, max) = (20.0, 20_000.0);
        assert!((log_position(min, min, max) - 0.0).abs() < f64::EPSILON);
        assert!((log_position(max, min, max) - 1000.0).abs() < f64::EPSILON);
        assert!((log_position(1.0, min, max) - 0.0).abs() < f64::EPSILON);
        for level in [20.0, 100.0, 440.0, 1000.0, 8000.0, 20_000.0] {
            let back = log_level(log_position(level, min, max), min, max);
            assert!(
                (back / level).ln().abs() < 0.004,
                "{level} came back as {back}"
            );
        }
    }

    #[test]
    fn fill_percent_clamps_and_handles_an_empty_range() {
        assert!((fill_percent(5.0, 0.0, 10.0) - 50.0).abs() < f64::EPSILON);
        assert!((fill_percent(-1.0, 0.0, 10.0) - 0.0).abs() < f64::EPSILON);
        assert!((fill_percent(11.0, 0.0, 10.0) - 100.0).abs() < f64::EPSILON);
        assert!((fill_percent(3.0, 3.0, 3.0) - 0.0).abs() < f64::EPSILON);
        assert!((fill_percent(3.0, 4.0, 2.0) - 0.0).abs() < f64::EPSILON);
    }
}
