use std::sync::Arc;

use leptos::prelude::*;

use crate::{
    components::ui::ChoiceSelect,
    model::{
        binding::{
            AMP_MODEL_SYMBOL, BASS_CABINET_SYMBOL, GUITAR_CABINET_SYMBOL, PEDAL_MODEL_SYMBOL,
        },
        catalog::BlockModel,
        files::FileKind,
        preset::Cell,
    },
    session::Session,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickPotChoice {
    symbol: Option<String>,
    label: String,
}

impl QuickPotChoice {
    fn of(symbol: &str, model: &BlockModel) -> Self {
        let label = model
            .param_index(symbol)
            .and_then(|index| model.params.get(index))
            .map_or_else(|| host_symbol_label(symbol), |spec| spec.name.clone());
        Self {
            symbol: Some(symbol.to_owned()),
            label,
        }
    }
}

pub fn host_symbol_label(symbol: &str) -> String {
    match symbol {
        BASS_CABINET_SYMBOL | GUITAR_CABINET_SYMBOL => "Cabinet file".to_owned(),
        AMP_MODEL_SYMBOL | PEDAL_MODEL_SYMBOL => "Model".to_owned(),
        other => other.to_owned(),
    }
}

pub fn quickpot_choices(model: &BlockModel, current: Option<&str>) -> Vec<QuickPotChoice> {
    let block_default = QuickPotChoice {
        symbol: None,
        label: "Default".to_owned(),
    };
    let host = FileKind::for_block(&model.uri)
        .and_then(FileKind::quickpot_symbol)
        .map(|symbol| QuickPotChoice::of(symbol, model));
    let allowed = model
        .params
        .iter()
        .filter(|spec| spec.allowed_in_quickpot())
        .map(|spec| QuickPotChoice::of(&spec.symbol, model));
    let listed: Vec<QuickPotChoice> = std::iter::once(block_default)
        .chain(host)
        .chain(allowed)
        .collect();
    let unlisted = current
        .filter(|symbol| {
            !listed
                .iter()
                .any(|choice| choice.symbol.as_deref() == Some(*symbol))
        })
        .map(|symbol| QuickPotChoice::of(symbol, model));
    listed.into_iter().chain(unlisted).collect()
}

#[component]
pub fn QuickPotControl(cell: Cell, model: Arc<BlockModel>) -> impl IntoView {
    let session = Session::expect();
    let choices = Memo::new(move |_| {
        session.draft.with(|preset| {
            let current = preset
                .block(cell)
                .and_then(|block| block.quickpot.as_deref());
            quickpot_choices(&model, current)
        })
    });
    let chosen = Signal::derive(move || {
        Some(
            session
                .draft
                .with(|preset| preset.block(cell)?.quickpot.clone()),
        )
    });
    let options = Signal::derive(move || {
        choices.with(|choices| {
            choices
                .iter()
                .map(|choice| (choice.symbol.clone(), choice.label.clone()))
                .collect::<Vec<_>>()
        })
    });
    let choose = move |symbol: Option<String>| session.set_quickpot(cell, symbol.as_deref());
    view! {
        <Show when=move || choices.with(|choices| choices.len() > 1)>
            <label class="flex items-center gap-2 text-sm">
                <span class="select-none">"Quick pot"</span>
                <div class="w-36">
                    <ChoiceSelect
                        choices=options
                        chosen
                        label="Quick pot"
                        on_choose=choose
                    />
                </div>
            </label>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::{QuickPotChoice, quickpot_choices};
    use crate::{
        model::catalog::{BlockModel, ParamSpec},
        protocol::hid::plugin::port_flags,
    };

    fn model() -> BlockModel {
        let hidden = ParamSpec {
            flags: port_flags::CONTROL | port_flags::HIDDEN,
            ..ParamSpec::opaque("secret", Some("Secret"), 0.0)
        };
        BlockModel {
            params: vec![
                ParamSpec::opaque("gain", Some("Gain"), 0.5),
                hidden,
                ParamSpec::opaque("tone", Some("Tone"), 0.5),
            ],
            ..BlockModel::from_document("urn:test:block", None)
        }
    }

    fn choice(symbol: Option<&str>, label: &str) -> QuickPotChoice {
        QuickPotChoice {
            symbol: symbol.map(str::to_owned),
            label: label.to_owned(),
        }
    }

    #[test]
    fn quickpot_lists_the_block_default_then_allowed_params() {
        assert_eq!(
            quickpot_choices(&model(), None),
            vec![
                choice(None, "Default"),
                choice(Some("gain"), "Gain"),
                choice(Some("tone"), "Tone"),
            ]
        );
        assert_eq!(quickpot_choices(&model(), Some("gain")).len(), 3);
    }

    #[test]
    fn a_cabinet_block_offers_its_file_selector() {
        let cabinet = BlockModel::from_document("urn:darkglass-anagram:cabinet-bass", None);
        let choices = quickpot_choices(&cabinet, None);
        assert_eq!(
            choices.get(1),
            Some(&choice(Some(":bass-cab"), "Cabinet file"))
        );
        let amp = BlockModel::from_document("urn:darkglass:neural:amp", None);
        assert_eq!(
            quickpot_choices(&amp, None).get(1),
            Some(&choice(Some(":amp-model"), "Model"))
        );
    }

    #[test]
    fn quickpot_keeps_an_unlisted_current_symbol_last() {
        let choices = quickpot_choices(&model(), Some("secret"));
        assert_eq!(choices.last(), Some(&choice(Some("secret"), "Secret")));
        let host = quickpot_choices(&model(), Some(":guitar-cab"));
        assert_eq!(
            host.last(),
            Some(&choice(Some(":guitar-cab"), "Cabinet file"))
        );
        let unknown = quickpot_choices(&model(), Some("missing"));
        assert_eq!(unknown.last(), Some(&choice(Some("missing"), "missing")));
    }
}
