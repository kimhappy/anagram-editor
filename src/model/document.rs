use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use serde_json::{Map, Value};

use super::{
    binding::{BYPASS_SYMBOL, Binding, BindingTarget},
    catalog::{BlockModel, Catalog, ParamKind, ParamSpec},
    midi_out::MidiOut,
    preset::{Block, BlockOverride, Cell, FieldExtras, Preset, overlay},
    slot::Slot,
};
use crate::protocol::{
    actuator::Actuator,
    hid::{
        preset::{
            BindingDoc, BindingTargetDoc, BlockDoc, ChainDoc, ParamDoc, PresetBody, PresetDocument,
            PropertyDoc, SceneDeltaDoc, SymbolValue,
        },
        uuid::PresetUuid,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportWarning {
    UnknownPlugin {
        cell: Cell,
        uri: String,
    },
    UnknownSymbol {
        cell: Cell,
        uri: String,
        symbol: String,
    },
    UnknownProperty {
        cell: Cell,
        uri: String,
    },
    DanglingBinding {
        key: String,
        row: u32,
        block: u32,
    },
    PropertyBinding {
        key: String,
    },
    EmptyBinding {
        key: String,
    },
    NotInScenes {
        cell: Cell,
        uri: String,
        symbol: String,
    },
    BadKey {
        path: String,
    },
    DuplicateKey {
        path: String,
    },
    UnreadableMidiOut {
        count: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Imported {
    pub preset: Preset,
    pub landing_scene: Slot,
    pub warnings: Vec<ImportWarning>,
}

#[must_use]
pub fn import(doc: &PresetDocument, catalog: &Catalog) -> Imported {
    let body = &doc.preset;
    let mut warnings = Vec::new();
    let mut preset = Preset::empty(body.name.clone().unwrap_or_else(|| "Untitled".to_owned()));
    preset.set_uuid(body.uuid.as_deref().and_then(PresetUuid::parse));
    preset.set_extra(body.extra.clone());

    import_chains(body, catalog, &mut preset, &mut warnings);
    preset.set_chain_extras(
        body.chains
            .iter()
            .filter(|(_, chain)| !chain.extra.is_empty())
            .map(|(row, chain)| (row.clone(), chain.extra.clone()))
            .collect(),
    );
    import_bindings(body, &mut preset, &mut warnings);

    let scene_names = unique_keys(
        body.scene_names
            .iter()
            .map(|(key, name)| (format!("sceneNames.{key}"), scene_from_1_based(key), name)),
    );
    for result in scene_names {
        match result {
            Ok((scene, name)) => preset.set_scene_name(scene, Some(name)),
            Err(warning) => warnings.push(warning),
        }
    }
    if let Some(metadata) = &body.metadata {
        let (midi_out, dropped) = MidiOut::from_doc(metadata);
        *preset.midi_out_mut() = midi_out;
        if dropped > 0 {
            warnings.push(ImportWarning::UnreadableMidiOut { count: dropped });
        }
    }

    let landing_scene = body
        .scene
        .and_then(|index| Slot::from_index(usize::try_from(index).unwrap_or(usize::MAX)))
        .unwrap_or_default();
    Imported {
        preset,
        landing_scene,
        warnings,
    }
}

fn import_chains(
    body: &PresetBody,
    catalog: &Catalog,
    preset: &mut Preset,
    warnings: &mut Vec<ImportWarning>,
) {
    let blocks = unique_keys(body.chains.iter().flat_map(|(row_key, chain)| {
        chain.blocks.iter().map(move |(block_key, block_doc)| {
            let path = format!("chains.{row_key}.blocks.{block_key}");
            let cell = row_key
                .parse::<u32>()
                .ok()
                .zip(block_key.parse::<u32>().ok())
                .and_then(|(row, block)| Cell::from_document(row, block));
            (path.clone(), cell, (path, block_doc))
        })
    }));
    for result in blocks {
        match result {
            Ok((cell, (path, block_doc))) => {
                import_placed_block(cell, &path, block_doc, catalog, preset, warnings);
            }
            Err(warning) => warnings.push(warning),
        }
    }
}

fn import_placed_block(
    cell: Cell,
    path: &str,
    block_doc: &BlockDoc,
    catalog: &Catalog,
    preset: &mut Preset,
    warnings: &mut Vec<ImportWarning>,
) {
    let block = import_block(cell, path, block_doc, catalog, warnings);
    let id = block.id;
    let model = Arc::clone(&block.model);
    preset.put(cell, block);
    let scenes = unique_keys(block_doc.scenes.iter().map(|(scene_key, delta)| {
        (
            format!("{path}.scenes.{scene_key}"),
            scene_from_1_based(scene_key),
            delta,
        )
    }));
    for result in scenes {
        match result {
            Ok((scene, delta)) => {
                let delta_path = format!("{path}.scenes.{}", scene.number());
                let override_ = import_override(cell, &delta_path, &model, delta, warnings);
                preset.set_scene_override(scene, id, override_);
            }
            Err(warning) => warnings.push(warning),
        }
    }
}

fn import_override(
    cell: Cell,
    path: &str,
    model: &BlockModel,
    delta: &SceneDeltaDoc,
    warnings: &mut Vec<ImportWarning>,
) -> BlockOverride {
    let entries = delta
        .parameters
        .iter()
        .enumerate()
        .map(|(position, entry)| {
            (
                format!("{path}.parameters.{position}"),
                &entry.symbol,
                entry.value,
            )
        });
    let (values, not_in_scenes): (Vec<_>, _) = partition_results(
        import_values(cell, model, entries, warnings)
            .into_iter()
            .map(|(index, value)| match model.params.get(index) {
                Some(spec) if !spec.allowed_in_scenes() => Err(ImportWarning::NotInScenes {
                    cell,
                    uri: model.uri.clone(),
                    symbol: spec.symbol.clone(),
                }),
                Some(_) | None => Ok((index, value)),
            }),
    );
    warnings.extend(not_in_scenes);
    BlockOverride {
        enabled: delta.enabled,
        values: values.into_iter().collect(),
        properties: delta.properties.clone(),
        extra: delta.extra.clone(),
        value_extras: extras_by(
            delta
                .parameters
                .iter()
                .map(|entry| (&entry.symbol, &entry.extra)),
        ),
    }
}

fn import_values<'doc>(
    cell: Cell,
    model: &BlockModel,
    entries: impl IntoIterator<Item = (String, &'doc String, f64)>,
    warnings: &mut Vec<ImportWarning>,
) -> Vec<(usize, f64)> {
    let (resolved, unknown): (Vec<_>, _) =
        partition_results(entries.into_iter().map(|(path, symbol, value)| {
            model
                .clamped(symbol, value)
                .map(|(index, clamped)| (path, Some(index), clamped))
                .ok_or_else(|| ImportWarning::UnknownSymbol {
                    cell,
                    uri: model.uri.clone(),
                    symbol: symbol.clone(),
                })
        }));
    let mut last_wins: Vec<_> = unique_keys(resolved.into_iter().rev()).collect();
    last_wins.reverse();
    let (values, duplicates): (Vec<_>, _) = partition_results(last_wins);
    warnings.extend(unknown.into_iter().chain(duplicates));
    values
}

fn device_order<'doc>(
    path: &str,
    parameters: &'doc BTreeMap<String, ParamDoc>,
) -> (Vec<(String, &'doc ParamDoc)>, Vec<ImportWarning>) {
    let read: Vec<(String, &ParamDoc)> = (1..=MAX_PARAMS_PER_BLOCK)
        .map_while(|number| {
            let key = number.to_string();
            parameters.get(&key).map(|param| (key, param))
        })
        .collect();
    let ignored = parameters
        .keys()
        .filter(|key| !read.iter().any(|(read_key, _)| read_key == *key))
        .map(|key| ImportWarning::BadKey {
            path: format!("{path}.parameters.{key}"),
        })
        .collect();
    (read, ignored)
}

const MAX_PARAMS_PER_BLOCK: usize = 60;

fn import_block(
    cell: Cell,
    path: &str,
    doc: &BlockDoc,
    catalog: &Catalog,
    warnings: &mut Vec<ImportWarning>,
) -> Block {
    let model = catalog.resolve(&doc.uri, Some(doc));
    if model.is_unknown() {
        warnings.push(ImportWarning::UnknownPlugin {
            cell,
            uri: doc.uri.clone(),
        });
    }
    let values = if model.is_unknown() {
        Vec::new()
    } else {
        let (read, ignored) = device_order(path, &doc.parameters);
        warnings.extend(ignored);
        let entries = read.into_iter().map(|(key, param)| {
            (
                format!("{path}.parameters.{key}"),
                &param.symbol,
                param.value,
            )
        });
        import_values(cell, &model, entries, warnings)
    };
    let (properties, unknown_properties): (Vec<(usize, String)>, _) =
        partition_results(doc.properties.values().map(|property| {
            model
                .property_index(&property.uri)
                .map(|index| (index, property.value.clone()))
                .ok_or_else(|| ImportWarning::UnknownProperty {
                    cell,
                    uri: property.uri.clone(),
                })
        }));
    warnings.extend(unknown_properties);
    let fresh = Block::new(model);
    Block {
        enabled: doc.enabled.unwrap_or(true),
        quickpot: doc.quickpot.clone().filter(|symbol| !symbol.is_empty()),
        extra: doc.extra.clone(),
        field_extras: FieldExtras {
            params: extras_by(
                doc.parameters
                    .values()
                    .map(|param| (&param.symbol, &param.extra)),
            ),
            properties: extras_by(
                doc.properties
                    .values()
                    .map(|property| (&property.uri, &property.extra)),
            ),
        },
        values: overlay(fresh.values, values),
        properties: overlay(fresh.properties, properties),
        ..fresh
    }
}

fn import_bindings(body: &PresetBody, preset: &mut Preset, warnings: &mut Vec<ImportWarning>) {
    for (key, binding_doc) in &body.bindings {
        let (targets, dangling) = import_targets(key, binding_doc, preset);
        warnings.extend(dangling);
        if targets.is_empty() {
            if binding_doc.parameters.is_empty() {
                let key = key.clone();
                warnings.push(if binding_doc.properties.is_empty() {
                    ImportWarning::EmptyBinding { key }
                } else {
                    ImportWarning::PropertyBinding { key }
                });
            }
            continue;
        }
        let name = binding_doc.name.clone().unwrap_or_else(|| {
            targets
                .first()
                .map_or_else(String::new, |target| target.symbol.clone())
        });
        let binding = Binding {
            name,
            targets,
            value: binding_doc.value.clamp(0.0, 1.0),
            properties: binding_doc.properties.clone(),
            extra: binding_doc.extra.clone(),
        };
        match Actuator::from_preset_key(key) {
            Some(actuator) => preset.set_binding(actuator, Some(binding)),
            None => preset.set_extra_binding(key, Some(binding)),
        }
    }
}

fn import_targets(
    key: &str,
    binding_doc: &BindingDoc,
    preset: &Preset,
) -> (Vec<BindingTarget>, Vec<ImportWarning>) {
    partition_results(binding_doc.parameters.iter().map(|target| {
        Cell::from_document(target.row, target.block)
            .and_then(|cell| preset.block(cell))
            .map(|block| {
                let (min, max) = target_range(target, block);
                BindingTarget {
                    block: block.id,
                    symbol: target.symbol.clone(),
                    min,
                    max,
                    extra: target.extra.clone(),
                }
            })
            .ok_or_else(|| ImportWarning::DanglingBinding {
                key: key.to_owned(),
                row: target.row,
                block: target.block,
            })
    }))
}

fn unique_keys<K: Ord + Copy, V>(
    entries: impl IntoIterator<Item = (String, Option<K>, V)>,
) -> impl Iterator<Item = Result<(K, V), ImportWarning>> {
    entries
        .into_iter()
        .scan(BTreeSet::new(), |seen, (path, key, value)| {
            Some(match key {
                Some(key) if seen.insert(key) => Ok((key, value)),
                Some(_) => Err(ImportWarning::DuplicateKey { path }),
                None => Err(ImportWarning::BadKey { path }),
            })
        })
}

fn extras_by<'doc>(
    entries: impl Iterator<Item = (&'doc String, &'doc Map<String, Value>)>,
) -> BTreeMap<String, Map<String, Value>> {
    entries
        .filter(|(_, extra)| !extra.is_empty())
        .map(|(key, extra)| (key.clone(), extra.clone()))
        .collect()
}

fn extra_of(extras: &BTreeMap<String, Map<String, Value>>, key: &str) -> Map<String, Value> {
    extras.get(key).cloned().unwrap_or_default()
}

fn partition_results<T, E>(results: impl IntoIterator<Item = Result<T, E>>) -> (Vec<T>, Vec<E>) {
    results
        .into_iter()
        .fold((Vec::new(), Vec::new()), |(mut oks, mut errs), result| {
            match result {
                Ok(ok) => oks.push(ok),
                Err(err) => errs.push(err),
            }
            (oks, errs)
        })
}

fn target_range(target: &BindingTargetDoc, block: &Block) -> (f64, f64) {
    if let (Some(min), Some(max)) = (target.min, target.max) {
        return (min, max);
    }
    if target.symbol == BYPASS_SYMBOL {
        return (0.0, 1.0);
    }
    block
        .model
        .param_index(&target.symbol)
        .and_then(|index| block.spec(index))
        .map_or((0.0, 1.0), |spec| (spec.min, spec.max))
}

fn scene_from_1_based(key: &str) -> Option<Slot> {
    key.parse::<u8>().ok().and_then(Slot::from_number)
}

#[must_use]
pub fn export(preset: &Preset, landing_scene: Slot) -> PresetDocument {
    let scenes = written_scenes(preset, landing_scene);
    let mut chains = preset.blocks().fold(
        BTreeMap::<String, ChainDoc>::new(),
        |mut chains, (cell, block)| {
            let (row, column) = cell.to_document();
            chains.entry(row.to_string()).or_default().blocks.insert(
                column.to_string(),
                export_block(preset, block, &scenes, landing_scene),
            );
            chains
        },
    );
    for (row, extra) in preset.chain_extras() {
        if let Some(chain) = chains.get_mut(row) {
            chain.extra.clone_from(extra);
        }
    }
    let extra_bindings = preset
        .extra_bindings()
        .iter()
        .filter_map(|(key, binding)| Some((key.clone(), export_binding(preset, binding)?)));
    let known_bindings = preset.bindings().iter().filter_map(|(actuator, binding)| {
        export_binding(preset, binding).map(|doc| (actuator.preset_key().to_owned(), doc))
    });
    let bindings: BTreeMap<String, BindingDoc> = extra_bindings.chain(known_bindings).collect();

    let metadata = (!preset.midi_out().is_empty()).then(|| preset.midi_out().to_doc());
    PresetDocument::new(PresetBody {
        name: Some(preset.name().to_owned()),
        uuid: preset.uuid().map(|uuid| uuid.as_str().to_owned()),
        scene: Some(u32::try_from(landing_scene.index()).unwrap_or(0)),
        scene_names: preset
            .scene_names()
            .iter()
            .map(|(scene, name)| (scene.number().to_string(), name.clone()))
            .collect(),
        bindings,
        chains,
        metadata,
        extra: preset.extra().clone(),
    })
}

fn export_binding(preset: &Preset, binding: &Binding) -> Option<BindingDoc> {
    let parameters: Vec<BindingTargetDoc> = binding
        .targets
        .iter()
        .filter_map(|target| {
            let cell = preset.cell_of(target.block)?;
            let (row, block) = cell.to_document();
            let range = (!is_range_unknown(preset.block(cell)?, target)).then_some(target);
            Some(BindingTargetDoc {
                block,
                row,
                min: range.map(|known| known.min),
                max: range.map(|known| known.max),
                symbol: target.symbol.clone(),
                extra: target.extra.clone(),
            })
        })
        .collect();
    (!parameters.is_empty()).then(|| BindingDoc {
        name: Some(binding.name.clone()),
        parameters,
        properties: binding.properties.clone(),
        value: binding.value.clamp(0.0, 1.0),
        extra: binding.extra.clone(),
    })
}

fn is_range_unknown(block: &Block, target: &BindingTarget) -> bool {
    block
        .model
        .param_index(&target.symbol)
        .and_then(|index| block.spec(index))
        .is_some_and(|spec| {
            spec.kind == ParamKind::Opaque && target.min.total_cmp(&target.max).is_eq()
        })
}

fn written_scenes(preset: &Preset, landing_scene: Slot) -> Vec<Slot> {
    let last = preset
        .last_defined_scene()
        .map_or(landing_scene, |found| found.max(landing_scene))
        .last_of_group();
    Slot::all().take_while(|scene| *scene <= last).collect()
}

fn rebased_override(
    block: &Block,
    own: Option<&BlockOverride>,
    landing: Option<&BlockOverride>,
) -> BlockOverride {
    let mut rebased = own.cloned().unwrap_or_default();
    let Some(landing) = landing else {
        return rebased;
    };
    if landing
        .enabled
        .is_some_and(|enabled| enabled != block.enabled)
    {
        rebased.enabled.get_or_insert(block.enabled);
    }
    for (index, landing_value) in &landing.values {
        if let Some(base) = block
            .value(*index)
            .filter(|base| base.total_cmp(landing_value).is_ne())
        {
            rebased.values.entry(*index).or_insert(base);
        }
    }
    rebased
}

fn export_block(preset: &Preset, block: &Block, scenes: &[Slot], landing_scene: Slot) -> BlockDoc {
    let landing = preset.scene_override(block.id, landing_scene);
    let landing_value = |index: usize| {
        landing
            .and_then(|found| found.values.get(&index).copied())
            .or_else(|| block.value(index))
    };
    let saved: Vec<(usize, &ParamSpec)> = block
        .model
        .params
        .iter()
        .enumerate()
        .filter(|(_, spec)| spec.should_save())
        .collect();
    let parameters = saved
        .iter()
        .zip(1..)
        .map(|((index, spec), key)| {
            (
                key.to_string(),
                ParamDoc {
                    name: Some(spec.name.clone()),
                    symbol: spec.symbol.clone(),
                    value: landing_value(*index).unwrap_or(spec.default),
                    extra: extra_of(&block.field_extras.params, &spec.symbol),
                },
            )
        })
        .collect();
    let properties = block
        .model
        .properties
        .iter()
        .zip(&block.properties)
        .zip(1..)
        .map(|((spec, value), key)| {
            (
                key.to_string(),
                PropertyDoc {
                    name: Some(spec.name.clone()),
                    uri: spec.uri.clone(),
                    value: value.clone(),
                    extra: extra_of(&block.field_extras.properties, &spec.uri),
                },
            )
        })
        .collect();
    let scenes = scenes
        .iter()
        .map(|scene| {
            let override_ =
                rebased_override(block, preset.scene_override(block.id, *scene), landing);
            let delta_parameters = override_
                .values
                .iter()
                .filter_map(|(index, value)| {
                    let spec = block.spec(*index)?;
                    spec.allowed_in_scenes().then(|| SymbolValue {
                        symbol: spec.symbol.clone(),
                        value: *value,
                        extra: extra_of(&override_.value_extras, &spec.symbol),
                    })
                })
                .collect();
            (
                scene.number().to_string(),
                SceneDeltaDoc {
                    enabled: override_.enabled,
                    parameters: delta_parameters,
                    properties: override_.properties,
                    extra: override_.extra,
                },
            )
        })
        .filter(|(_, delta)| {
            delta.enabled.is_some()
                || !delta.parameters.is_empty()
                || !delta.extra.is_empty()
                || delta
                    .properties
                    .as_ref()
                    .is_some_and(|found| !found.is_empty())
        })
        .collect();
    BlockDoc {
        uri: block.model.uri.clone(),
        enabled: Some(
            landing
                .and_then(|found| found.enabled)
                .unwrap_or(block.enabled),
        ),
        parameters,
        properties,
        quickpot: Some(block.quickpot.clone().unwrap_or_default()),
        scenes,
        extra: block.extra.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use serde_json::{Value, json};

    use super::{ImportWarning, export, import, partition_results, unique_keys};
    use crate::{
        model::{
            catalog::Catalog,
            midi_out::{MidiOutMessage, MidiOutValue},
            preset::{Cell, EditScope, Side},
            slot::Slot,
            testing::{self, DRIVE, GAIN, REVERB},
        },
        protocol::{
            actuator::Actuator,
            hid::{preset::PresetDocument, uuid::PresetUuid},
        },
    };

    const UUID: &str = "3dd5c292e7834ee0-9d5dbafd-304c14df-bee01bc7fd76cb85606e3dc2";

    fn document(chains: Value, bindings: Value) -> PresetDocument {
        serde_json::from_value(json!({
            "preset": {
                "name": "TEST",
                "uuid": UUID,
                "scene": 1,
                "sceneNames": { "2": "Loud" },
                "bindings": bindings,
                "chains": chains,
                "metadata": {
                    "midi_messages_on_preset_change": [
                        { "type": 0, "channel": 1, "number": 20, "value": 5 }
                    ],
                    "midi_messages_on_scene_change": {
                        "1": [{ "type": 1, "channel": 2, "slotNumber": true }]
                    }
                },
                "background": "blue"
            },
            "type": "preset",
            "version": 1
        }))
        .expect("document parses")
    }

    fn drive_block(drive_symbol: &str) -> Value {
        json!({
            "uri": DRIVE,
            "enabled": true,
            "parameters": {
                "1": { "name": "Drive", "symbol": drive_symbol, "value": 7.0 },
                "2": { "name": "Level", "symbol": "level", "value": -3.0 },
                "3": { "name": "Mode", "symbol": "mode", "value": 2.0 }
            },
            "properties": {},
            "quickpot": "",
            "scenes": {
                "2": { "enabled": false, "parameters": [{ "symbol": drive_symbol, "value": 9.0 }] }
            }
        })
    }

    fn valid_document() -> PresetDocument {
        document(
            json!({
                "1": { "blocks": { "1": drive_block("drive"), "3": { "uri": GAIN, "enabled": true, "parameters": {} } } }
            }),
            json!({
                "foot1": {
                    "name": "Drive",
                    "parameters": [{ "block": 1, "row": 1, "min": 0.0, "max": 10.0, "symbol": "drive" }],
                    "properties": [],
                    "value": 0.3
                }
            }),
        )
    }

    #[test]
    fn a_document_maps_onto_the_grid_scenes_bindings_and_midi_out() {
        let imported = import(&valid_document(), &testing::catalog());
        assert!(imported.warnings.is_empty(), "{:?}", imported.warnings);
        let preset = &imported.preset;
        let drive = Cell { row: 0, column: 0 };
        let second_scene = Slot::from_index(1).expect("scene");

        assert_eq!(preset.name(), "TEST");
        assert_eq!(preset.uuid().map(PresetUuid::as_str), Some(UUID));
        assert_eq!(imported.landing_scene, second_scene);
        assert_eq!(
            preset.block(drive).map(|block| block.model.uri.as_str()),
            Some(DRIVE)
        );
        assert_eq!(
            preset
                .block(Cell { row: 0, column: 2 })
                .map(|block| block.model.uri.as_str()),
            Some(GAIN)
        );
        assert_eq!(preset.value(drive, 0, None), Some(7.0));
        assert_eq!(preset.value(drive, 1, None), Some(-3.0));
        assert_eq!(preset.value(drive, 0, Some(second_scene)), Some(9.0));
        assert_eq!(preset.is_enabled(drive, Some(second_scene)), Some(false));
        assert_eq!(preset.scene_name(second_scene), Some("Loud"));
        assert_eq!(preset.extra().get("background"), Some(&json!("blue")));

        let binding = preset.binding(Actuator::FootA).expect("bound");
        let target = binding.target().expect("target");
        assert_eq!(target.block, preset.block(drive).expect("drive").id);
        assert_eq!(target.symbol, "drive");
        assert_eq!((target.min, target.max), (0.0, 10.0));

        let midi_out = preset.midi_out();
        assert_eq!(
            midi_out.on_preset,
            [MidiOutMessage::ControlChange {
                channel: 1,
                controller: 20,
                value: MidiOutValue::Fixed(5)
            }]
        );
        assert_eq!(
            midi_out.on_scene.get(&second_scene),
            Some(&vec![MidiOutMessage::ProgramChange {
                channel: 2,
                program: MidiOutValue::SlotNumber
            }])
        );
    }

    #[test]
    fn export_is_stable_across_a_round_trip() {
        let catalog = testing::catalog();
        let first = import(&valid_document(), &catalog);
        let exported = export(&first.preset, first.landing_scene);
        let second = import(&exported, &catalog);
        assert!(second.warnings.is_empty(), "{:?}", second.warnings);
        let again = export(&second.preset, second.landing_scene);
        assert_eq!(
            serde_json::to_value(&again).expect("serialises"),
            serde_json::to_value(&exported).expect("serialises")
        );

        let value = serde_json::to_value(&exported).expect("serialises");
        let body = &value["preset"];
        assert_eq!(body["scene"], json!(1));
        assert_eq!(body["sceneNames"]["2"], json!("Loud"));
        assert_eq!(body["chains"]["1"]["blocks"]["1"]["uri"], json!(DRIVE));
        assert_eq!(
            body["chains"]["1"]["blocks"]["1"]["parameters"]["1"]["symbol"],
            json!("drive")
        );
        assert_eq!(
            body["chains"]["1"]["blocks"]["1"]["scenes"]["2"]["enabled"],
            json!(false)
        );
        assert_eq!(
            body["bindings"]["foot1"]["parameters"][0]["block"],
            json!(1)
        );
        assert_eq!(body["bindings"]["foot1"]["parameters"][0]["row"], json!(1));
        assert_eq!(
            body["metadata"]["midi_messages_on_scene_change"]["1"][0]["slotNumber"],
            json!(true)
        );
        assert_eq!(value["version"], json!(1));
    }

    #[test]
    fn unknown_fields_below_the_block_survive_a_round_trip() {
        let block = json!({
            "uri": DRIVE,
            "enabled": true,
            "parameters": {
                "1": { "name": "Drive", "symbol": "drive", "value": 7.0, "ramp": 3 }
            },
            "scenes": {
                "2": {
                    "enabled": false,
                    "fade": "slow",
                    "parameters": [{ "symbol": "drive", "value": 9.0, "curve": 1 }]
                }
            }
        });
        let original = document(
            json!({ "1": { "tint": "red", "blocks": { "1": block } } }),
            json!({
                "foot1": {
                    "name": "Drive",
                    "parameters": [{ "block": 1, "row": 1, "min": 0.0, "max": 10.0, "symbol": "drive", "taper": "log" }],
                    "properties": [],
                    "value": 0.3
                }
            }),
        );
        let imported = import(&original, &testing::catalog());
        let value = serde_json::to_value(export(&imported.preset, imported.landing_scene))
            .expect("serialises");
        let chain = &value["preset"]["chains"]["1"];
        assert_eq!(chain["tint"], json!("red"));
        assert_eq!(chain["blocks"]["1"]["parameters"]["1"]["ramp"], json!(3));
        assert_eq!(chain["blocks"]["1"]["scenes"]["2"]["fade"], json!("slow"));
        assert_eq!(
            chain["blocks"]["1"]["scenes"]["2"]["parameters"][0]["curve"],
            json!(1)
        );
        assert_eq!(
            value["preset"]["bindings"]["foot1"]["parameters"][0]["taper"],
            json!("log")
        );
    }

    #[test]
    fn unknown_plugins_symbols_and_dangling_bindings_are_reported() {
        let imported = import(
            &document(
                json!({
                    "1": {
                        "blocks": {
                            "1": drive_block("bogus"),
                            "2": { "uri": "urn:example:mystery", "parameters": { "1": { "symbol": "x", "value": 1.0 } } }
                        }
                    }
                }),
                json!({
                    "pot1": {
                        "name": "Nothing",
                        "parameters": [{ "block": 5, "row": 1, "min": 0.0, "max": 1.0, "symbol": "drive" }],
                        "properties": [],
                        "value": 0.0
                    }
                }),
            ),
            &testing::catalog(),
        );
        let warnings = imported.warnings;
        assert!(warnings.iter().any(|warning| matches!(
            warning,
            ImportWarning::UnknownSymbol { symbol, .. } if symbol == "bogus"
        )));
        assert!(warnings.iter().any(|warning| matches!(
            warning,
            ImportWarning::UnknownPlugin { uri, .. } if uri == "urn:example:mystery"
        )));
        assert!(warnings.iter().any(|warning| matches!(
            warning,
            ImportWarning::DanglingBinding { key, block: 5, .. } if key == "pot1"
        )));
        let preset = imported.preset;
        assert!(preset.binding(Actuator::Knob1).is_none());
        assert!(
            preset
                .block(Cell { row: 0, column: 1 })
                .is_some_and(|block| block.model.is_unknown())
        );
        assert_eq!(preset.value(Cell { row: 0, column: 0 }, 0, None), Some(5.0));
    }

    #[test]
    fn a_cell_written_twice_keeps_the_lowest_key_and_warns() {
        let imported = import(
            &document(
                json!({
                    "01": { "blocks": { "1": { "uri": GAIN, "parameters": {} } } },
                    "1": { "blocks": { "1": drive_block("drive"), "01": drive_block("drive") } }
                }),
                json!({}),
            ),
            &testing::catalog(),
        );
        let paths: Vec<&str> = imported
            .warnings
            .iter()
            .filter_map(|warning| match warning {
                ImportWarning::DuplicateKey { path } => Some(path.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(paths, ["chains.1.blocks.01", "chains.1.blocks.1"]);
        assert_eq!(
            imported
                .preset
                .block(Cell { row: 0, column: 0 })
                .map(|block| block.model.uri.as_str()),
            Some(GAIN)
        );
    }

    #[test]
    fn a_scene_written_twice_keeps_the_lowest_key_and_warns() {
        let mut block = drive_block("drive");
        block["scenes"] = json!({
            "02": { "parameters": [{ "symbol": "drive", "value": 4.0 }] },
            "2": { "parameters": [{ "symbol": "drive", "value": 9.0 }] }
        });
        let mut doc = document(json!({ "1": { "blocks": { "1": block } } }), json!({}));
        doc.preset
            .scene_names
            .insert("02".to_owned(), "Quiet".to_owned());
        let imported = import(&doc, &testing::catalog());
        let second_scene = Slot::from_index(1).expect("scene");
        let paths: Vec<&str> = imported
            .warnings
            .iter()
            .filter_map(|warning| match warning {
                ImportWarning::DuplicateKey { path } => Some(path.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(paths, ["chains.1.blocks.1.scenes.2", "sceneNames.2"]);
        let preset = &imported.preset;
        assert_eq!(
            preset.value(Cell { row: 0, column: 0 }, 0, Some(second_scene)),
            Some(4.0)
        );
        assert_eq!(preset.scene_name(second_scene), Some("Quiet"));
    }

    fn round_trip(doc: &PresetDocument, catalog: &Catalog) -> Value {
        let imported = import(doc, catalog);
        serde_json::to_value(export(&imported.preset, imported.landing_scene)).expect("serialises")
    }

    #[test]
    fn scene_deltas_drop_params_scenes_cannot_hold_and_keep_their_properties() {
        let doc = document(
            json!({ "1": { "blocks": {
                "1": {
                    "uri": REVERB,
                    "parameters": {
                        "1": { "symbol": "size", "value": 5.0 },
                        "2": { "symbol": "mix", "value": 0.5 }
                    },
                    "scenes": {
                        "1": { "parameters": [{ "symbol": "size", "value": 9.0 }, { "symbol": "mix", "value": 0.1 }], "properties": [] },
                        "2": { "parameters": [{ "symbol": "mix", "value": 0.2 }] },
                        "3": { "parameters": [], "properties": [] }
                    }
                }
            } } }),
            json!({}),
        );
        let catalog = testing::catalog();
        let imported = import(&doc, &catalog);
        let at = Cell { row: 0, column: 0 };
        let first = Slot::from_index(0).expect("scene");
        assert_eq!(imported.preset.value(at, 0, Some(first)), Some(5.0));
        assert_eq!(imported.preset.value(at, 1, Some(first)), Some(0.1));
        assert!(imported.warnings.contains(&ImportWarning::NotInScenes {
            cell: at,
            uri: REVERB.to_owned(),
            symbol: "size".to_owned()
        }));
        let written = round_trip(&doc, &catalog);
        let block = &written["preset"]["chains"]["1"]["blocks"]["1"];
        assert_eq!(block["parameters"]["2"]["value"], json!(0.2));
        let scenes = &block["scenes"];
        assert_eq!(
            scenes["1"],
            json!({ "parameters": [{ "symbol": "mix", "value": 0.1 }], "properties": [] })
        );
        assert_eq!(
            scenes["2"],
            json!({ "parameters": [{ "symbol": "mix", "value": 0.2 }] })
        );
        assert_eq!(
            scenes["3"],
            json!({ "parameters": [{ "symbol": "mix", "value": 0.5 }] })
        );
    }

    #[test]
    fn unknown_plugin_bindings_keep_their_range_or_its_absence() {
        let doc = document(
            json!({ "1": { "blocks": { "1": {
                "uri": "urn:example:mystery",
                "parameters": { "1": { "symbol": "depth", "value": 4.0999 } }
            } } } }),
            json!({
                "pot1": { "parameters": [{ "block": 1, "row": 1, "symbol": "depth" }], "value": 0.0 },
                "pot2": { "parameters": [{ "block": 1, "row": 1, "min": 0.0, "max": 1.0, "symbol": "depth" }], "value": 0.0 }
            }),
        );
        let bindings = &round_trip(&doc, &testing::catalog())["preset"]["bindings"];
        let open_range = &bindings["pot1"]["parameters"][0];
        assert!(open_range.get("min").is_none() && open_range.get("max").is_none());
        assert_eq!(bindings["pot2"]["parameters"][0]["min"], json!(0.0));
        assert_eq!(bindings["pot2"]["parameters"][0]["max"], json!(1.0));
    }

    #[test]
    fn bindings_under_unknown_keys_follow_moves_and_leave_with_their_block() {
        let doc = document(
            json!({ "1": { "blocks": { "1": drive_block("drive") } } }),
            json!({
                "foot4": { "name": "Future", "parameters": [{ "block": 1, "row": 1, "min": 0.0, "max": 10.0, "symbol": "drive" }], "value": 0.0 }
            }),
        );
        let mut preset = import(&doc, &testing::catalog()).preset;
        assert_eq!(
            preset.insert(
                Cell { row: 0, column: 0 },
                Cell { row: 1, column: 3 },
                Side::Right
            ),
            Ok(())
        );
        let moved = serde_json::to_value(export(&preset, Slot::default())).expect("serialises");
        let target = &moved["preset"]["bindings"]["foot4"]["parameters"][0];
        assert_eq!((&target["row"], &target["block"]), (&json!(2), &json!(4)));
        assert_eq!(
            moved["preset"]["bindings"]["foot4"]["name"],
            json!("Future")
        );

        preset.remove(Cell { row: 1, column: 3 });
        let removed = serde_json::to_value(export(&preset, Slot::default())).expect("serialises");
        assert!(removed["preset"]["bindings"].get("foot4").is_none());
    }

    #[test]
    fn bindings_without_targets_are_reported() {
        let imported = import(
            &document(
                json!({ "1": { "blocks": { "1": drive_block("drive") } } }),
                json!({
                    "pot3": { "parameters": [], "properties": [{ "uri": "urn:x" }], "value": 0.0 },
                    "pot4": { "parameters": [], "properties": [], "value": 0.0 },
                    "foot9": { "value": 0.0 }
                }),
            ),
            &testing::catalog(),
        );
        let reported: Vec<&ImportWarning> = imported
            .warnings
            .iter()
            .filter(|warning| {
                matches!(
                    warning,
                    ImportWarning::PropertyBinding { .. } | ImportWarning::EmptyBinding { .. }
                )
            })
            .collect();
        assert_eq!(
            reported,
            [
                &ImportWarning::EmptyBinding {
                    key: "foot9".to_owned()
                },
                &ImportWarning::PropertyBinding {
                    key: "pot3".to_owned()
                },
                &ImportWarning::EmptyBinding {
                    key: "pot4".to_owned()
                },
            ]
        );
    }

    #[test]
    fn params_load_like_the_device_in_numeric_order_until_a_gap_with_the_last_duplicate_winning() {
        let mut block = drive_block("drive");
        block["parameters"]["4"] = json!({ "symbol": "drive", "value": 1.0 });
        block["parameters"]["10"] = json!({ "symbol": "level", "value": 9.0 });
        block["scenes"] = json!({
            "2": { "parameters": [{ "symbol": "drive", "value": 9.0 }, { "symbol": "drive", "value": 4.0 }] }
        });
        let imported = import(
            &document(json!({ "1": { "blocks": { "1": block } } }), json!({})),
            &testing::catalog(),
        );
        let at = Cell { row: 0, column: 0 };
        let preset = &imported.preset;
        assert_eq!(preset.value(at, 0, None), Some(1.0));
        assert_eq!(preset.value(at, 1, None), Some(-3.0));
        assert_eq!(
            preset.value(at, 0, Some(Slot::from_index(1).expect("scene"))),
            Some(4.0)
        );
        let flagged = |path: &str| {
            imported
                .warnings
                .iter()
                .any(|warning| matches!(warning, ImportWarning::DuplicateKey { path: found } if found == path))
        };
        assert!(flagged("chains.1.blocks.1.parameters.1"));
        assert!(!flagged("chains.1.blocks.1.parameters.4"));
        assert!(flagged("chains.1.blocks.1.scenes.2.parameters.0"));
        assert!(imported.warnings.contains(&ImportWarning::BadKey {
            path: "chains.1.blocks.1.parameters.10".to_owned()
        }));
    }

    #[test]
    fn an_unknown_plugin_keeps_both_entries_that_share_a_symbol() {
        let parameters: serde_json::Map<String, Value> = (1..=10)
            .map(|number| {
                let symbol = if number == 10 {
                    "p2".to_owned()
                } else {
                    format!("p{number}")
                };
                (
                    number.to_string(),
                    json!({ "symbol": symbol, "value": f64::from(number) }),
                )
            })
            .collect();
        let doc = document(
            json!({ "1": { "blocks": { "1": { "uri": "urn:example:twin", "parameters": parameters } } } }),
            json!({}),
        );
        let written = &round_trip(&doc, &testing::catalog())["preset"]["chains"]["1"]["blocks"]["1"]
            ["parameters"];
        assert_eq!(written["2"]["value"], json!(2.0));
        assert_eq!(written["10"]["value"], json!(10.0));
    }

    #[test]
    fn an_unknown_plugin_keeps_more_than_nine_params_in_numeric_order() {
        let parameters: serde_json::Map<String, Value> = (1..=11)
            .map(|number| {
                (
                    number.to_string(),
                    json!({ "name": format!("P{number}"), "symbol": format!("p{number}"), "value": f64::from(number) }),
                )
            })
            .collect();
        let doc = document(
            json!({ "1": { "blocks": { "1": { "uri": "urn:example:wide", "parameters": parameters } } } }),
            json!({}),
        );
        let catalog = testing::catalog();
        let imported = import(&doc, &catalog);
        let block = imported
            .preset
            .block(Cell { row: 0, column: 0 })
            .expect("placed");
        assert_eq!(block.model.params[1].symbol, "p2");
        assert_eq!(block.model.params[9].symbol, "p10");
        assert_eq!(block.value(9), Some(10.0));
        let written =
            &round_trip(&doc, &catalog)["preset"]["chains"]["1"]["blocks"]["1"]["parameters"];
        assert_eq!(written["2"]["symbol"], json!("p2"));
        assert_eq!(written["10"]["symbol"], json!("p10"));
        assert_eq!(written["10"]["value"], json!(10.0));
    }

    #[test]
    fn a_scene_edit_of_a_param_scenes_cannot_hold_survives_export() {
        let doc = document(
            json!({ "1": { "blocks": { "1": { "uri": REVERB, "parameters": {} } } } }),
            json!({}),
        );
        let mut preset = import(&doc, &testing::catalog()).preset;
        let scene = Slot::from_index(1).expect("scene");
        preset.set_value(Cell { row: 0, column: 0 }, 0, 9.0, EditScope::Scene(scene));
        let written = serde_json::to_value(export(&preset, scene)).expect("serialises");
        let block = &written["preset"]["chains"]["1"]["blocks"]["1"];
        assert_eq!(block["parameters"]["1"]["value"], json!(9.0));
        assert!(
            block["scenes"]
                .as_object()
                .is_some_and(serde_json::Map::is_empty)
        );
    }

    #[test]
    fn the_top_level_state_is_the_landing_scene_because_the_device_plays_it_on_load() {
        let doc = document(
            json!({ "1": { "blocks": { "1": {
                "uri": DRIVE,
                "enabled": false,
                "parameters": { "1": { "symbol": "drive", "value": 1.0 } },
                "scenes": { "2": { "enabled": true, "parameters": [{ "symbol": "drive", "value": 7.0 }] } }
            } } } }),
            json!({}),
        );
        let imported = import(&doc, &testing::catalog());
        let written = serde_json::to_value(export(&imported.preset, imported.landing_scene))
            .expect("serialises");
        let block = &written["preset"]["chains"]["1"]["blocks"]["1"];
        assert_eq!(
            (&block["enabled"], &block["parameters"]["1"]["value"]),
            (&json!(true), &json!(7.0))
        );
        let scene = |key: &str| {
            (
                block["scenes"][key]["enabled"].clone(),
                block["scenes"][key]["parameters"][0]["value"].clone(),
            )
        };
        assert_eq!(scene("1"), (json!(false), json!(1.0)));
        assert_eq!(scene("2"), (json!(true), json!(7.0)));
        assert_eq!(scene("3"), (json!(false), json!(1.0)));
        assert!(block["scenes"].get("4").is_none());

        let again = import(
            &serde_json::from_value(written.clone()).expect("parses"),
            &testing::catalog(),
        );
        let rewritten =
            serde_json::to_value(export(&again.preset, again.landing_scene)).expect("serialises");
        assert_eq!(rewritten, written);
    }

    fn factory_presets() -> Option<Vec<(String, PresetDocument)>> {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../midi-protocol/device/factory-data");
        let presets = fs::read_dir(root)
            .ok()?
            .map(|entry| entry.expect("readable entry"))
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("presets"))
            .flat_map(|entry| fs::read_dir(entry.path()).expect("readable directory"))
            .map(|entry| {
                let path = entry.expect("readable entry").path();
                let text = fs::read_to_string(&path).expect("readable preset");
                let doc = serde_json::from_str(&text)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                (path.display().to_string(), doc)
            })
            .collect();
        Some(presets)
    }

    type Range = Option<(f64, f64)>;

    fn binding_targets(body: &Value) -> Vec<(String, Range)> {
        members(&body["bindings"])
            .flat_map(|(key, binding)| {
                binding["parameters"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(move |target| {
                        let place = format!(
                            "{key}:{}.{}.{}",
                            target["row"], target["block"], target["symbol"]
                        );
                        let range = target["min"].as_f64().zip(target["max"].as_f64());
                        (place, range)
                    })
            })
            .collect()
    }

    fn keeps_range(place: &str, original: Range, written: Range) -> bool {
        let is_bypass = place.ends_with(r#"":bypass""#);
        match (original, written) {
            (Some(before), Some(after)) => before == after,
            (None, Some(after)) => is_bypass && after == (0.0, 1.0),
            (None, None) => true,
            (Some(_), None) => false,
        }
    }

    fn members(value: &Value) -> impl Iterator<Item = (&String, &Value)> {
        value.as_object().into_iter().flatten()
    }

    fn scene_parameters(body: &Value) -> Vec<(String, Vec<(String, Value)>)> {
        members(&body["chains"])
            .flat_map(|(row, chain)| {
                members(&chain["blocks"]).flat_map(move |(column, block)| {
                    members(&block["scenes"]).map(move |(scene, delta)| {
                        let mut parameters: Vec<(String, Value)> = delta["parameters"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .map(|entry| (entry["symbol"].to_string(), entry["value"].clone()))
                            .collect();
                        parameters.sort_by(|left, right| left.0.cmp(&right.0));
                        (format!("{row}.{column}.{scene}"), parameters)
                    })
                })
            })
            .filter(|(_, parameters)| !parameters.is_empty())
            .collect()
    }

    #[test]
    #[cfg_attr(miri, ignore = "reads the factory presets from disk")]
    fn factory_presets_as_unknown_plugins_keep_binding_ranges_and_scene_deltas() {
        let Some(presets) = factory_presets() else {
            return;
        };
        assert_ne!(presets.len(), 0);
        let catalog = Catalog::default();
        for (path, doc) in presets {
            let original = serde_json::to_value(&doc).expect("serialises");
            let written = round_trip(&doc, &catalog);
            let before = binding_targets(&original["preset"]);
            let after = binding_targets(&written["preset"]);
            assert_eq!(before.len(), after.len(), "{path}");
            for ((place, range), (written_place, written_range)) in before.into_iter().zip(after) {
                assert_eq!(place, written_place, "{path}");
                assert!(keeps_range(&place, range, written_range), "{path} {place}");
            }
            assert_eq!(
                scene_parameters(&written["preset"]),
                scene_parameters(&original["preset"]),
                "{path}"
            );
        }
    }

    #[test]
    fn unique_keys_keeps_the_first_and_reports_the_rest_in_order() {
        let results: Vec<_> = unique_keys([
            ("a".to_owned(), Some(1), 'a'),
            ("b".to_owned(), None, 'b'),
            ("c".to_owned(), Some(1), 'c'),
            ("d".to_owned(), Some(2), 'd'),
        ])
        .collect();
        assert_eq!(
            results,
            [
                Ok((1, 'a')),
                Err(ImportWarning::BadKey {
                    path: "b".to_owned()
                }),
                Err(ImportWarning::DuplicateKey {
                    path: "c".to_owned()
                }),
                Ok((2, 'd')),
            ]
        );
    }

    #[test]
    fn partition_results_splits_in_order() {
        assert_eq!(
            partition_results([Ok(1), Err('x'), Ok(2), Err('y')]),
            (vec![1, 2], vec!['x', 'y'])
        );
    }
}
