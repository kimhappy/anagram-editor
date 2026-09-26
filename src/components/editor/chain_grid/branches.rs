use leptos::prelude::*;

use super::{
    LINE_CLASS, SVG_CLASS,
    geometry::{Junction, lower_row_line},
};
use crate::model::preset::Routing;

pub fn segment(style: String) -> impl IntoView {
    view! { <rect class=LINE_CLASS style=style /> }
}

pub fn junction_segments(junction: Junction, column: usize) -> impl IntoView {
    junction.lines(column).map(segment)
}

#[component]
pub fn Branches(routing: Memo<Routing>) -> impl IntoView {
    view! {
        <svg class=SVG_CLASS aria-hidden="true">
            {segment(lower_row_line())}
            {move || routing.get().split.map(|column| junction_segments(Junction::Split, column))}
            {move || routing.get().merge.map(|column| junction_segments(Junction::Merge, column))}
        </svg>
    }
}
