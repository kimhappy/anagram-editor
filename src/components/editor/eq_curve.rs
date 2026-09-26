use leptos::{
    ev::{PointerEvent, WheelEvent},
    html,
    prelude::*,
};

use crate::model::eq::{
    EqPoint, GAIN_RANGE, fraction_of, freq_at, gain_at, gain_fraction, response_db,
};

const VIEW_WIDTH: f64 = 1000.0;
const VIEW_HEIGHT: f64 = 360.0;
const CURVE_STEPS: u32 = 240;
const WIDTH_STEP: f64 = 0.02;
const FREQ_TICKS: [(f64, &str); 9] = [
    (50.0, "50"),
    (100.0, "100"),
    (200.0, "200"),
    (500.0, "500"),
    (1000.0, "1k"),
    (2000.0, "2k"),
    (5000.0, "5k"),
    (10_000.0, "10k"),
    (20_000.0, "20k"),
];
const GAIN_TICKS: [f64; 5] = [12.0, 6.0, 0.0, -6.0, -12.0];
const HANDLE_CLASS: &str = "absolute flex size-6 -translate-x-1/2 -translate-y-1/2 cursor-grab items-center justify-center rounded-full border text-[0.625rem] font-medium tabular-nums shadow-sm outline-none transition-[background-color,opacity] focus-visible:ring-3 focus-visible:ring-ring/50 data-[selected=true]:bg-primary data-[selected=true]:text-primary-foreground data-[selected=false]:bg-background data-[active=false]:opacity-40";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqHandle {
    pub label: &'static str,
    pub mark: &'static str,
    pub point: EqPoint,
    pub freq_range: (f64, f64),
    pub is_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Drag {
    index: usize,
    pointer_id: i32,
    point: EqPoint,
}

#[expect(clippy::float_arithmetic, reason = "fractions of the view box")]
fn curve_path(points: &[EqPoint]) -> String {
    (0..=CURVE_STEPS)
        .map(|step| {
            let fraction = f64::from(step) / f64::from(CURVE_STEPS);
            let db = response_db(points, freq_at(fraction));
            let command = if step == 0 { 'M' } else { 'L' };
            format!(
                "{command}{:.1} {:.1}",
                fraction * VIEW_WIDTH,
                gain_fraction(db) * VIEW_HEIGHT
            )
        })
        .collect::<Vec<_>>()
        .concat()
}

#[expect(clippy::float_arithmetic, reason = "fractions of the view box")]
fn filled_path(curve: &str) -> String {
    let zero = gain_fraction(0.0) * VIEW_HEIGHT;
    format!("{curve}L{VIEW_WIDTH} {zero:.1}L0 {zero:.1}Z")
}

#[expect(clippy::float_arithmetic, reason = "rounding to the shown precision")]
fn rounded(value: f64, step: f64) -> f64 {
    (value / step).round() * step
}

fn freq_step(freq: f64) -> f64 {
    if freq >= 1000.0 { 10.0 } else { 1.0 }
}

#[expect(
    clippy::float_arithmetic,
    reason = "pointer position within the surface"
)]
fn dragged(handle: &EqHandle, surface: &web_sys::Element, event: &PointerEvent) -> EqPoint {
    let rect = surface.get_bounding_client_rect();
    let across = (event.client_x() - rect.left()) / rect.width().max(1.0);
    let down = (event.client_y() - rect.top()) / rect.height().max(1.0);
    let (low, high) = handle.freq_range;
    let freq = freq_at(across).clamp(low, high);
    EqPoint {
        freq: rounded(freq, freq_step(freq)).clamp(low, high),
        gain: rounded(gain_at(down), 0.1).clamp(-GAIN_RANGE, GAIN_RANGE),
        ..handle.point
    }
}

#[expect(clippy::float_arithmetic, reason = "percentages for CSS")]
fn position_style(point: &EqPoint) -> String {
    format!(
        "left: {:.2}%; top: {:.2}%",
        fraction_of(point.freq) * 100.0,
        gain_fraction(point.gain) * 100.0
    )
}

#[expect(clippy::float_arithmetic, reason = "percentages for CSS")]
fn percent(fraction: f64) -> String {
    format!("{:.2}%", fraction * 100.0)
}

#[expect(clippy::float_arithmetic, reason = "fractions of the view box")]
fn view_x(freq: f64) -> String {
    format!("{:.1}", fraction_of(freq) * VIEW_WIDTH)
}

