use leptos::prelude::*;

use crate::{
    components::{
        icon::{Icon, IconKind},
        ui::{
            Button, ButtonSize, ButtonVariant, COMPACT_NUMBER_INPUT_CLASS, CommitInput, Field,
            FieldLook, IconButton, Select, stored_labels,
        },
    },
    model::midi_out::{MidiOutMessage, MidiOutValue},
};

const KINDS: [&str; 2] = ["CC", "PC"];
const VALUE_MODES: [&str; 2] = ["Slot number", "Fixed value"];
const NEW_MESSAGE: MidiOutMessage = MidiOutMessage::ProgramChange {
    channel: 1,
    program: MidiOutValue::SlotNumber,
};

fn replaced(list: &[MidiOutMessage], index: usize, message: MidiOutMessage) -> Vec<MidiOutMessage> {
    list.iter()
        .enumerate()
        .map(|(at, kept)| if at == index { message } else { *kept })
        .collect()
}

fn removed(list: &[MidiOutMessage], index: usize) -> Vec<MidiOutMessage> {
    list.iter()
        .enumerate()
        .filter(|(at, _)| *at != index)
        .map(|(_, kept)| *kept)
        .collect()
}

fn appended(list: &[MidiOutMessage]) -> Vec<MidiOutMessage> {
    list.iter()
        .copied()
        .chain(std::iter::once(NEW_MESSAGE))
        .collect()
}

fn number_field(text: &str, max: u8) -> Option<u8> {
    text.trim().parse::<u8>().ok().map(|number| number.min(max))
}

#[component]
pub fn MidiOutEditor(
    #[prop(into)] messages: Signal<Vec<MidiOutMessage>>,
    #[prop(into)] on_change: Callback<Vec<MidiOutMessage>>,
) -> impl IntoView {
    let replace = move |index: usize, message: MidiOutMessage| {
        on_change.run(messages.with_untracked(|list| replaced(list, index, message)));
    };
    let remove = move |index: usize| {
        on_change.run(messages.with_untracked(|list| removed(list, index)));
    };
    let add = move |_| on_change.run(messages.with_untracked(|list| appended(list)));

    let indexes = move || (0..messages.with(Vec::len)).collect::<Vec<usize>>();
    let row = move |index: usize| {
        let message = Signal::derive(move || {
            messages
                .with(|list| list.get(index).copied())
                .unwrap_or(NEW_MESSAGE)
        });
        view! {
            <MessageRow
                message
                on_change=move |changed| replace(index, changed)
                on_remove=move |()| remove(index)
            />
        }
    };

    view! {
        <div class="grid grid-cols-1 gap-2">
            <For each=indexes key=|index| *index children=row />
            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm class="w-fit" on:click=add>
                <Icon kind=IconKind::Plus />
                "Add message"
            </Button>
        </div>
    }
}

