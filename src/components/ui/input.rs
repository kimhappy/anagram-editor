use leptos::{ev::KeyboardEvent, prelude::*};
use web_sys::HtmlInputElement;

macro_rules! input_class {
    () => {
        "placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground border-input h-9 w-full min-w-0 rounded-md border bg-background px-3 py-1 text-base disabled:opacity-50 transition-[color,box-shadow] outline-none md:text-sm focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3"
    };
}

pub const INPUT_CLASS: &str = input_class!();
pub const NUMBER_INPUT_CLASS: &str = concat!(input_class!(), " tabular-nums");
pub const COMPACT_NUMBER_INPUT_CLASS: &str = concat!(input_class!(), " px-2 tabular-nums");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction {
    Commit,
    Revert,
    Pass,
}

impl KeyAction {
    fn of(key: &str, is_edited: bool) -> Self {
        match (key, is_edited) {
            ("Enter", true) => Self::Commit,
            ("Escape", true) => Self::Revert,
            _ => Self::Pass,
        }
    }
}

#[must_use]
pub fn parse_number(text: &str) -> Option<f64> {
    text.trim()
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}

#[component]
pub fn CommitInput(
    #[prop(into)] shown: Signal<String>,
    #[prop(into)] on_commit: Callback<String>,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let is_edited = StoredValue::new(false);
    let revert = move |input: &HtmlInputElement| {
        is_edited.set_value(false);
        input.set_value(&shown.get_untracked());
    };
    let commit = move |input: &HtmlInputElement| {
        if is_edited.get_value() {
            on_commit.run(input.value());
            revert(input);
        }
    };
    view! {
        <input
            type="text"
            class=class
            disabled=disabled
            prop:value=shown
            on:input=move |_| is_edited.set_value(true)
            on:change=move |event| commit(&event_target::<HtmlInputElement>(&event))
            on:keydown=move |event: KeyboardEvent| {
                let input = event_target::<HtmlInputElement>(&event);
                match KeyAction::of(&event.key(), is_edited.get_value()) {
                    KeyAction::Commit => commit(&input),
                    KeyAction::Revert => {
                        event.stop_propagation();
                        revert(&input);
                    }
                    KeyAction::Pass => {}
                }
            }
        />
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyAction, parse_number};

    #[test]
    fn keys_act_only_on_edited_text() {
        assert_eq!(KeyAction::of("Enter", true), KeyAction::Commit);
        assert_eq!(KeyAction::of("Escape", true), KeyAction::Revert);
        assert_eq!(KeyAction::of("Enter", false), KeyAction::Pass);
        assert_eq!(KeyAction::of("Escape", false), KeyAction::Pass);
        assert_eq!(KeyAction::of("a", true), KeyAction::Pass);
    }

    #[test]
    fn parse_number_trims_and_rejects_non_finite() {
        assert_eq!(parse_number("  2.5 "), Some(2.5));
        assert_eq!(parse_number("-3"), Some(-3.0));
        assert_eq!(parse_number("NaN"), None);
        assert_eq!(parse_number("inf"), None);
        assert_eq!(parse_number("-infinity"), None);
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("   "), None);
        assert_eq!(parse_number("1 2"), None);
    }
}
