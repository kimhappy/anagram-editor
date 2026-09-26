use leptos::prelude::*;

use super::{
    LABEL_CLASS, PARAM_CLASS,
    marks::{ParamLabel, ParamMark, SwitchState, bound_to},
};
use crate::{
    components::ui::{ChoiceSelect, RangeField, RangeText, Switch},
    model::{
        catalog::{ParamKind, ParamSpec},
        preset::Cell,
    },
    session::Session,
};

#[component]
pub fn ParamControl(cell: Cell, index: usize, spec: ParamSpec) -> impl IntoView {
    let session = Session::expect();
    let default = spec.default;
    let value = Signal::derive(move || {
        let scene = session.viewed_scene();
        session
            .draft
            .with(|preset| preset.value(cell, index, scene))
            .unwrap_or(default)
    });
    let mark = ParamMark {
        is_changed: Signal::derive(move || session.is_value_edited(cell, index)),
        is_scene_specific: Signal::derive(move || {
            session
                .draft
                .with(|preset| preset.is_value_scene_specific(cell, index))
        }),
        bound: bound_to(cell, spec.symbol.clone()),
    };
    let preview = Callback::new(move |new_value| session.set_value_preview(cell, index, new_value));
    let set = Callback::new(move |new_value| session.set_value(cell, index, new_value));

    let control = match spec.kind {
        ParamKind::Continuous { .. } | ParamKind::Integer => {
            view! { <RangeControl spec value preview set mark /> }.into_any()
        }
        ParamKind::Toggle => view! { <ToggleControl spec value set mark /> }.into_any(),
        ParamKind::Enum(_) => view! { <ChoiceControl spec value set mark /> }.into_any(),
        ParamKind::Opaque => view! { <OpaqueControl spec value mark /> }.into_any(),
    };
    view! { <div class=PARAM_CLASS>{control}</div> }
}

#[component]
pub fn RangeControl(
    spec: ParamSpec,
    value: Signal<f64>,
    preview: Callback<f64>,
    set: Callback<f64>,
    mark: ParamMark,
) -> impl IntoView {
    let bounds = (spec.min, spec.max, spec.step());
    let is_logarithmic = spec.is_logarithmic();
    let name = spec.name.clone();
    let text = {
        let number_spec = spec.clone();
        let unit_spec = spec.clone();
        RangeText {
            number: Callback::new(move |shown: f64| number_spec.reading(shown).number),
            unit: Callback::new(move |shown: f64| unit_spec.reading(shown).unit),
            parse: Callback::new(move |(typed, shown): (String, f64)| {
                spec.parse_typed(&typed, shown)
            }),
        }
    };
    view! {
        <RangeField
            bounds
            logarithmic=is_logarithmic
            text
            value
            on_input=preview
            on_commit=set
            label=name.clone()
            header_class=LABEL_CLASS
        >
            <ParamLabel name mark />
        </RangeField>
    }
}

#[component]
pub fn ToggleControl(
    spec: ParamSpec,
    value: Signal<f64>,
    set: Callback<f64>,
    mark: ParamMark,
) -> impl IntoView {
    let on_value = spec.toggle_value(true);
    let off_value = spec.toggle_value(false);
    let threshold = spec.min;
    let is_on = Signal::derive(move || value.get() > threshold);
    let label = spec.name.clone();
    view! {
        <div class=LABEL_CLASS>
            <ParamLabel name=spec.name mark />
        </div>
        <div class="flex items-center gap-2">
            <Switch
                checked=is_on
                label
                on:click=move |_| set.run(if is_on.get_untracked() { off_value } else { on_value })
            />
            <SwitchState checked=is_on class="text-muted-foreground text-sm select-none" />
        </div>
    }
}

#[component]
pub fn ChoiceControl(
    spec: ParamSpec,
    value: Signal<f64>,
    set: Callback<f64>,
    mark: ParamMark,
) -> impl IntoView {
    let options: Vec<(usize, String)> = spec.enum_labels().into_iter().enumerate().collect();
    let indexer = spec.clone();
    let chooser = spec.clone();
    let chosen = Signal::derive(move || indexer.enum_index(value.get()));
    view! {
        <div class=LABEL_CLASS>
            <ParamLabel name=spec.name.clone() mark />
        </div>
        <ChoiceSelect
            choices=Signal::stored(options)
            chosen
            label=spec.name
            on_choose=move |index: usize| {
                if let Some(point) = chooser.enum_value(index) {
                    set.run(point);
                }
            }
        />
    }
}

#[component]
pub fn OpaqueControl(spec: ParamSpec, value: Signal<f64>, mark: ParamMark) -> impl IntoView {
    view! {
        <div class=LABEL_CLASS>
            <ParamLabel name=spec.name mark />
        </div>
        <div class="text-muted-foreground flex h-9 items-center text-sm tabular-nums">
            {move || format!("{}", value.get())}
        </div>
    }
}
