use leptos::prelude::*;
use tw_merge::tw_merge;

use super::{SettingRange, draft::SettingsDraft};
use crate::{
    components::ui::{ChoiceSelect, CommitInput, Field, INPUT_CLASS},
    protocol::{
        hid::settings::{
            audio::{INPUT_GAIN_BOUNDS, INPUT_GAIN_DB, INPUT_GAIN_PRESETS, InputGainPreset},
            general::{BPM_BOUNDS, BRIGHTNESS_BOUNDS, BRIGHTNESS_LEVELS},
        },
        name::INPUT_GAIN_NAME,
    },
};

#[component]
pub fn DisplayTab(draft: SettingsDraft) -> impl IntoView {
    let general = move || draft.with(|settings| settings.general);
    let set_brightness = move |level: f64| {
        let level = BRIGHTNESS_LEVELS
            .rev()
            .find(|candidate| f64::from(*candidate) <= level)
            .unwrap_or(*BRIGHTNESS_LEVELS.start());
        draft.update(|settings| settings.general.brightness = level);
    };
    view! {
        <div class="grid grid-cols-2 gap-x-8 gap-y-5">
            <SettingRange
                draft
                label="Display brightness"
                bounds=BRIGHTNESS_BOUNDS
                value=Signal::derive(move || f64::from(general().brightness))
                on_change=set_brightness
            />
            <SettingRange
                draft
                label="Tempo"
                bounds=BPM_BOUNDS
                value=Signal::derive(move || general().bpm)
                on_change=move |value| draft.update(|settings| settings.general.bpm = value)
            />
        </div>
    }
}

#[component]
pub fn InputGainTab(draft: SettingsDraft) -> impl IntoView {
    let presets = Signal::derive(move || {
        draft.with(|settings| {
            settings
                .input_gain
                .presets
                .iter()
                .enumerate()
                .map(|(index, preset)| (index, preset.name.clone()))
                .collect::<Vec<_>>()
        })
    });
    let choose_active =
        move |index: usize| draft.update(|settings| settings.input_gain.active = index);
    view! {
        <div class="grid gap-5">
            <Field label="Active input gain">
                <ChoiceSelect
                    choices=presets
                    chosen=Signal::derive(move || Some(draft.with(|settings| settings.input_gain.active)))
                    label="Active input gain"
                    on_choose=choose_active
                />
            </Field>
            <div class="grid grid-cols-2 gap-x-8 gap-y-5">
                {(0..INPUT_GAIN_PRESETS)
                    .map(|index| view! { <InputGainRow index draft /> })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
fn InputGainRow(index: usize, draft: SettingsDraft) -> impl IntoView {
    let stored = move || draft.with(|settings| settings.input_gain.presets.get(index).cloned());
    let update = move |change: Box<dyn FnOnce(&mut InputGainPreset)>| {
        draft.update(|settings| {
            if let Some(preset) = settings.input_gain.presets.get_mut(index) {
                change(preset);
            }
        });
    };
    let rename = move |typed: String| {
        if let Some(accepted) = INPUT_GAIN_NAME.accept(&typed) {
            update(Box::new(move |preset| preset.name = accepted));
        }
    };
    let set_db = move |level: f64| {
        let db = INPUT_GAIN_DB
            .rev()
            .find(|candidate| f64::from(*candidate) <= level)
            .unwrap_or(*INPUT_GAIN_DB.start());
        update(Box::new(move |preset| preset.db = db));
    };
    view! {
        <SettingRange
            draft
            label=format!("Gain of input gain {}", index + 1)
            bounds=INPUT_GAIN_BOUNDS
            value=Signal::derive(move || stored().map_or_default(|preset| f64::from(preset.db)))
            on_change=set_db
        >
            <CommitInput
                shown=Signal::derive(move || stored().map_or_default(|preset| preset.name))
                class=tw_merge!(INPUT_CLASS, "h-8 min-w-0 flex-1")
                attr:aria-label=format!("Name of input gain {}", index + 1)
                attr:maxlength=INPUT_GAIN_NAME.max_length.to_string()
                on_commit=rename
            />
        </SettingRange>
    }
}
