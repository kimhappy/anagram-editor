use floating_ui_leptos::{
    ApplyState, Flip, FlipOptions, MiddlewareVec, Offset, OffsetOptions, Placement, Shift,
    ShiftOptions, Size, SizeOptions, Strategy, UseFloatingOptions, use_floating,
};
use leptos::{
    ev::{FocusEvent, KeyboardEvent},
    prelude::*,
};
use leptos_node_ref::AnyNodeRef;
use leptos_use::on_click_outside;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, Window};

use super::focus::{self, Navigation};
use crate::components::{
    icon::{Icon, IconKind},
    marquee::Marquee,
};

const OPTIONS: &str = "[role=option]";

const TRIGGER_CLASS: &str = "disabled:opacity-50 disabled:pointer-events-none border-input focus-visible:border-ring focus-visible:ring-ring/50 flex h-9 w-full items-center justify-between gap-2 rounded-md border bg-background px-3 py-2 text-sm whitespace-nowrap transition-[color,box-shadow] outline-none focus-visible:ring-3";

const CONTENT_CLASS: &str = "bg-popover text-popover-foreground z-50 min-w-32 overflow-x-hidden overflow-y-auto rounded-md border p-1";

const ITEM_CLASS: &str = "hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent group relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 pr-8 pl-2 text-left text-sm outline-hidden select-none";

const OFFSET: f64 = 4.0;
const LIST_MAX_HEIGHT: f64 = 240.0;

fn fit_to_trigger(state: ApplyState<Element, Window>) {
    if let Some(list) = state.state.elements.floating.dyn_ref::<HtmlElement>() {
        let style = list.style();
        let width = state.state.rects.reference.width;
        let height = state.available_height.min(LIST_MAX_HEIGHT);
        style
            .set_property("width", &format!("{width}px"))
            .unwrap_or_default();
        style
            .set_property("max-height", &format!("{height}px"))
            .unwrap_or_default();
    }
}

fn middleware() -> MiddlewareVec {
    vec![
        Box::new(Offset::new(OffsetOptions::Value(OFFSET))),
        Box::new(Flip::new(FlipOptions::default())),
        Box::new(Shift::new(ShiftOptions::default())),
        Box::new(Size::new(SizeOptions::new().apply(&fit_to_trigger))),
    ]
}

#[derive(Clone, Copy)]
struct Listbox {
    is_open: RwSignal<bool>,
    trigger: AnyNodeRef,
    list: AnyNodeRef,
    value: Signal<Option<usize>>,
}

impl Listbox {
    fn options(self) -> Vec<HtmlElement> {
        self.list
            .try_get_untracked()
            .flatten()
            .map_or_default(|list| focus::enabled_items(&list, OPTIONS))
    }

    fn focus_selected(self) {
        let items = self.options();
        let selected = self
            .value
            .try_get_untracked()
            .flatten()
            .and_then(|index| items.get(index));
        if let Some(item) = selected.or_else(|| items.first()) {
            focus::focus(item);
        }
    }

    fn close_to_trigger(self) {
        self.is_open.set(false);
        request_animation_frame(move || {
            if let Some(button) = self
                .trigger
                .try_get_untracked()
                .flatten()
                .filter(|button| button.is_connected())
                .and_then(|button| button.dyn_into::<HtmlElement>().ok())
            {
                focus::focus(&button);
            }
        });
    }

    fn on_keydown(self, event: &KeyboardEvent) {
        let key = event.key();
        let is_listing = self.is_open.get_untracked();
        if key == "Escape" && is_listing {
            event.stop_propagation();
            self.close_to_trigger();
            return;
        }
        match (is_listing, Navigation::of_key(&key)) {
            (true, Some(navigation)) => {
                event.prevent_default();
                navigation.apply(&self.options());
            }
            (false, Some(Navigation::Step(_))) => {
                event.prevent_default();
                self.is_open.set(true);
            }
            _ => {}
        }
    }
}

#[must_use]
pub fn stored_labels<T: Into<String>>(labels: impl IntoIterator<Item = T>) -> Signal<Vec<String>> {
    Signal::stored(labels.into_iter().map(Into::into).collect())
}

fn accessible_name(label: &str, selected: Option<&str>) -> Option<String> {
    (!label.is_empty()).then(|| {
        selected.map_or_else(
            || label.to_owned(),
            |selected| format!("{label}: {selected}"),
        )
    })
}

fn is_change(current: Option<usize>, picked: usize) -> bool {
    current != Some(picked)
}

