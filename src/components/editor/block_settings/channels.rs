use std::sync::Arc;

use leptos::prelude::*;

use crate::{
    components::ui::ChoiceSelect,
    model::{
        catalog::{BlockModel, Channels, Variants},
        preset::Cell,
    },
    session::Session,
};

#[component]
pub fn ChannelsControl(cell: Cell, model: Arc<BlockModel>, pair: Variants) -> impl IntoView {
    let session = Session::expect();
    let chosen = pair.channels_of(&model.uri);
    let options: Vec<(Channels, String)> = Channels::ALL
        .iter()
        .map(|option| (*option, option.label().to_owned()))
        .collect();
    view! {
        <label class="flex items-center gap-2 text-sm">
            <span class="select-none">"Mode"</span>
            <div class="w-28">
                <ChoiceSelect
                    choices=Signal::stored(options)
                    chosen=Signal::stored(chosen)
                    label="Mono or stereo"
                    on_choose=move |channels: Channels| {
                        session.change_model(cell, Arc::clone(pair.get(channels)));
                    }
                />
            </div>
        </label>
    }
}
