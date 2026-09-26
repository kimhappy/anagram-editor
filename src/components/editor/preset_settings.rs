use leptos::prelude::*;

use super::{
    midi_out_editor::MidiOutEditor,
    side_panel::{BackButton, SidePanelHeader},
};
use crate::{
    components::ui::SlotField,
    session::{Session, SidePanel},
};

#[component]
pub fn PresetSettingsPanel() -> impl IntoView {
    let session = Session::expect();
    let is_read_only = Signal::derive(move || session.is_read_only());

    view! {
        <aside class="bg-sidebar text-sidebar-foreground flex min-h-0 flex-col border-l">
            <SidePanelHeader title="Preset settings">
                <BackButton label="Back to blocks" on_click=move |()| session.ui.panel.set(SidePanel::Browser) />
            </SidePanelHeader>
            <div
                class=move || {
                    format!(
                        "grid min-h-0 flex-1 content-start gap-8 overflow-y-auto p-4 {}",
                        if is_read_only.get() { "opacity-50" } else { "" }
                    )
                }
                inert=move || is_read_only.get()
            >
                <PresetMidiOutSection />
                <SceneMidiOutSection />
            </div>
        </aside>
    }
}

#[component]
fn PresetMidiOutSection() -> impl IntoView {
    let session = Session::expect();
    let messages = Signal::derive(move || {
        session
            .draft
            .with(|preset| preset.midi_out().on_preset.clone())
    });
    view! {
        <section class="grid gap-3">
            <h3 class="text-sm font-medium">"MIDI Out on preset change"</h3>
            <MidiOutEditor messages on_change=move |changed| session.set_preset_midi_out(changed) />
        </section>
    }
}

#[component]
fn SceneMidiOutSection() -> impl IntoView {
    let session = Session::expect();
    let scene = RwSignal::new(session.scene.get_untracked());
    let messages = Signal::derive(move || {
        let chosen = scene.get();
        session.draft.with(|preset| {
            preset
                .midi_out()
                .on_scene
                .get(&chosen)
                .cloned()
                .unwrap_or_default()
        })
    });
    view! {
        <section class="grid gap-3">
            <div class="flex items-center justify-between gap-2">
                <h3 class="text-sm font-medium">"MIDI Out on scene change"</h3>
                <SlotField value=scene label="Scene" class="w-16 text-sm" on_commit=move |slot| scene.set(slot) />
            </div>
            <MidiOutEditor
                messages
                on_change=move |changed| session.set_scene_midi_out(scene.get_untracked(), changed)
            />
        </section>
    }
}
