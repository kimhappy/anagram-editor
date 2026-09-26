use leptos::prelude::*;

use crate::{
    components::{
        icon::{Icon, IconKind},
        marquee::Marquee,
        ui::{Button, ButtonVariant, IconButton, InlineField, SlotField, Tabs, static_tabs},
    },
    model::{
        library::PresetRef,
        mode::{Mode, SceneControl},
    },
    protocol::name::PRESET_NAME,
    session::{Modal, Session, SidePanel},
};

#[component]
pub fn Header() -> impl IntoView {
    let session = Session::expect();
    let is_scene_mode = move || session.mode.get() == Mode::Scene;
    let is_stomp_mode = move || session.mode.get() == Mode::Stomp;

    view! {
        <header class="flex h-16 shrink-0 items-center gap-4 border-b pr-[calc(0.875rem-0.5px)] pl-5">
            <PresetTitle />
            <StatusLine />
            <div class="ml-auto flex shrink-0 items-center gap-3">
                <Show when=is_scene_mode>
                    <Tabs
                        options=static_tabs(&SceneControl::TABS)
                        value=session.scene_control
                        on_change=move |control| session.scene_control.set(control)
                    />
                    <SceneStepper />
                </Show>
                <Show when=is_stomp_mode>
                    <Button
                        variant=ButtonVariant::Outline
                        class="h-10"
                        attr:title="Assign knobs, footswitches and the expression pedal"
                        on:click=move |_| session.ui.open(Modal::Bindings)
                    >
                        "Stomp settings"
                    </Button>
                </Show>
                <Tabs
                    options=static_tabs(&Mode::TABS)
                    value=session.mode
                    on_change=move |mode| session.set_mode(mode)
                />
                <div class="flex items-center gap-1 border-l pl-3">
                    <ToggleIconButton
                        kind=IconKind::AudioWaveform
                        label="Tuner"
                        title="Open or close the tuner on the Anagram"
                        is_active=session.ui.is_tuner_open
                        on_click=move |()| session.toggle_tuner()
                    />
                    <ToggleIconButton
                        kind=IconKind::FolderOpen
                        label="User files"
                        title="Cabinet IRs, neural models and plugins on the Anagram"
                        is_active=session.ui.is_open(Modal::UserFiles)
                        on_click=move |()| session.ui.open(Modal::UserFiles)
                    />
                    <ToggleIconButton
                        kind=IconKind::SlidersHorizontal
                        label="Preset settings"
                        title="MIDI Out of this preset"
                        is_active=Signal::derive(move || session.ui.panel.get() == SidePanel::PresetSettings)
                        on_click=move |()| {
                            session.ui.panel.update(|panel| {
                                *panel = if *panel == SidePanel::PresetSettings {
                                    SidePanel::Browser
                                } else {
                                    SidePanel::PresetSettings
                                };
                            });
                        }
                    />
                    <IconButton
                        kind=IconKind::Settings
                        label="Device settings"
                        attr:title="Settings of the Anagram"
                        on:click=move |_| session.ui.open(Modal::DeviceSettings)
                    />
                </div>
            </div>
        </header>
    }
}

#[component]
fn ToggleIconButton(
    kind: IconKind,
    label: &'static str,
    title: &'static str,
    #[prop(into)] is_active: Signal<bool>,
    #[prop(into)] on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <IconButton
            kind
            class="data-[active=true]:bg-accent data-[active=true]:text-foreground"
            attr:data-active=move || is_active.get().to_string()
            label
            attr:aria-pressed=move || is_active.get().to_string()
            attr:title=title
            on:click=move |_| on_click.run(())
        />
    }
}

