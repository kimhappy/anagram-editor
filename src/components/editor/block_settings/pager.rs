use leptos::prelude::*;

use crate::components::{icon::IconKind, ui::IconButton};

#[component]
pub fn Pager(page: RwSignal<usize>, page_count: usize) -> impl IntoView {
    let last = page_count.saturating_sub(1);
    view! {
        <div class="flex items-center gap-1">
            <IconButton
                kind=IconKind::ChevronLeft
                class="size-8"
                label="Previous parameters"
                disabled=Signal::derive(move || page.get() == 0)
                on:click=move |_| page.update(|page| *page = page.saturating_sub(1))
            />
            <span class="w-12 text-center text-sm tabular-nums">
                {move || format!("{} / {page_count}", page.get() + 1)}
            </span>
            <IconButton
                kind=IconKind::ChevronRight
                class="size-8"
                label="Next parameters"
                disabled=Signal::derive(move || page.get() >= last)
                on:click=move |_| page.update(|page| *page = (*page + 1).min(last))
            />
        </div>
    }
}
