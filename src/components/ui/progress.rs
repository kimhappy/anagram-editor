use leptos::prelude::*;

#[component]
pub fn ProgressBar(#[prop(into)] share: Signal<Option<f64>>) -> impl IntoView {
    let percent = move || share.get().map(percent_of);
    let style = move || {
        percent().map_or_else(
            || "width: 30%".to_owned(),
            |percent| format!("width: {percent:.1}%"),
        )
    };
    let class = move || {
        if share.get().is_some() {
            "bg-primary h-full rounded-full transition-[width] duration-300"
        } else {
            "bg-primary animate-sweep h-full rounded-full"
        }
    };
    view! {
        <div
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow=move || percent().map(|percent| format!("{percent:.0}"))
            class="bg-muted h-1.5 w-full overflow-hidden rounded-full"
        >
            <div class=class style=style></div>
        </div>
    }
}

#[expect(clippy::float_arithmetic, reason = "a share shown as a percentage")]
fn percent_of(share: f64) -> f64 {
    (share * 100.0).clamp(0.0, 100.0)
}
