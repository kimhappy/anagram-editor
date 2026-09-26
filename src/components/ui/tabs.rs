use leptos::prelude::*;
use tw_merge::tw_merge;

const TAB_LIST_CLASS: &str =
    "bg-muted text-muted-foreground inline-flex h-10 w-fit items-center justify-center rounded-lg";

const TRIGGER_CLASS: &str = "inline-flex h-full flex-1 items-center justify-center gap-1.5 rounded-md border-[3px] border-transparent bg-clip-padding px-4 py-1 text-sm font-normal whitespace-nowrap transition-[color,box-shadow] outline-none focus-visible:ring-ring/50 focus-visible:ring-3 hover:text-foreground";

const ACTIVE_CLASS: &str = "bg-background text-foreground";

pub fn static_tabs<T: Copy + Send + Sync + 'static>(
    options: &'static [(T, &'static str)],
) -> Signal<Vec<(T, String)>> {
    Signal::stored(
        options
            .iter()
            .map(|&(option, label)| (option, label.to_owned()))
            .collect(),
    )
}

#[component]
pub fn Tabs<T>(
    #[prop(into)] options: Signal<Vec<(T, String)>>,
    #[prop(into)] value: Signal<T>,
    #[prop(into)] on_change: Callback<T>,
    #[prop(into, optional)] class: String,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    view! {
        <div role="tablist" class=tw_merge!(TAB_LIST_CLASS, class.as_str())>
            {move || {
                options
                    .get()
                    .into_iter()
                    .map(|(option, label)| {
                        let chosen = option.clone();
                        let is_active = Signal::derive(move || value.get() == option);
                        view! {
                            <button
                                type="button"
                                role="tab"
                                aria-selected=move || is_active.get().to_string()
                                class=move || {
                                    tw_merge!(TRIGGER_CLASS, if is_active.get() { ACTIVE_CLASS } else { "" })
                                }
                                on:click=move |_| on_change.run(chosen.clone())
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