#[component]
pub fn Select(
    #[prop(into)] options: Signal<Vec<String>>,
    #[prop(into)] value: Signal<Option<usize>>,
    #[prop(into)] on_change: Callback<usize>,
    #[prop(into, optional)] label: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let root = NodeRef::<leptos::html::Div>::new();
    let listbox = Listbox {
        is_open,
        trigger: AnyNodeRef::new(),
        list: AnyNodeRef::new(),
        value,
    };
    let position = use_floating(
        listbox.trigger,
        listbox.list,
        UseFloatingOptions::default()
            .open(is_open)
            .placement(Placement::BottomStart)
            .strategy(Strategy::Fixed)
            .middleware(SendWrapper::new(middleware()))
            .while_elements_mounted_auto_update(),
    );
    let _stop_listening = on_click_outside(root, move |_| is_open.set(false));
    Effect::new(move |was_shown: Option<bool>| {
        let is_shown = is_open.get() && position.is_positioned.get();
        if is_shown && was_shown != Some(true) {
            request_animation_frame(move || listbox.focus_selected());
        }
        is_shown
    });
    let choose = Callback::new(move |index| {
        if is_change(value.get_untracked(), index) {
            on_change.run(index);
        }
        listbox.close_to_trigger();
    });
    let on_focusout = move |event: FocusEvent| {
        if root
            .get_untracked()
            .is_some_and(|root| focus::leaves(&root, &event))
        {
            is_open.set(false);
        }
    };
    let selected_label = Signal::derive(move || {
        let index = value.get()?;
        options.with(|options| options.get(index).cloned())
    });
    let trigger_text = Signal::derive(move || selected_label.get().unwrap_or_default());
    let list_style = move || {
        let visibility = if position.is_positioned.get() {
            ""
        } else {
            " visibility: hidden;"
        };
        format!("{}{visibility}", position.floating_styles.get())
    };

    view! {
        <div
            node_ref=root
            class="relative"
            on:keydown=move |event| listbox.on_keydown(&event)
            on:focusout=on_focusout
        >
            <button
                type="button"
                node_ref=listbox.trigger
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get().to_string()
                aria-label=move || selected_label.with(|selected| accessible_name(&label, selected.as_deref()))
                class=TRIGGER_CLASS
                disabled=move || disabled.get()
                on:click=move |_| is_open.update(|is_open| *is_open = !*is_open)
            >
                <Marquee text=trigger_text class="flex-1 text-left" />
                <Icon kind=IconKind::ChevronDown class="shrink-0 opacity-50" />
            </button>
            <Show when=move || is_open.get()>
                <div
                    node_ref=listbox.list
                    role="listbox"
                    class=CONTENT_CLASS
                    style=list_style
                >
                    {options
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(index, text)| view! { <SelectOption text index value choose /> })
                        .collect_view()}
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SelectOption(
    text: String,
    index: usize,
    value: Signal<Option<usize>>,
    choose: Callback<usize>,
) -> impl IntoView {
    let is_selected = move || value.get() == Some(index);
    view! {
        <button
            type="button"
            role="option"
            aria-selected=move || is_selected().to_string()
            class=ITEM_CLASS
            on:click=move |_| choose.run(index)
        >
            <Marquee text=Signal::stored(text) class="flex-1" scroll_on_hover=true />
            <Show when=is_selected>
                <span class="absolute right-2 flex size-3.5 items-center justify-center">
                    <Icon kind=IconKind::Check />
                </span>
            </Show>
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::{accessible_name, is_change};

    #[test]
    fn accessible_name_carries_the_selected_label_when_there_is_one() {
        assert_eq!(
            accessible_name("Quick pot", Some("Gain")),
            Some("Quick pot: Gain".to_owned())
        );
        assert_eq!(
            accessible_name("Quick pot", None),
            Some("Quick pot".to_owned())
        );
        assert_eq!(accessible_name("", Some("Gain")), None);
    }

    #[test]
    fn picking_counts_as_a_change_unless_it_is_the_current_option() {
        assert!(!is_change(Some(2), 2));
        assert!(is_change(Some(2), 0));
        assert!(is_change(None, 0));
    }
}

#[component]
pub fn ChoiceSelect<T>(
    #[prop(into)] choices: Signal<Vec<(T, String)>>,
    #[prop(into)] chosen: Signal<Option<T>>,
    #[prop(into)] on_choose: Callback<T>,
    #[prop(into, optional)] label: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let labels = Signal::derive(move || {
        choices.with(|choices| {
            choices
                .iter()
                .map(|(_, shown)| shown.clone())
                .collect::<Vec<_>>()
        })
    });
    let position = Signal::derive(move || {
        let chosen = chosen.get()?;
        choices.with(|choices| choices.iter().position(|(value, _)| *value == chosen))
    });
    let choose = move |index: usize| {
        let picked =
            choices.with_untracked(|choices| choices.get(index).map(|(value, _)| value.clone()));
        if let Some(value) = picked {
            on_choose.run(value);
        }
    };
    view! { <Select options=labels value=position label disabled on_change=choose /> }
}
