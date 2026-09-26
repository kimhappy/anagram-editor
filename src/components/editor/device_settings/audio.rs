use leptos::prelude::*;

use super::{SettingRange, draft::SettingsDraft};
use crate::{
    components::{
        editor::eq_curve::{BandTabs, EqCurve, EqHandle},
        ui::{Button, ButtonSize, ButtonVariant, SwitchField},
    },
    model::eq::{EqPoint, Shape},
    protocol::hid::settings::{
        GlobalEq,
        audio::{
            EQ_GAIN_BOUNDS, EQ_WIDTH_BOUNDS, EqBand, EqBandId, MIXER_VOLUME_BOUNDS, MixerChannel,
            MixerOutput, StereoPair,
        },
    },
};

const fn shape_of(id: EqBandId) -> (Shape, &'static str) {
    match id {
        EqBandId::LowShelf => (Shape::LowShelf, "L"),
        EqBandId::Band1 => (Shape::Peak, "1"),
        EqBandId::Band2 => (Shape::Peak, "2"),
        EqBandId::Band3 => (Shape::Peak, "3"),
        EqBandId::Band4 => (Shape::Peak, "4"),
        EqBandId::HighShelf => (Shape::HighShelf, "H"),
    }
}

const fn handle_of(id: EqBandId, band: EqBand) -> EqHandle {
    let (shape, mark) = shape_of(id);
    let bounds = id.freq_bounds();
    EqHandle {
        label: id.label(),
        mark,
        point: EqPoint {
            shape,
            freq: band.freq,
            gain: band.gain,
            width: band.width,
        },
        freq_range: (bounds.min, bounds.max),
        is_active: true,
    }
}

#[component]
pub fn EqTab(draft: SettingsDraft) -> impl IntoView {
    let selected = RwSignal::new(1_usize);
    let handles = Signal::derive(move || {
        draft.with(|settings| {
            EqBandId::ALL
                .into_iter()
                .map(|id| handle_of(id, settings.eq.band(id)))
                .collect::<Vec<_>>()
        })
    });
    let apply = move |(index, point): (usize, EqPoint)| {
        if let Some(id) = EqBandId::ALL.get(index) {
            draft.update(|settings| {
                settings.eq.update(*id, |band| {
                    *band = EqBand {
                        freq: point.freq,
                        gain: point.gain,
                        width: point.width,
                    };
                });
            });
        }
    };
    let on_input = Callback::new(move |moved| {
        draft.begin_gesture();
        apply(moved);
    });
    let on_commit = Callback::new(move |moved| {
        apply(moved);
        draft.end_gesture();
    });
    view! {
        <div class="grid gap-4">
            <div class="flex items-center justify-end gap-3">
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Sm
                    on:click=move |_| draft.update(|settings| settings.eq = GlobalEq::default())
                >
                    "Reset EQ"
                </Button>
            </div>
            <EqCurve handles selected on_input on_commit class="h-56" />
            <BandTabs labels=EqBandId::ALL.iter().map(|id| id.label()).collect() selected />
            {move || EqBandId::ALL.get(selected.get()).copied().map(|id| view! { <EqBandFields id draft /> })}
        </div>
    }
}

#[component]
fn EqBandFields(id: EqBandId, draft: SettingsDraft) -> impl IntoView {
    let current_band = move || draft.with(|settings| settings.eq.band(id));
    let set = move |change: fn(&mut EqBand, f64)| {
        Callback::new(move |value: f64| {
            draft.update(|settings| settings.eq.update(id, |band| change(band, value)));
        })
    };
    view! {
        <div class="grid grid-cols-3 gap-4">
            <SettingRange
                draft
                label="Freq"
                bounds=id.freq_bounds()
                value=Signal::derive(move || current_band().freq)
                on_change=set(|band, value| band.freq = value)
            />
            <SettingRange
                draft
                label="Gain"
                bounds=EQ_GAIN_BOUNDS
                value=Signal::derive(move || current_band().gain)
                on_change=set(|band, value| band.gain = value)
            />
            <SettingRange
                draft
                label="Width"
                bounds=EQ_WIDTH_BOUNDS
                value=Signal::derive(move || current_band().width)
                on_change=set(|band, value| band.width = value)
            />
        </div>
    }
}

#[component]
pub fn MixerTab(draft: SettingsDraft) -> impl IntoView {
    view! {
        <div class="grid gap-5">
            <div class="grid grid-cols-2 gap-4">
                {MixerOutput::ALL.into_iter().map(|output| view! { <MixerRow output draft /> }).collect_view()}
            </div>
            <div class="grid grid-cols-2 gap-x-8">
                {StereoPair::ALL
                    .into_iter()
                    .map(|pair| {
                        view! {
                            <SwitchField
                                label=pair.label()
                                checked=Signal::derive(move || draft.with(|settings| settings.mixer.is_linked(pair)))
                                on_toggle=move |()| draft.update(|settings| {
                                    let is_linked = settings.mixer.is_linked(pair);
                                    settings.mixer.set_linked(pair, !is_linked);
                                })
                            />
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
fn MixerRow(output: MixerOutput, draft: SettingsDraft) -> impl IntoView {
    let current_channel = move || draft.with(|settings| settings.mixer.channel(output));
    let update = move |change: Box<dyn Fn(&mut MixerChannel)>| {
        draft.update(|settings| settings.mixer.update(output, |channel| change(channel)));
    };
    view! {
        <section class="grid gap-2 rounded-md border p-3">
            <SettingRange
                draft
                label=output.label()
                bounds=MIXER_VOLUME_BOUNDS
                value=Signal::derive(move || current_channel().volume)
                on_change=move |volume: f64| update(Box::new(move |channel| channel.volume = volume))
            />
            {output.follows_master_volume().then(|| view! {
                <SwitchField
                    label="Follow master"
                    checked=Signal::derive(move || current_channel().follows_master)
                    on_toggle=move |()| {
                        let follows = !current_channel().follows_master;
                        update(Box::new(move |channel| channel.follows_master = follows));
                    }
                />
            })}
        </section>
    }
}
