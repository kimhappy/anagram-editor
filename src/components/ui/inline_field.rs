use leptos::{ev::KeyboardEvent, html, prelude::*};

use crate::model::slot::Slot;

#[component]
pub fn SlotField(
    #[prop(into)] value: Signal<Slot>,
    #[prop(into)] on_commit: Callback<Slot>,
    label: &'static str,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let text = Signal::derive(move || value.get().to_string());
    view! {
        <InlineField
            text
            label
            maxlength=3
            class=format!("h-7 w-14 text-center font-medium tabular-nums {class}")
            on_commit=move |typed: String| {
                if let Ok(slot) = typed.parse() {
                    on_commit.run(slot);
                }
            }
        >
            {text}
        </InlineField>
    }
}

#[component]
pub fn InlineField(
    #[prop(into)] text: Signal<String>,
    #[prop(into)] on_commit: Callback<String>,
    label: &'static str,
    maxlength: usize,
    #[prop(into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let is_editing = RwSignal::new(false);
    let is_returning_focus = StoredValue::new(false);
    let input_ref = NodeRef::<html::Input>::new();
    let button_ref = NodeRef::<html::Button>::new();
    Effect::watch(
        move || text.track(),
        move |(), _, _| is_editing.set(false),
        false,
    );
    let finish = move |typed: Option<String>| {
        if !is_editing.get_untracked() {
            return;
        }
        is_editing.set(false);
        if let Some(typed) = typed {
            on_commit.run(typed);
        }
    };
    let on_keydown = move |event: KeyboardEvent| match event.key().as_str() {
        "Enter" => {
            is_returning_focus.set_value(true);
            finish(Some(event_target_value(&event)));
        }
        "Escape" => {
            event.stop_propagation();
            is_returning_focus.set_value(true);
            finish(None);
        }
        _ => {}
    };

    Effect::new(move |_| {
        if let Some(input) = input_ref.get()
            && is_editing.get()
        {
            input.focus().unwrap_or_default();
            input.select();
        }
    });
    Effect::new(move |_| {
        if let Some(button) = button_ref.get()
            && is_returning_focus.get_value()
        {
            is_returning_focus.set_value(false);
            button.focus().unwrap_or_default();
        }
    });

    let button_class = format!("hover:bg-accent min-w-0 cursor-text rounded-md {class}");
    let input_class = format!(
        "border-input focus-visible:border-ring focus-visible:ring-ring/50 min-w-0 rounded-md border bg-transparent outline-none focus-visible:ring-3 {class}"
    );
    view! {
        <Show
            when=move || is_editing.get()
            fallback=move || {
                view! {
                    <button
                        type="button"
                        node_ref=button_ref
                        class=button_class.clone()
                        aria-description=format!("{label}, click to type")
                        on:click=move |_| is_editing.set(true)
                    >
                        {children()}
                    </button>
                }
            }
        >
            <input
                node_ref=input_ref
                class=input_class.clone()
                aria-label=label
                maxlength=maxlength.to_string()
                prop:value=text.get_untracked()
                on:keydown=on_keydown
                on:blur=move |event| finish(Some(event_target_value(&event)))
            />
        </Show>
    }
}
