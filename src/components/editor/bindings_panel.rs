use leptos::prelude::*;

use super::side_panel::Placeholder;
use crate::{
    components::{
        icon::IconKind,
        ui::{
            Button, ButtonSize, ButtonVariant, ChoiceSelect, CommitInput, Dialog, INPUT_CLASS,
            IconButton, NUMBER_INPUT_CLASS, Slider, parse_number,
        },
    },
    model::{
        binding::{BYPASS_SYMBOL, Binding, BindingTarget, default_binding_name},
        mode::Mode,
        preset::BlockId,
    },
    protocol::{actuator::Actuator, name::PRESET_NAME},
    session::{Modal, Session},
};

const GRID_CLASS: &str = "grid grid-cols-[6rem_minmax(0,2fr)_minmax(0,1.5fr)_5rem_5rem_minmax(0,1fr)_2rem_7rem] items-center gap-3";

#[derive(Clone, Debug, PartialEq)]
struct Target {
    id: BlockId,
    name: String,
    label: String,
    params: Vec<TargetParam>,
}

#[derive(Clone, Debug, PartialEq)]
struct TargetParam {
    symbol: String,
    label: String,
    min: f64,
    max: f64,
    suits_expression: bool,
}

impl TargetParam {
    fn bypass() -> Self {
        Self {
            symbol: BYPASS_SYMBOL.to_owned(),
            label: "On / Off".to_owned(),
            min: 0.0,
            max: 1.0,
            suits_expression: false,
        }
    }

    fn is_on_off(&self) -> bool {
        self.symbol == BYPASS_SYMBOL
    }

    fn default_name(&self, block_name: &str) -> String {
        default_binding_name(
            block_name,
            (!self.is_on_off()).then_some(self.label.as_str()),
        )
    }

    fn target(&self, block: BlockId) -> BindingTarget {
        BindingTarget {
            block,
            symbol: self.symbol.clone(),
            min: self.min,
            max: self.max,
            extra: serde_json::Map::new(),
        }
    }
}

fn default_param(actuator: Actuator, params: &[TargetParam]) -> Option<&TargetParam> {
    let adjustable = || params.iter().find(|param| !param.is_on_off());
    let on_off = || params.iter().find(|param| param.is_on_off());
    match actuator {
        Actuator::ExpPedal => params
            .iter()
            .find(|param| param.suits_expression)
            .or_else(adjustable)
            .or_else(on_off),
        Actuator::FootA | Actuator::FootB | Actuator::FootC => on_off().or_else(adjustable),
        _ => adjustable().or_else(on_off),
    }
}

fn ordered_for(actuator: Actuator, params: &[TargetParam]) -> Vec<TargetParam> {
    let mut ordered = params.to_vec();
    if actuator == Actuator::ExpPedal {
        ordered.sort_by_key(|param| !param.suits_expression);
    }
    ordered
}

#[component]
pub fn BindingsDialog() -> impl IntoView {
    let session = Session::expect();
    let targets = Memo::new(move |_| {
        session.draft.with(|preset| {
            preset
                .blocks()
                .map(|(cell, block)| Target {
                    id: block.id,
                    name: block.model.name.clone(),
                    label: format!(
                        "{} · {} (R{} S{})",
                        block.model.short(),
                        block.model.name,
                        cell.row + 1,
                        cell.column + 1
                    ),
                    params: std::iter::once(TargetParam::bypass())
                        .chain(
                            block
                                .model
                                .params
                                .iter()
                                .filter(|spec| spec.allowed_in_bindings())
                                .map(|spec| TargetParam {
                                    symbol: spec.symbol.clone(),
                                    label: spec.name.clone(),
                                    min: spec.min,
                                    max: spec.max,
                                    suits_expression: spec.suits_expression_pedal(),
                                }),
                        )
                        .collect(),
                })
                .collect::<Vec<_>>()
        })
    });

    Effect::new(move |_| {
        let is_bindings_open = session
            .ui
            .modal
            .with_untracked(|modal| *modal == Some(Modal::Bindings));
        if session.mode.get() != Mode::Stomp && is_bindings_open {
            session.ui.close();
        }
    });

    view! {
        <Dialog
            is_open=session.ui.is_open(Modal::Bindings)
            on_close=move |()| session.ui.close()
            title="Stomp settings"
            class="max-w-5xl"
        >
            <Show
                when=move || !session.is_read_only()
                fallback=|| view! { <Placeholder text="Bindings of a read-only preset cannot be edited." /> }
            >
                <div class="grid gap-3">
                    <div class=format!("text-muted-foreground {GRID_CLASS} text-xs")>
                        <span>"Actuator"</span>
                        <span>"Block"</span>
                        <span>"Parameter"</span>
                        <span>"Min"</span>
                        <span>"Max"</span>
                        <span>"Name"</span>
                        <span></span>
                        <span>"Try it"</span>
                    </div>
                    {Actuator::ALL
                        .into_iter()
                        .map(|actuator| view! { <BindingRow actuator targets /> })
                        .collect_view()}
                </div>
            </Show>
        </Dialog>
    }
}