#[component]
fn MessageRow(
    message: Signal<MidiOutMessage>,
    #[prop(into)] on_change: Callback<MidiOutMessage>,
    #[prop(into)] on_remove: Callback<()>,
) -> impl IntoView {
    let kind_index = Signal::derive(move || match message.get() {
        MidiOutMessage::ControlChange { .. } => 0,
        MidiOutMessage::ProgramChange { .. } => 1,
    });
    let value = Signal::derive(move || match message.get() {
        MidiOutMessage::ControlChange { value, .. } => value,
        MidiOutMessage::ProgramChange { program, .. } => program,
    });
    let controller = Signal::derive(move || match message.get() {
        MidiOutMessage::ControlChange { controller, .. } => Some(controller),
        MidiOutMessage::ProgramChange { .. } => None,
    });
    let fixed = Signal::derive(move || match value.get() {
        MidiOutValue::SlotNumber => None,
        MidiOutValue::Fixed(fixed) => Some(fixed),
    });
    let channel = Signal::derive(move || message.get().channel());
    let is_control_change = move || kind_index.get() == 0;

    let rebuild =
        move |kind: usize, next_channel: u8, next_controller: u8, next_value: MidiOutValue| {
            if kind == 0 {
                MidiOutMessage::ControlChange {
                    channel: next_channel,
                    controller: next_controller,
                    value: next_value,
                }
            } else {
                MidiOutMessage::ProgramChange {
                    channel: next_channel,
                    program: next_value,
                }
            }
        };
    let current = move || {
        (
            kind_index.get_untracked(),
            channel.get_untracked(),
            controller.get_untracked().unwrap_or(0),
            value.get_untracked(),
        )
    };
    let shown = |number: Option<u8>| number.map_or_else(String::new, |number| number.to_string());

    let value_label = move || {
        if is_control_change() {
            "Value"
        } else {
            "Program"
        }
    };
    let fixed_title = move || {
        if is_control_change() {
            "CC value 0–127"
        } else {
            "Program 0–127"
        }
    };

    view! {
        <div class="grid grid-cols-1 gap-2 rounded-md border border-zinc-300 bg-zinc-100 p-2">
            <div class="grid grid-cols-[5.5rem_3.5rem_3.5rem_minmax(0,1fr)] items-end gap-2">
                <Field label="Type" look=FieldLook::Caption>
                    <Select
                        options=stored_labels(KINDS)
                        value=kind_index
                        label="Message type"
                        on_change=move |kind| {
                            let (_, now_channel, now_controller, now_value) = current();
                            on_change.run(rebuild(kind, now_channel, now_controller, now_value));
                        }
                    />
                </Field>
                <Field label="Ch" look=FieldLook::Caption>
                    <CommitInput
                        shown=Signal::derive(move || channel.get().to_string())
                        class=COMPACT_NUMBER_INPUT_CLASS
                        attr:inputmode="numeric"
                        attr:aria-label="Channel"
                        attr:title="Channel 1–16"
                        on_commit=move |text: String| {
                            if let Some(typed) = number_field(&text, 16).filter(|typed| *typed >= 1) {
                                let (now_kind, _, now_controller, now_value) = current();
                                on_change.run(rebuild(now_kind, typed, now_controller, now_value));
                            }
                        }
                    />
                </Field>
                <Field label="CC no." look=FieldLook::Caption>
                    <CommitInput
                        shown=Signal::derive(move || shown(controller.get()))
                        class=COMPACT_NUMBER_INPUT_CLASS
                        disabled=Signal::derive(move || !is_control_change())
                        attr:inputmode="numeric"
                        attr:aria-label="Controller number"
                        attr:title="CC number 0–127"
                        on_commit=move |text: String| {
                            if let Some(typed) = number_field(&text, 127) {
                                let (now_kind, now_channel, _, now_value) = current();
                                on_change.run(rebuild(now_kind, now_channel, typed, now_value));
                            }
                        }
                    />
                </Field>
                <IconButton
                    kind=IconKind::Trash
                    class="size-9 justify-self-end"
                    label="Remove message"
                    on:click=move |_| on_remove.run(())
                />
            </div>
            <Field label=Signal::derive(move || value_label().to_owned()) look=FieldLook::Caption>
                <div class="grid grid-cols-[minmax(0,1fr)_4rem] gap-2">
                    <Select
                        options=stored_labels(VALUE_MODES)
                        value=Signal::derive(move || usize::from(fixed.get().is_some()))
                        label="Value mode"
                        on_change=move |mode| {
                            let next = if mode == 0 {
                                MidiOutValue::SlotNumber
                            } else {
                                MidiOutValue::Fixed(fixed.get_untracked().unwrap_or(1))
                            };
                            let (now_kind, now_channel, now_controller, _) = current();
                            on_change.run(rebuild(now_kind, now_channel, now_controller, next));
                        }
                    />
                    <CommitInput
                        shown=Signal::derive(move || shown(fixed.get()))
                        class=COMPACT_NUMBER_INPUT_CLASS
                        disabled=Signal::derive(move || fixed.get().is_none())
                        attr:inputmode="numeric"
                        attr:aria-label="Fixed value"
                        attr:title=fixed_title
                        on_commit=move |text: String| {
                            if let Some(number) = number_field(&text, 127) {
                                let (now_kind, now_channel, now_controller, _) = current();
                                on_change.run(rebuild(now_kind, now_channel, now_controller, MidiOutValue::Fixed(number)));
                            }
                        }
                    />
                </div>
            </Field>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{NEW_MESSAGE, appended, number_field, removed, replaced};
    use crate::model::midi_out::{MidiOutMessage, MidiOutValue};

    const CC: MidiOutMessage = MidiOutMessage::ControlChange {
        channel: 2,
        controller: 7,
        value: MidiOutValue::Fixed(100),
    };

    #[test]
    fn list_helpers_touch_only_the_given_index() {
        let list = [NEW_MESSAGE, CC];
        assert_eq!(replaced(&list, 0, CC), vec![CC, CC]);
        assert_eq!(replaced(&list, 5, CC), list.to_vec());
        assert_eq!(removed(&list, 0), vec![CC]);
        assert_eq!(removed(&list, 5), list.to_vec());
        assert_eq!(appended(&[CC]), vec![CC, NEW_MESSAGE]);
        assert_eq!(appended(&[]), vec![NEW_MESSAGE]);
    }

    #[test]
    fn number_field_parses_trimmed_bytes_and_caps_them() {
        assert_eq!(number_field(" 12 ", 127), Some(12));
        assert_eq!(number_field("200", 127), Some(127));
        assert_eq!(number_field("300", 127), None);
        assert_eq!(number_field("-1", 127), None);
        assert_eq!(number_field("", 16), None);
        assert_eq!(number_field("1.5", 16), None);
    }
}
