use leptos::prelude::*;

use super::draft::SettingsDraft;
use crate::{
    components::ui::{ChoiceSelect, Field, Switch, SwitchField},
    host,
    protocol::{
        actuator::Actuator,
        hid::settings::ThroughMode,
        midi::{ALLOWED_CCS, Channel},
    },
};

#[component]
pub fn MidiTab(draft: SettingsDraft) -> impl IntoView {
    let channels: Vec<(Option<Channel>, String)> = std::iter::once((None, "Omni".to_owned()))
        .chain(
            (1..=16)
                .filter_map(Channel::new)
                .map(|channel| (Some(channel), channel.to_string())),
        )
        .collect();
    let through_modes: Vec<(ThroughMode, String)> = ThroughMode::ALL
        .iter()
        .map(|mode| (*mode, mode.label().to_owned()))
        .collect();
    let choose_channel =
        move |channel: Option<Channel>| draft.update(|settings| settings.midi.channel = channel);
    let choose_through =
        move |mode: ThroughMode| draft.update(|settings| settings.midi.through = mode);
    let current_midi = move || draft.with(|settings| settings.midi);
    let toggle = move |change: fn(&mut crate::protocol::hid::settings::MidiSettings)| {
        move |()| draft.update(|settings| change(&mut settings.midi))
    };
    view! {
        <div class="grid gap-5">
            <div class="grid grid-cols-2 gap-x-8 gap-y-5">
                <Field label="MIDI In channel">
                    <ChoiceSelect
                        choices=Signal::stored(channels)
                        chosen=Signal::derive(move || Some(current_midi().channel))
                        label="MIDI In channel"
                        on_choose=choose_channel
                    />
                </Field>
                <Field label="MIDI Through">
                    <ChoiceSelect
                        choices=Signal::stored(through_modes)
                        chosen=Signal::derive(move || Some(current_midi().through))
                        label="MIDI Through"
                        on_choose=choose_through
                    />
                </Field>
                <SwitchField
                    label="Program Change Through"
                    checked=Signal::derive(move || current_midi().pc_through)
                    disabled=Signal::derive(move || !current_midi().is_pc_through_available())
                    on_toggle=toggle(|midi| midi.pc_through = !midi.pc_through)
                />
                <SwitchField
                    label="Ignore redundant PC"
                    checked=Signal::derive(move || current_midi().ignore_redundant_pc)
                    on_toggle=toggle(|midi| midi.ignore_redundant_pc = !midi.ignore_redundant_pc)
                />
                <Field label="Toggle logic">
                    <ChoiceSelect
                        choices=Signal::stored(vec![(false, "Value".to_owned()), (true, "Toggle".to_owned())])
                        chosen=Signal::derive(move || Some(current_midi().toggle_logic))
                        label="Toggle logic"
                        on_choose=move |toggles: bool| draft.update(|settings| settings.midi.toggle_logic = toggles)
                    />
                </Field>
            </div>
        </div>
    }
}

#[component]
pub fn BindingsTab(draft: SettingsDraft) -> impl IntoView {
    view! {
        <div class="grid gap-4">
            <div class="text-muted-foreground grid grid-cols-[5rem_auto_minmax(0,1fr)] items-center gap-x-3 text-xs">
                <span>"Control"</span>
                <span>"CC Enable"</span>
                <span>"CC"</span>
            </div>
            <div class="grid gap-3">
                {Actuator::ALL
                    .into_iter()
                    .map(|actuator| view! { <BindingCcRow actuator draft /> })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
fn BindingCcRow(actuator: Actuator, draft: SettingsDraft) -> impl IntoView {
    let options: Vec<u8> = std::iter::once(actuator.default_cc())
        .chain(ALLOWED_CCS.iter().copied())
        .collect();
    let choices: Vec<(u8, String)> = options.iter().map(|cc| (*cc, format!("CC {cc}"))).collect();
    let bindings = move || draft.with(|settings| settings.midi.bindings);
    let chosen = Signal::derive(move || Some(bindings().cc(actuator)));
    let is_enabled = Signal::derive(move || bindings().is_enabled(actuator));
    let choose = move |cc: u8| {
        let holder = bindings().holder_of(cc, actuator);
        let is_confirmed = holder.is_none_or(|other| {
            host::confirm(&format!(
                "CC {cc} is used by {}. Continue? {} goes back to its default CC {}.",
                other.label(),
                other.label(),
                other.default_cc()
            ))
        });
        if is_confirmed {
            draft.update(|settings| {
                settings.midi.bindings.assign(actuator, cc);
            });
        }
    };
    view! {
        <div class="grid grid-cols-[5rem_auto_minmax(0,1fr)] items-center gap-x-3 text-sm">
            <span>{actuator.label()}</span>
            <Switch
                checked=is_enabled
                label=format!("CC Enable for {}", actuator.label())
                on:click=move |_| draft.update(|settings| {
                    let enabled = settings.midi.bindings.is_enabled(actuator);
                    settings.midi.bindings.set_enabled(actuator, !enabled);
                })
            />
            <ChoiceSelect
                choices=Signal::stored(choices)
                chosen
                label=format!("{} CC", actuator.label())
                disabled=Signal::derive(move || !is_enabled.get())
                on_choose=choose
            />
        </div>
    }
}