#[component]
fn BindingRow(actuator: Actuator, targets: Memo<Vec<Target>>) -> impl IntoView {
    let session = Session::expect();
    let binding = Memo::new(move |_| {
        session
            .draft
            .with(|preset| preset.binding(actuator).cloned())
    });
    let target = Memo::new(move |_| binding.get()?.target().cloned());
    let is_cc_enabled = Signal::derive(move || {
        session
            .midi_settings
            .with(|settings| settings.bindings.is_enabled(actuator))
    });
    let cc = Signal::derive(move || {
        session
            .midi_settings
            .with(|settings| settings.bindings.cc(actuator))
    });
    let is_changed = Signal::derive(move || {
        session
            .edits
            .with(|edits| edits.is_binding_edited(actuator))
    });

    let block_choices = Signal::derive(move || {
        targets.with(|targets| {
            std::iter::once((None, "None".to_owned()))
                .chain(
                    targets
                        .iter()
                        .map(|candidate| (Some(candidate.id), candidate.label.clone())),
                )
                .collect::<Vec<_>>()
        })
    });
    let chosen_block = Signal::derive(move || Some(target.get().map(|target| target.block)));
    let bound_target = Signal::derive(move || {
        let target = target.get()?;
        targets.with(|targets| {
            targets
                .iter()
                .find(|candidate| candidate.id == target.block)
                .cloned()
        })
    });
    let params = Signal::derive(move || {
        bound_target.get().map_or_else(Vec::new, |candidate| {
            ordered_for(actuator, &candidate.params)
        })
    });
    let param_choices = Signal::derive(move || {
        params.with(|params| {
            params
                .iter()
                .map(|param| (param.symbol.clone(), param.label.clone()))
                .collect::<Vec<_>>()
        })
    });
    let chosen_param = Signal::derive(move || target.get().map(|target| target.symbol));

    let current_default = move || {
        let target = target.get_untracked()?;
        let candidate = bound_target.get_untracked()?;
        candidate
            .params
            .iter()
            .find(|param| param.symbol == target.symbol)
            .map(|param| param.default_name(&candidate.name))
    };
    let retarget = move |block: &Target, param: &TargetParam| {
        let old_default = current_default().unwrap_or_default();
        let new_default = param.default_name(&block.name);
        let updated = binding.get_untracked().map_or_else(
            || Binding::single(&new_default, param.target(block.id)),
            |mut current| {
                current.retarget(param.target(block.id), &old_default, &new_default);
                current
            },
        );
        session.set_binding(actuator, Some(updated));
    };
    let choose_block = move |id: Option<BlockId>| {
        let Some(chosen) = id.and_then(|id| {
            targets.with_untracked(|targets| {
                targets.iter().find(|candidate| candidate.id == id).cloned()
            })
        }) else {
            session.set_binding(actuator, None);
            return;
        };
        let param = default_param(actuator, &chosen.params)
            .cloned()
            .unwrap_or_else(TargetParam::bypass);
        retarget(&chosen, &param);
    };
    let choose_param = move |symbol: String| {
        let (Some(block), Some(param)) = (
            bound_target.get_untracked(),
            params.with_untracked(|params| {
                params.iter().find(|param| param.symbol == symbol).cloned()
            }),
        ) else {
            return;
        };
        retarget(&block, &param);
    };
    let set_bound = move |is_min: bool, text: &str| {
        let Some(value) = parse_number(text) else {
            return;
        };
        let Some(mut current) = binding.get_untracked() else {
            return;
        };
        if let Some(bound) = current.target_mut() {
            let value = params.with_untracked(|params| {
                params
                    .iter()
                    .find(|param| param.symbol == bound.symbol)
                    .map_or(value, |param| value.clamp(param.min, param.max))
            });
            if is_min {
                bound.min = value;
            } else {
                bound.max = value;
            }
        }
        session.set_binding(actuator, Some(current));
    };
    let set_name = move |name: String| {
        let Some(mut current) = binding.get_untracked() else {
            return;
        };
        let name = PRESET_NAME.sanitize(&name);
        if !name.is_empty() {
            current.name = name;
            session.set_binding(actuator, Some(current));
        }
    };

    let has_no_target = Signal::derive(move || target.get().is_none());
    let has_no_binding = Signal::derive(move || binding.get().is_none());

    view! {
        <div class=GRID_CLASS>
            <div class="grid leading-tight">
                <span class=move || if is_changed.get() { "text-sm italic" } else { "text-sm" }>{actuator.label()}</span>
                <span class="text-muted-foreground text-xs tabular-nums">
                    {move || if is_cc_enabled.get() { format!("CC {}", cc.get()) } else { "CC Enable off".to_owned() }}
                </span>
            </div>
            <ChoiceSelect
                choices=block_choices
                chosen=chosen_block
                label=format!("{} block", actuator.label())
                on_choose=choose_block
            />
            <ChoiceSelect
                choices=param_choices
                chosen=chosen_param
                label=format!("{} parameter", actuator.label())
                disabled=has_no_target
                on_choose=choose_param
            />
            <CommitInput
                shown=Signal::derive(move || shown_bound(target.get().map(|target| target.min)))
                class=NUMBER_INPUT_CLASS
                disabled=has_no_target
                attr:inputmode="decimal"
                attr:aria-label=format!("{} minimum", actuator.label())
                on_commit=move |text: String| set_bound(true, &text)
            />
            <CommitInput
                shown=Signal::derive(move || shown_bound(target.get().map(|target| target.max)))
                class=NUMBER_INPUT_CLASS
                disabled=has_no_target
                attr:inputmode="decimal"
                attr:aria-label=format!("{} maximum", actuator.label())
                on_commit=move |text: String| set_bound(false, &text)
            />
            <CommitInput
                shown=Signal::derive(move || binding.get().map_or_else(String::new, |binding| binding.name))
                class=INPUT_CLASS
                disabled=has_no_binding
                attr:aria-label=format!("{} name", actuator.label())
                attr:maxlength=PRESET_NAME.max_length.to_string()
                on_commit=set_name
            />
            <IconButton
                kind=IconKind::Unlink
                class="size-8"
                label=format!("Clear {} binding", actuator.label())
                disabled=has_no_binding
                on:click=move |_| session.set_binding(actuator, None)
            />
            <BindingTest actuator is_enabled=is_cc_enabled />
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Press {
    label: &'static str,
    value: u8,
}

const fn foot_presses(toggle_logic: bool) -> &'static [Press] {
    if toggle_logic {
        &[Press {
            label: "Press",
            value: 127,
        }]
    } else {
        &[
            Press {
                label: "Off",
                value: 0,
            },
            Press {
                label: "On",
                value: 127,
            },
        ]
    }
}

#[component]
fn BindingTest(actuator: Actuator, is_enabled: Signal<bool>) -> impl IntoView {
    let session = Session::expect();
    let is_foot = matches!(
        actuator,
        Actuator::FootA | Actuator::FootB | Actuator::FootC
    );
    let title = move || {
        if is_enabled.get() {
            ""
        } else {
            "Turn on CC Enable for this control in the device settings first"
        }
    };
    if is_foot {
        let toggle_logic =
            Signal::derive(move || session.midi_settings.with(|settings| settings.toggle_logic));
        return view! {
            <div class="flex gap-1" title=title>
                {move || {
                    foot_presses(toggle_logic.get())
                        .iter()
                        .map(|press| {
                            let Press { label, value } = *press;
                            view! {
                                <Button
                                    variant=ButtonVariant::Outline
                                    size=ButtonSize::Sm
                                    disabled=Signal::derive(move || !is_enabled.get())
                                    attr:aria-label=format!("{label} {}", actuator.label())
                                    on:click=move |_| {
                                        let _sent = session.test_binding(actuator, value);
                                    }
                                >
                                    {label}
                                </Button>
                            }
                        })
                        .collect_view()
                }}
            </div>
        }
        .into_any();
    }
    let level = RwSignal::new(if actuator == Actuator::ExpPedal {
        0.0
    } else {
        63.0
    });
    let last_sent = StoredValue::new(None::<u8>);
    let send = move |raw: f64| {
        if !is_enabled.get_untracked() {
            return;
        }
        level.set(raw);
        let value = level_to_cc(raw);
        if last_sent.get_value() != Some(value) && session.test_binding(actuator, value) {
            last_sent.set_value(Some(value));
        }
    };
    view! {
        <div class="flex items-center" title=title>
            <Slider
                bounds=(0.0, 127.0, 1.0)
                value=level
                label=format!("Try {}", actuator.label())
                disabled=Signal::derive(move || !is_enabled.get())
                on_input=send
            />
        </div>
    }
    .into_any()
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a slider position in 0..=127 becomes a CC value"
)]
const fn level_to_cc(level: f64) -> u8 {
    level.round().clamp(0.0, 127.0) as u8
}

