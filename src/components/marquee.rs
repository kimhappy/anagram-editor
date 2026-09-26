use leptos::{html, prelude::*};
use leptos_use::use_resize_observer;
use tw_merge::tw_merge;

const MILLIS_PER_CHAR: usize = 250;
const MIN_MILLIS: usize = 4000;

#[component]
pub fn Marquee(
    #[prop(into)] text: Signal<String>,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] text_class: Signal<String>,
    #[prop(optional)] scroll_on_hover: bool,
) -> impl IntoView {
    let frame = NodeRef::<html::Div>::new();
    let content = NodeRef::<html::Span>::new();
    let is_overflowing = RwSignal::new(false);

    let measure = move || {
        if let (Some(frame), Some(content)) = (frame.get_untracked(), content.get_untracked()) {
            is_overflowing.set(content.scroll_width() > frame.client_width());
        }
    };

    Effect::new(move |_| {
        text.track();
        measure();
    });
    use_resize_observer(frame, move |_, _| measure());
    use_resize_observer(content, move |_, _| measure());

    let scrolling_class = if scroll_on_hover {
        "group-hover:animate-marquee"
    } else {
        "animate-marquee hover:[animation-play-state:paused]"
    };
    let duration = move || {
        let millis = text
            .with(|text| text.chars().count() * MILLIS_PER_CHAR)
            .max(MIN_MILLIS);
        format!("animation-duration: {millis}ms")
    };

    view! {
        <div node_ref=frame title=move || is_overflowing.get().then(|| text.get()) class=tw_merge!("min-w-0 overflow-hidden whitespace-nowrap", class.as_str())>
            <div
                style=duration
                class=move || {
                    tw_merge!(
                        "inline-flex",
                        if is_overflowing.get() { scrolling_class } else { "" }
                    )
                }
            >
                <span node_ref=content class=move || format!("px-0.5 {}", text_class.get())>{text}</span>
                <Show when=move || is_overflowing.get()>
                    <span class="inline-block w-12" aria-hidden="true"></span>
                    <span class=move || format!("px-0.5 {}", text_class.get()) aria-hidden="true">{text}</span>
                    <span class="inline-block w-12" aria-hidden="true"></span>
                </Show>
            </div>
        </div>
    }
}
