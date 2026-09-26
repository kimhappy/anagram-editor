use leptos::prelude::*;

use super::{CommitInput, Slider};

const FIELD_CLASS: &str = "hover:bg-accent focus-within:bg-accent focus-within:ring-ring/50 flex h-7 shrink-0 items-center rounded-md px-1 transition-[color,background-color,box-shadow] focus-within:ring-3";
const NUMBER_CLASS: &str = "field-sizing-content h-full min-w-[2ch] bg-transparent text-right text-[0.8125rem] font-normal tabular-nums outline-none";

#[derive(Clone, Copy)]
pub struct RangeText {
    pub number: Callback<f64, String>,
    pub unit: Callback<f64, String>,
    pub parse: Callback<(String, f64), Option<f64>>,
}

#[component]
pub fn RangeField(
    bounds: (f64, f64, f64),
    #[prop(optional)] logarithmic: bool,
    text: RangeText,
    #[prop(into)] value: Signal<f64>,
    #[prop(into)] on_input: Callback<f64>,
    #[prop(into)] on_commit: Callback<f64>,
    #[prop(into)] label: String,
    #[prop(into)] header_class: String,
    children: Children,
) -> impl IntoView {
    let (min, max, _) = bounds;
    let value_text = Callback::new(move |shown: f64| {
        format!("{} {}", text.number.run(shown), text.unit.run(shown))
            .trim_end()
            .to_owned()
    });
    let commit = move |typed: String| {
        if let Some(parsed) = text.parse.run((typed, value.get_untracked())) {
            on_commit.run(parsed.clamp(min, max));
        }
    };
    let field_label = format!("{label} value");
    view! {
        <div class=header_class>
            {children()}
            <label class=FIELD_CLASS>
                <CommitInput
                    shown=Signal::derive(move || text.number.run(value.get()))
                    class=NUMBER_CLASS
                    attr:inputmode="decimal"
                    attr:aria-label=field_label
                    on_commit=commit
                />
                <span class="text-muted-foreground shrink-0 pl-0.5 text-[0.8125rem] empty:hidden">
                    {move || text.unit.run(value.get())}
                </span>
            </label>
        </div>
        <div class="flex items-center">
            <Slider bounds value logarithmic label value_text on_input on_commit />
        </div>
    }
}