fn shown_bound(bound: Option<f64>) -> String {
    bound.map_or_else(String::new, |bound| format!("{bound}"))
}

#[cfg(test)]
mod tests {
    use super::{TargetParam, default_param, foot_presses, ordered_for};
    use crate::protocol::actuator::Actuator;

    fn param(symbol: &str, suits_expression: bool) -> TargetParam {
        TargetParam {
            symbol: symbol.to_owned(),
            label: symbol.to_owned(),
            min: 0.0,
            max: 1.0,
            suits_expression,
        }
    }

    #[test]
    fn each_control_starts_on_the_parameter_that_suits_it() {
        let params = [
            TargetParam::bypass(),
            param("mix", false),
            param("position", true),
        ];
        let first = |actuator| default_param(actuator, &params).map(|param| param.symbol.clone());
        assert_eq!(first(Actuator::ExpPedal).as_deref(), Some("position"));
        assert_eq!(first(Actuator::FootA).as_deref(), Some(":bypass"));
        assert_eq!(first(Actuator::Knob1).as_deref(), Some("mix"));
        let only_bypass = [TargetParam::bypass()];
        assert_eq!(
            default_param(Actuator::Knob1, &only_bypass).map(|param| param.symbol.as_str()),
            Some(":bypass")
        );
    }

    #[test]
    fn the_expression_pedal_lists_its_parameters_first() {
        let params = [
            TargetParam::bypass(),
            param("mix", false),
            param("position", true),
        ];
        let ordered: Vec<String> = ordered_for(Actuator::ExpPedal, &params)
            .into_iter()
            .map(|param| param.symbol)
            .collect();
        assert_eq!(ordered.first().map(String::as_str), Some("position"));
        assert_eq!(
            ordered_for(Actuator::Knob1, &params)
                .first()
                .map(|param| param.symbol.as_str()),
            Some(":bypass")
        );
    }

    #[test]
    fn footswitch_tests_follow_the_toggle_logic() {
        assert_eq!(foot_presses(false).len(), 2);
        assert_eq!(foot_presses(true).len(), 1);
    }
}