#[expect(clippy::float_arithmetic, reason = "fractions of the view box")]
fn view_y(gain: f64) -> String {
    format!("{:.1}", gain_fraction(gain) * VIEW_HEIGHT)
}

#[expect(clippy::float_arithmetic, reason = "one wheel notch of width")]
fn widened(width: f64, is_wider: bool) -> f64 {
    let step = if is_wider { WIDTH_STEP } else { -WIDTH_STEP };
    rounded(width + step, 0.01).clamp(0.0, 1.0)
}

fn is_zero(gain: f64) -> bool {
    gain.total_cmp(&0.0).is_eq()
}

#[derive(Clone, Copy)]
struct Surface {
    node: NodeRef<html::Div>,
    drag: StoredValue<Option<Drag>>,
    handles: Signal<Vec<EqHandle>>,
    selected: RwSignal<usize>,
    on_input: Callback<(usize, EqPoint)>,
    on_commit: Callback<(usize, EqPoint)>,
    disabled: Signal<bool>,
}

impl Surface {
    fn handle(self, index: usize) -> Option<EqHandle> {
        self.handles.with(|handles| handles.get(index).copied())
    }

    fn grab(self, index: usize, event: &PointerEvent) {
        self.selected.set(index);
        if self.disabled.get_untracked() || event.button() != 0 {
            return;
        }
        let (Some(current), Some(element)) = (self.handle(index), self.node.get_untracked()) else {
            return;
        };
        event.prevent_default();
        element
            .set_pointer_capture(event.pointer_id())
            .unwrap_or_default();
        self.drag.set_value(Some(Drag {
            index,
            pointer_id: event.pointer_id(),
            point: current.point,
        }));
    }

    fn dragging(self, event: &PointerEvent) -> Option<Drag> {
        self.drag
            .get_value()
            .filter(|active| active.pointer_id == event.pointer_id())
    }

    fn track(self, event: &PointerEvent) {
        let Some(active) = self.dragging(event) else {
            return;
        };
        let (Some(handle), Some(element)) = (self.handle(active.index), self.node.get_untracked())
        else {
            return;
        };
        let point = dragged(&handle, &element, event);
        if point != active.point {
            self.drag.set_value(Some(Drag { point, ..active }));
            self.on_input.run((active.index, point));
        }
    }

    fn finish(self, event: &PointerEvent) {
        if let Some(active) = self.dragging(event) {
            self.drag.set_value(None);
            self.on_commit.run((active.index, active.point));
        }
    }

    fn widen(self, event: &WheelEvent) {
        let index = self.selected.get_untracked();
        let Some(handle) = self
            .handle(index)
            .filter(|_| !self.disabled.get_untracked())
        else {
            return;
        };
        event.prevent_default();
        let width = widened(handle.point.width, event.delta_y() < 0.0);
        self.on_commit.run((
            index,
            EqPoint {
                width,
                ..handle.point
            },
        ));
    }

    fn flatten(self, index: usize) {
        if let Some(current) = self
            .handle(index)
            .filter(|_| !self.disabled.get_untracked())
        {
            self.on_commit.run((
                index,
                EqPoint {
                    gain: 0.0,
                    ..current.point
                },
            ));
        }
    }
}

#[component]
pub fn EqCurve(
    #[prop(into)] handles: Signal<Vec<EqHandle>>,
    selected: RwSignal<usize>,
    #[prop(into)] on_input: Callback<(usize, EqPoint)>,
    #[prop(into)] on_commit: Callback<(usize, EqPoint)>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let surface = Surface {
        node: NodeRef::new(),
        drag: StoredValue::new(None),
        handles,
        selected,
        on_input,
        on_commit,
        disabled,
    };
    let curve = Memo::new(move |_| {
        let points: Vec<EqPoint> = handles.with(|handles| {
            handles
                .iter()
                .filter(|handle| handle.is_active)
                .map(|handle| handle.point)
                .collect()
        });
        curve_path(&points)
    });
    view! {
        <div
            node_ref=surface.node
            class=format!("bg-muted/40 relative touch-none select-none overflow-hidden rounded-md border {class}")
            on:pointermove=move |event| surface.track(&event)
            on:pointerup=move |event| surface.finish(&event)
            on:lostpointercapture=move |event| surface.finish(&event)
            on:wheel=move |event| surface.widen(&event)
        >
            <svg
                class="pointer-events-none absolute inset-0 size-full"
                viewBox=format!("0 0 {VIEW_WIDTH} {VIEW_HEIGHT}")
                preserveAspectRatio="none"
                aria-hidden="true"
            >
                <GridLines />
                <path d=move || filled_path(&curve.get()) class="fill-primary/10" />
                <path d=move || curve.get() class="stroke-primary fill-none" stroke-width="2" vector-effect="non-scaling-stroke" />
            </svg>
            <AxisLabels />
            <For
                each=move || 0..handles.with(Vec::len)
                key=|index| *index
                children=move |index| view! { <Handle surface index /> }
            />
        </div>
    }
}