#[component]
fn StatusLine() -> impl IntoView {
    let session = Session::expect();
    view! {
        <div role="status" aria-live="polite" class="text-muted-foreground flex min-w-0 grow basis-96 items-center gap-2 text-sm">
            {move || {
                let is_applying = session.link.audition.with(|audition| audition.in_flight);
                session
                    .link.busy
                    .get()
                    .or_else(|| is_applying.then(|| "Applying edits…".to_owned()))
                    .map(|text| {
                        view! {
                            <span class="flex min-w-0 items-center gap-1.5">
                                <Icon kind=IconKind::LoaderCircle class="shrink-0 animate-spin" />
                                <Marquee text class="max-w-96" />
                            </span>
                        }
                    })
            }}
            {move || {
                session.notice.get().map(|notice| {
                    let is_error = notice.is_error;
                    let class = if is_error { "text-destructive flex min-w-0 items-center gap-1.5" } else { "flex min-w-0 items-center gap-1.5" };
                    let action = notice.action;
                    view! {
                        <span class="flex min-w-0 items-center gap-2">
                            <button type="button" class=class title="Dismiss" on:click=move |_| session.clear_notice()>
                                {is_error.then(|| view! { <Icon kind=IconKind::TriangleAlert class="shrink-0" /> })}
                                <Marquee text=notice.text class="max-w-96" />
                            </button>
                            {action.map(|action| view! {
                                <button
                                    type="button"
                                    class="text-foreground hover:bg-accent shrink-0 rounded-md border px-2 py-0.5 text-xs"
                                    on:click=move |_| session.run_notice_action(action)
                                >
                                    {action.label()}
                                </button>
                            })}
                        </span>
                    }
                })
            }}
        </div>
    }
}

#[component]
fn PresetTitle() -> impl IntoView {
    let session = Session::expect();
    let name = Signal::derive(move || session.draft.with(|preset| preset.name().to_owned()));
    let area_label = move || {
        let area = session.current.get().area;
        session.library.with(|library| library.label(area))
    };
    let can_save = Signal::derive(move || {
        session.is_dirty.get() && !session.is_read_only() && session.is_idle()
    });

    view! {
        <div class="flex shrink-0 items-center gap-3">
            <div class="mr-7 grid w-20 shrink-0 justify-items-center gap-1 pt-1 leading-none">
                <span class="text-muted-foreground max-w-full truncate text-xs font-normal tracking-wide uppercase">
                    {area_label}
                </span>
                <SlotField
                    value=Signal::derive(move || session.current.get().slot)
                    label="Preset number"
                    class="text-lg"
                    on_commit=move |slot| {
                        let current = session.current.get_untracked();
                        if slot != current.slot {
                            session.load(PresetRef { area: current.area, slot });
                        }
                    }
                />
            </div>
            <div class="flex h-9 w-80 items-center">
                <InlineField
                    text=name
                    label="Preset name"
                    maxlength=PRESET_NAME.max_length
                    class="-ml-2 h-9 w-[calc(100%+0.5rem)] px-2 text-left text-lg font-medium"
                    on_commit=move |typed: String| session.rename(&typed)
                >
                    <div class=move || if session.is_dirty.get() { "italic" } else { "" }>
                        <Marquee text=name />
                    </div>
                </InlineField>
            </div>
            <IconButton
                kind=IconKind::Save
                label="Save preset"
                attr:title="Write the preset to its slot on the Anagram"
                disabled=Signal::derive(move || !can_save.get())
                on:click=move |_| session.save()
            />
        </div>
    }
}

#[component]
fn SceneStepper() -> impl IntoView {
    let session = Session::expect();
    let scene_name = move || {
        let scene = session.scene.get();
        session
            .draft
            .with(|preset| preset.scene_name(scene).map(str::to_owned))
    };
    view! {
        <div class="flex h-10 items-center rounded-md border">
            <IconButton
                kind=IconKind::ChevronLeft
                class="h-full w-10 rounded-r-none"
                label="Previous scene"
                on:click=move |_| session.set_scene(session.scene.get_untracked().previous())
            />
            <SlotField
                value=session.scene
                label="Scene number"
                class="w-16 text-base"
                on_commit=move |slot| session.set_scene(slot)
            />
            <Show when=move || scene_name().is_some()>
                <span class="text-muted-foreground max-w-32 truncate text-sm">{scene_name}</span>
            </Show>
            <IconButton
                kind=IconKind::ChevronRight
                class="h-full w-10 rounded-l-none"
                label="Next scene"
                on:click=move |_| session.set_scene(session.scene.get_untracked().next())
            />
        </div>
    }
}
