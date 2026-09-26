use leptos::{
    ev::{FocusEvent, KeyboardEvent},
    html,
    prelude::*,
};
use leptos_use::on_click_outside;

use super::focus::{self, Edge, Navigation};
use crate::components::icon::{Icon, IconKind};

const MENU_CLASS: &str = "bg-popover text-popover-foreground fixed z-50 w-max min-w-40 rounded-md border p-1 whitespace-nowrap";
const ITEM_CLASS: &str = "hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-sm outline-hidden select-none disabled:pointer-events-none disabled:opacity-50";
const ENABLED_ITEMS: &str = "[role=menuitem]:not([disabled])";
const MENU_MARGIN: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuAnchor {
    pub x: f64,
    pub y: f64,
}

impl MenuAnchor {
    #[must_use]
    pub fn from_mouse(event: &leptos::ev::MouseEvent) -> Self {
        Self {
            x: event.client_x(),
            y: event.client_y(),
        }
    }
}

pub struct MenuState<K: Send + Sync + 'static> {
    pub anchor: RwSignal<Option<MenuAnchor>>,
    pub target: RwSignal<Option<K>>,
}

impl<K: Send + Sync + 'static> Clone for MenuState<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: Send + Sync + 'static> Copy for MenuState<K> {}

impl<K: Send + Sync + 'static> MenuState<K> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            anchor: RwSignal::new(None),
            target: RwSignal::new(None),
        }
    }

    pub fn open(self, event: &leptos::ev::MouseEvent, target: K) {
        event.prevent_default();
        self.target.set(Some(target));
        self.anchor.set(Some(MenuAnchor::from_mouse(event)));
    }

    pub fn close(self) {
        self.anchor.set(None);
    }
}

impl<K: Send + Sync + 'static> Default for MenuState<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct MenuItem {
    pub label: &'static str,
    pub icon: IconKind,
    pub disabled: bool,
    pub action: Callback<()>,
}

#[component]
pub fn ContextMenu(
    anchor: RwSignal<Option<MenuAnchor>>,
    #[prop(into)] items: Signal<Vec<MenuItem>>,
) -> impl IntoView {
    let root = NodeRef::<html::Div>::new();
    let placed = RwSignal::new(None::<(f64, f64)>);
    let opener = StoredValue::new_local(None::<web_sys::HtmlElement>);
    let close = move || {
        opener.set_value(None);
        if anchor.with_untracked(Option::is_some) {
            anchor.set(None);
        }
    };
    let close_and_restore = move || {
        let previous = opener.get_value();
        close();
        if let Some(element) = previous {
            focus::focus(&element);
        }
    };
    let _stop_listening = on_click_outside(root, move |_| close());
    let wheel_listener = window_event_listener(leptos::ev::wheel, move |_| close());
    let resize_listener = window_event_listener(leptos::ev::resize, move |_| close());
    on_cleanup(move || {
        wheel_listener.remove();
        resize_listener.remove();
    });
    Effect::new(move |was_shown: Option<bool>| {
        let (Some(at), Some(menu)) = (anchor.get(), root.get()) else {
            placed.set(None);
            return false;
        };
        items.track();
        let rect = menu.get_bounding_client_rect();
        let (width, height) = viewport();
        placed.set(Some((
            fit(at.x, rect.width(), width),
            fit(at.y, rect.height(), height),
        )));
        if was_shown != Some(true) {
            opener.set_value(focus::focused_element());
            request_animation_frame(move || {
                if let Some(shown) = root.try_get_untracked().flatten() {
                    focus::focus_edge(&focus::enabled_items(&shown, ENABLED_ITEMS), Edge::First);
                }
            });
        }
        true
    });
    let on_keydown = move |event: KeyboardEvent| match event.key().as_str() {
        "Escape" => {
            event.stop_propagation();
            close_and_restore();
        }
        "Tab" => {
            event.prevent_default();
            close_and_restore();
        }
        key => {
            if let (Some(navigation), Some(menu)) = (Navigation::of_key(key), root.get_untracked())
            {
                event.prevent_default();
                navigation.apply(&focus::enabled_items(&menu, ENABLED_ITEMS));
            }
        }
    };
    let on_focusout = move |event: FocusEvent| {
        if root
            .get_untracked()
            .is_some_and(|menu| focus::leaves(&menu, &event))
        {
            close();
        }
    };
    let style = move || match (anchor.get(), placed.get()) {
        (Some(_), Some((x, y))) => format!("left: {x}px; top: {y}px"),
        (Some(at), None) => format!("left: {}px; top: {}px; visibility: hidden", at.x, at.y),
        (None, _) => String::new(),
    };

    view! {
        <Show when=move || anchor.with(Option::is_some)>
            <div
                node_ref=root
                role="menu"
                class=MENU_CLASS
                style=style
                on:keydown=on_keydown
                on:focusout=on_focusout
                on:contextmenu=move |event| event.prevent_default()
            >
                {move || {
                    items
                        .get()
                        .into_iter()
                        .map(|item| view! { <MenuButton item on_choose=Callback::new(move |()| close_and_restore()) /> })
                        .collect_view()
                }}
            </div>
        </Show>
    }
}

#[component]
fn MenuButton(item: MenuItem, on_choose: Callback<()>) -> impl IntoView {
    let action = item.action;
    view! {
        <button
            type="button"
            role="menuitem"
            class=ITEM_CLASS
            disabled=item.disabled
            on:click=move |_| {
                on_choose.run(());
                action.run(());
            }
        >
            <Icon kind=item.icon class="text-muted-foreground" />
            {item.label}
        </button>
    }
}

#[expect(clippy::float_arithmetic, reason = "viewport pixel positions")]
fn fit(position: f64, size: f64, limit: f64) -> f64 {
    position.min(limit - size - MENU_MARGIN).max(MENU_MARGIN)
}

fn viewport() -> (f64, f64) {
    let window = web_sys::window();
    let size =
        |read: fn(&web_sys::Window) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>| {
            window
                .as_ref()
                .and_then(|window| read(window).ok())
                .and_then(|value| value.as_f64())
                .unwrap_or(f64::INFINITY)
        };
    (
        size(web_sys::Window::inner_width),
        size(web_sys::Window::inner_height),
    )
}

#[cfg(test)]
mod tests {
    use super::fit;

    #[test]
    fn fit_keeps_the_menu_inside_the_viewport_margin() {
        assert!((fit(100.0, 50.0, 1000.0) - 100.0).abs() < f64::EPSILON);
        assert!((fit(980.0, 50.0, 1000.0) - 942.0).abs() < f64::EPSILON);
        assert!((fit(2.0, 50.0, 1000.0) - 8.0).abs() < f64::EPSILON);
        assert!((fit(100.0, 2000.0, 1000.0) - 8.0).abs() < f64::EPSILON);
        assert!((fit(100.0, 50.0, f64::INFINITY) - 100.0).abs() < f64::EPSILON);
    }
}