#[component]
fn GridLines() -> impl IntoView {
    let verticals = FREQ_TICKS.iter().map(|(freq, _)| {
        let x = view_x(*freq);
        view! { <line x1=x.clone() x2=x y1="0" y2=VIEW_HEIGHT.to_string() class="stroke-border" vector-effect="non-scaling-stroke" /> }
    });
    let horizontals = GAIN_TICKS.iter().map(|gain| {
        let y = view_y(*gain);
        let stroke = if is_zero(*gain) { "stroke-muted-foreground/60" } else { "stroke-border" };
        view! { <line x1="0" x2=VIEW_WIDTH.to_string() y1=y.clone() y2=y class=stroke vector-effect="non-scaling-stroke" /> }
    });
    view! {
        {verticals.collect_view()}
        {horizontals.collect_view()}
    }
}

#[component]
fn AxisLabels() -> impl IntoView {
    let freqs = FREQ_TICKS.iter().map(|(freq, text)| {
        view! {
            <span
                class="text-muted-foreground pointer-events-none absolute bottom-0.5 -translate-x-1/2 text-[0.625rem] tabular-nums"
                style=format!("left: {}", percent(fraction_of(*freq).min(0.97)))
            >
                {*text}
            </span>
        }
    });
    let gains = GAIN_TICKS.iter().filter(|gain| !is_zero(**gain)).map(|gain| {
        view! {
            <span
                class="text-muted-foreground pointer-events-none absolute left-1 -translate-y-1/2 text-[0.625rem] tabular-nums"
                style=format!("top: {}", percent(gain_fraction(*gain)))
            >
                {format!("{gain:+}")}
            </span>
        }
    });
    view! {
        {freqs.collect_view()}
        {gains.collect_view()}
    }
}

#[component]
fn Handle(surface: Surface, index: usize) -> impl IntoView {
    let handle = move || surface.handle(index);
    view! {
        <button
            type="button"
            class=HANDLE_CLASS
            style=move || handle().map_or_default(|current| position_style(&current.point))
            data-selected=move || (surface.selected.get() == index).to_string()
            data-active=move || handle().is_some_and(|current| current.is_active).to_string()
            title=move || handle().map_or_default(|current| current.label)
            aria-label=move || handle().map_or_default(|current| current.label)
            on:pointerdown=move |event| surface.grab(index, &event)
            on:dblclick=move |_| surface.flatten(index)
        >
            {move || handle().map_or_default(|current| current.mark)}
        </button>
    }
}

#[component]
pub fn BandTabs(labels: Vec<&'static str>, selected: RwSignal<usize>) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-1" role="tablist">
            {labels
                .into_iter()
                .enumerate()
                .map(|(index, label)| view! {
                    <button
                        type="button"
                        role="tab"
                        class="hover:bg-accent data-[selected=true]:bg-primary data-[selected=true]:text-primary-foreground rounded-md border px-2 py-0.5 text-xs"
                        data-selected=move || (selected.get() == index).to_string()
                        aria-selected=move || (selected.get() == index).to_string()
                        on:click=move |_| selected.set(index)
                    >
                        {label}
                    </button>
                })
                .collect_view()}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{curve_path, rounded};
    use crate::model::eq::{EqPoint, Shape};

    #[test]
    fn a_flat_eq_draws_a_straight_line_through_zero_db() {
        let flat = EqPoint {
            shape: Shape::Peak,
            freq: 1000.0,
            gain: 0.0,
            width: 0.5,
        };
        let path = curve_path(&[flat]);
        assert!(path.starts_with("M0.0 180.0L"));
        assert!(path.ends_with("L1000.0 180.0"));
        assert!((rounded(1234.0, 10.0) - 1230.0).abs() < 1e-9);
    }
}
