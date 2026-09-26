use std::rc::Rc;

use leptos::prelude::*;

use super::{
    LABEL_CLASS, PARAM_CLASS,
    marks::{ParamLabel, ParamMark},
};
use crate::{
    components::{editor::uploads, ui::ChoiceSelect},
    model::{
        catalog::PropertySpec,
        files::{FileChoice, FileKind, describe_path},
        preset::{BlockId, Cell},
    },
    protocol::hid::files::UserFile,
    session::Session,
};

#[component]
pub fn FileControl(cell: Cell, index: usize, spec: PropertySpec, uri: String) -> impl IntoView {
    let session = Session::expect();
    let kind = FileKind::for_block(&uri);
    let block = session.block_id(cell);
    if let Some(kind) = kind {
        session.ensure_user_files(kind.user_dir());
    }
    let current = Signal::derive(move || {
        session
            .draft
            .with(|preset| preset.block(cell)?.properties.get(index).cloned())
            .unwrap_or_default()
    });
    let choices = Memo::new(move |_| {
        let listed = kind.map_or_else(Vec::new, |kind| {
            session.files.user_files.with(|files| kind.choices(files))
        });
        let path = current.get();
        let unlisted = (!path.is_empty()
            && !listed.iter().any(|choice| choice.device_path == path))
        .then(|| FileChoice {
            label: describe_path(&path),
            device_path: path,
        });
        unlisted.into_iter().chain(listed).collect::<Vec<_>>()
    });
    let options = Signal::derive(move || {
        choices.with(|choices| {
            choices
                .iter()
                .map(|choice| (choice.device_path.clone(), choice.label.clone()))
                .collect::<Vec<_>>()
        })
    });
    let chosen = Signal::derive(move || Some(current.get()));
    let mark = ParamMark {
        is_changed: Signal::derive(move || session.is_property_edited(cell, index)),
        is_scene_specific: Signal::stored(false),
        bound: Signal::stored(String::new()),
    };
    let is_disabled = Signal::derive(move || options.with(Vec::is_empty));

    view! {
        <div class=PARAM_CLASS>
            <div class=LABEL_CLASS>
                <ParamLabel name=spec.name.clone() mark />
                {kind.map(|kind| {
                    let assign: Rc<dyn Fn(UserFile)> = Rc::new(move |uploaded| {
                        if let Some(target) = block {
                            assign_uploaded_file(&session, target, index, &uploaded);
                        }
                    });
                    view! { <uploads::UploadButton dir=kind.user_dir() on_uploaded=assign /> }
                })}
            </div>
            <ChoiceSelect
                choices=options
                chosen
                label=spec.name
                disabled=is_disabled
                on_choose=move |path: String| session.set_property(cell, index, &path)
            />
        </div>
    }
}

pub fn assign_uploaded_file(session: &Session, target: BlockId, index: usize, uploaded: &UserFile) {
    let cell = session
        .draft
        .with_untracked(|preset| preset.cell_of(target));
    if let Some(cell) = cell {
        session.set_property(cell, index, &uploaded.device_path());
    }
}
