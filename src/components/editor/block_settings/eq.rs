use std::sync::Arc;

use leptos::prelude::*;

use super::param_controls::ParamControl;
use crate::{
    components::editor::eq_curve::{BandTabs, EqCurve, EqHandle},
    model::{
        catalog::{BlockModel, ParamSpec},
        eq::{BAND_FIELDS, BandParams, EqPoint},
        preset::Cell,
    },
    session::Session,
};

const fn mark_of(index: usize) -> &'static str {
    match index {
        0 => "L",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        _ => "H",
    }
}

#[component]
pub fn BlockEq(cell: Cell, model: Arc<BlockModel>, bands: Vec<BandParams>) -> impl IntoView {
    let session = Session::expect();
    let selected = RwSignal::new(1_usize);
    let labels = bands.iter().map(|band| band.label).collect();
    let bands = StoredValue::new(bands);
    let reader = Arc::clone(&model);
    let read = move |index: usize| {
        let scene = session.viewed_scene();
        session
            .draft
            .with(|preset| preset.value(cell, index, scene))
            .or_else(|| reader.params.get(index).map(|spec| spec.default))
            .unwrap_or_default()
    };
    let ranges = Arc::clone(&model);
    let handles = Signal::derive(move || {
        bands.with_value(|bands| {
            bands
                .iter()
                .enumerate()
                .map(|(index, band)| EqHandle {
                    label: band.label,
                    mark: mark_of(index),
                    point: EqPoint {
                        shape: band.shape,
                        freq: read(band.freq),
                        gain: read(band.gain),
                        width: read(band.width),
                    },
                    freq_range: ranges
                        .params
                        .get(band.freq)
                        .map_or((20.0, 20_000.0), |spec| (spec.min, spec.max)),
                    is_active: band.enable.is_none_or(|enable| read(enable) > 0.5),
                })
                .collect::<Vec<_>>()
        })
    });
    let values_of = move |index: usize, point: EqPoint| {
        bands.with_value(|bands| {
            bands.get(index).map(|band| {
                [
                    (band.freq, point.freq),
                    (band.gain, point.gain),
                    (band.width, point.width),
                ]
            })
        })
    };
    let on_input = Callback::new(move |(index, point): (usize, EqPoint)| {
        if let Some(values) = values_of(index, point) {
            session.set_values_preview(cell, &values);
        }
    });
    let on_commit = Callback::new(move |(index, point): (usize, EqPoint)| {
        if let Some(values) = values_of(index, point) {
            session.set_values(cell, &values);
        }
    });
    let controls = move || {
        let band = bands.with_value(|bands| bands.get(selected.get()).copied())?;
        Some(
            [band.freq, band.gain, band.width]
                .into_iter()
                .zip(BAND_FIELDS)
                .filter_map(|(index, field)| {
                    let spec = ParamSpec {
                        name: field.to_owned(),
                        ..model.params.get(index).cloned()?
                    };
                    Some(view! { <ParamControl cell index spec /> })
                })
                .collect_view(),
        )
    };
    view! {
        <div class="grid h-full min-h-0 grid-cols-[minmax(0,1fr)_calc(3*7rem+2*0.75rem)] gap-4 px-4">
            <EqCurve
                handles
                selected
                on_input
                on_commit
                disabled=Signal::derive(move || session.is_read_only())
                class="h-full min-h-32"
            />
            <div class="grid content-center gap-6">
                <BandTabs labels selected />
                <div class="grid grid-cols-3 gap-x-3">{controls}</div>
            </div>
        </div>
    }
}
