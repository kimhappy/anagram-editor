use leptos::prelude::*;

use super::{
    super::{bound_marks, change_marks},
    LABEL_TEXT_CLASS,
};
use crate::{components::marquee::Marquee, model::preset::Cell, session::Session};

#[derive(Clone, Copy)]
pub struct ParamMark {
    pub is_changed: Signal<bool>,
    pub is_scene_specific: Signal<bool>,
    pub bound: Signal<String>,
}

impl ParamMark {
    fn text_class(self) -> Signal<String> {
        Signal::derive(move || change_marks(self.is_changed.get(), self.is_scene_specific.get()))
    }
}

#[component]
pub fn ParamLabel(name: String, mark: ParamMark) -> impl IntoView {
    view! {
        <span class="flex min-w-0 flex-1 items-center">
            <Marquee text=name class=LABEL_TEXT_CLASS text_class=mark.text_class() />
            <BoundMark bound=mark.bound />
        </span>
    }
}

#[component]
pub fn BoundMark(bound: Signal<String>) -> impl IntoView {
    view! {
        <span class="text-muted-foreground shrink-0 pl-1 text-[0.6875rem] leading-none font-medium empty:hidden">
            {bound}
        </span>
    }
}

pub fn bound_to(cell: Cell, symbol: String) -> Signal<String> {
    let session = Session::expect();
    Signal::derive(move || {
        session
            .draft
            .with(|preset| bound_marks(preset.bindings_of_param(cell, &symbol)))
    })
}

#[component]
pub fn SwitchState(checked: Signal<bool>, class: &'static str) -> impl IntoView {
    view! {
        <span class=class aria-hidden="true">
            {move || if checked.get() { "On" } else { "Off" }}
        </span>
    }
}
