mod audio;
mod draft;
mod firmware;
mod general;
mod midi;

use leptos::{ev, prelude::*};

use self::{
    audio::{EqTab, MixerTab},
    draft::SettingsDraft,
    firmware::FirmwareTab,
    general::{DisplayTab, InputGainTab},
    midi::{BindingsTab, MidiTab},
};
use super::screen::{Shortcut, is_typing};
use crate::{
    components::ui::{Button, Dialog, RangeField, RangeText, Tabs, static_tabs},
    model::unit::{self, Unit},
    protocol::hid::settings::{DeviceSettings, SettingsChange, value::Bounds},
    session::{Modal, Session},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SettingsTab {
    #[default]
    Midi,
    Bindings,
    Display,
    GlobalEq,
    Mixer,
    InputGain,
    Firmware,
}

const TABS: &[(SettingsTab, &str)] = &[
    (SettingsTab::Midi, "MIDI"),
    (SettingsTab::Bindings, "Bindings"),
    (SettingsTab::Display, "Display & tempo"),
    (SettingsTab::GlobalEq, "Global EQ"),
    (SettingsTab::Mixer, "Mixer"),
    (SettingsTab::InputGain, "Input gain"),
    (SettingsTab::Firmware, "Firmware"),
];

#[component]
pub fn DeviceSettingsDialog() -> impl IntoView {
    let session = Session::expect();
    let opened = RwSignal::new(DeviceSettings::default());
    let draft = SettingsDraft::new();
    let tab = RwSignal::new(SettingsTab::default());
    let is_open = session.ui.is_open(Modal::DeviceSettings);
    Effect::new(move |_| {
        if is_open.get() {
            let live = session.device_settings.get_untracked();
            opened.set(live.clone());
            draft.start_over(live);
        }
    });
    Effect::watch(
        move || session.device_settings.get(),
        move |live, _, _| {
            if is_open.get_untracked() {
                let edits =
                    opened.with_untracked(|from| draft.with_untracked(|to| from.diff(to).changes));
                opened.set(live.clone());
                draft.replace(live.with_changes(&edits));
            }
        },
        false,
    );
    let change = Memo::new(move |_| opened.with(|from| draft.with(|to| from.diff(to))));
    let close = move |()| {
        let is_confirmed = change.with_untracked(SettingsChange::is_empty)
            || crate::host::confirm("Close without applying the changed settings?");
        if is_confirmed {
            session.ui.close();
        }
    };
    let apply = move |_| {
        let is_confirmed = !change.with_untracked(|change| change.needs_restart)
            || crate::host::confirm(
                "Apart from display brightness and input gain names, the Anagram reads its settings only when it starts. Restart it to apply these settings? Audio stops for about 10 seconds.",
            );
        if is_confirmed {
            session.apply_device_settings(&opened.get_untracked(), &draft.get_untracked());
        }
    };
    let shortcuts = window_event_listener(ev::keydown, move |event| {
        if !is_open.get_untracked() || is_typing(&event) {
            return;
        }
        match Shortcut::of(&event) {
            Some(Shortcut::Undo) => draft.undo(),
            Some(Shortcut::Redo) => draft.redo(),
            _ => return,
        }
        event.prevent_default();
    });
    on_cleanup(move || shortcuts.remove());

    view! {
        <Dialog is_open on_close=close title="Anagram settings" class="h-[min(46rem,90vh)] max-w-3xl gap-5 overflow-hidden">
            <Tabs options=static_tabs(TABS) value=tab on_change=move |picked| tab.set(picked) class="w-full" />
            <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2 [scrollbar-gutter:stable]">
                {move || match tab.get() {
                    SettingsTab::Midi => view! { <MidiTab draft /> }.into_any(),
                    SettingsTab::Bindings => view! { <BindingsTab draft /> }.into_any(),
                    SettingsTab::Display => view! { <DisplayTab draft /> }.into_any(),
                    SettingsTab::GlobalEq => view! { <EqTab draft /> }.into_any(),
                    SettingsTab::Mixer => view! { <MixerTab draft /> }.into_any(),
                    SettingsTab::InputGain => view! { <InputGainTab draft /> }.into_any(),
                    SettingsTab::Firmware => view! { <FirmwareTab /> }.into_any(),
                }}
            </div>
            <div class="flex h-9 shrink-0 items-center justify-end gap-2">
                <span class="text-muted-foreground mr-auto text-xs">
                    {move || session.link.busy.get()}
                </span>
                <Show when=move || tab.get() != SettingsTab::Firmware>
                    <Button
                        disabled=Signal::derive(move || !session.is_idle() || change.with(SettingsChange::is_empty))
                        on:click=apply
                    >
                        {move || if change.with(|change| change.needs_restart) { "Apply and restart" } else { "Apply" }}
                    </Button>
                </Show>
            </div>
        </Dialog>
    }
}

fn bounds_text(bounds: Bounds) -> RangeText {
    let unit = Unit::parse(bounds.unit);
    let number_unit = unit.clone();
    let label_unit = unit.clone();
    RangeText {
        number: Callback::new(move |shown: f64| {
            unit::reading(&number_unit, shown, bounds.precision).number
        }),
        unit: Callback::new(move |shown: f64| {
            unit::reading(&label_unit, shown, bounds.precision).unit
        }),
        parse: Callback::new(move |(typed, shown): (String, f64)| {
            unit::parse_typed(&unit, &typed, shown).map(|value| bounds.snap(value))
        }),
    }
}

#[component]
fn SettingRange(
    #[prop(into)] label: String,
    bounds: Bounds,
    #[prop(into)] value: Signal<f64>,
    draft: SettingsDraft,
    #[prop(into)] on_change: Callback<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let on_input = Callback::new(move |raw: f64| {
        draft.begin_gesture();
        on_change.run(bounds.snap(raw));
    });
    let on_commit = Callback::new(move |raw: f64| {
        on_change.run(bounds.snap(raw));
        draft.end_gesture();
    });
    view! {
        <div class="grid min-w-0 gap-1">
            <RangeField
                bounds=(bounds.min, bounds.max, bounds.step)
                logarithmic=bounds.logarithmic
                text=bounds_text(bounds)
                value
                on_input
                on_commit
                label=label.clone()
                header_class="flex items-center justify-between gap-2 text-sm"
            >
                {children.map_or_else(|| view! { <span>{label}</span> }.into_any(), |children| children())}
            </RangeField>
        </div>
    }
}
