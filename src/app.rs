use std::rc::Rc;

use gloo_events::EventListener;
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::{
    backend::Backend,
    components::{ConnectScreen, EditorScreen},
    devices::{self, hid},
    lifecycle::{AppLink, Reconnect, ReconnectCause},
};

#[component]
pub fn App() -> impl IntoView {
    let backend = RwSignal::new(None::<SendWrapper<Rc<dyn Backend>>>);
    let unplug_listener = StoredValue::new_local(None::<EventListener>);
    let app = AppLink::new();
    provide_context(app);

    let on_connect = move |connected: SendWrapper<Rc<dyn Backend>>| {
        let own = Rc::clone(&connected);
        let listener = hid::on_disconnect(move |gone| {
            if !own.is_device(&gone) {
                return;
            }
            let is_expected = app.reconnect.with_untracked(Option::is_some);
            if !is_expected {
                app.reconnect.set(Some(Reconnect {
                    cause: ReconnectCause::Unplugged,
                    deadline: None,
                }));
                devices::tab_lock::release();
            }
            backend.set(None);
        })
        .ok();
        unplug_listener.set_value(listener);
        backend.set(Some(connected));
    };

    move || {
        backend.get().map_or_else(
            || view! { <ConnectScreen on_connect awaiting=app.reconnect /> }.into_any(),
            |connected| view! { <EditorScreen backend=connected /> }.into_any(),
        )
    }
}
