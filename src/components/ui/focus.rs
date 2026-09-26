use wasm_bindgen::JsCast;
use web_sys::{Element, Event, FocusEvent, FocusOptions, HtmlElement};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    First,
    Last,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Navigation {
    Step(isize),
    Jump(Edge),
}

impl Navigation {
    #[must_use]
    pub fn of_key(key: &str) -> Option<Self> {
        match key {
            "ArrowDown" => Some(Self::Step(1)),
            "ArrowUp" => Some(Self::Step(-1)),
            "Home" => Some(Self::Jump(Edge::First)),
            "End" => Some(Self::Jump(Edge::Last)),
            _ => None,
        }
    }

    pub fn apply(self, items: &[HtmlElement]) {
        match self {
            Self::Step(step) => move_focus(items, step),
            Self::Jump(edge) => focus_edge(items, edge),
        }
    }
}

#[must_use]
pub fn focused_element() -> Option<HtmlElement> {
    web_sys::window()?
        .document()?
        .active_element()?
        .dyn_into::<HtmlElement>()
        .ok()
}

#[must_use]
pub fn enabled_items(root: &Element, selector: &str) -> Vec<HtmlElement> {
    let Ok(nodes) = root.query_selector_all(selector) else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|index| nodes.item(index)?.dyn_into::<HtmlElement>().ok())
        .collect()
}

pub fn focus(element: &HtmlElement) {
    element.focus().unwrap_or_default();
}

pub fn focus_without_scroll(element: &HtmlElement) {
    let options = FocusOptions::new();
    options.set_prevent_scroll(true);
    element.focus_with_options(&options).unwrap_or_default();
}

#[must_use]
pub fn is_within(event: &Event, selector: &str) -> bool {
    event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|element| element.closest(selector).ok().flatten())
        .is_some()
}

pub fn move_focus(items: &[HtmlElement], step: isize) {
    let active = focused_element();
    let current = items.iter().position(|item| active.as_ref() == Some(item));
    let count = items.len().cast_signed();
    let next = current.map_or(0, |index| (index.cast_signed() + step).rem_euclid(count));
    if let Some(item) = items.get(next.cast_unsigned()) {
        focus(item);
    }
}

pub fn focus_edge(items: &[HtmlElement], edge: Edge) {
    let item = match edge {
        Edge::First => items.first(),
        Edge::Last => items.last(),
    };
    if let Some(item) = item {
        focus(item);
    }
}

#[must_use]
pub fn leaves(root: &Element, event: &FocusEvent) -> bool {
    event
        .related_target()
        .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
        .is_some_and(|next| !root.contains(Some(&next)))
}

#[cfg(test)]
mod tests {
    use super::{Edge, Navigation};

    #[test]
    fn navigation_keys_map_to_steps_and_jumps() {
        assert_eq!(Navigation::of_key("ArrowDown"), Some(Navigation::Step(1)));
        assert_eq!(Navigation::of_key("ArrowUp"), Some(Navigation::Step(-1)));
        assert_eq!(
            Navigation::of_key("Home"),
            Some(Navigation::Jump(Edge::First))
        );
        assert_eq!(
            Navigation::of_key("End"),
            Some(Navigation::Jump(Edge::Last))
        );
        assert_eq!(Navigation::of_key("ArrowLeft"), None);
        assert_eq!(Navigation::of_key("arrowdown"), None);
    }
}
