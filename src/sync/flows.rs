use leptos::prelude::*;

use super::fake::{Call, FakeAnagram, document, started, user_slot};
use crate::{host, protocol::hid::files::UserFile};

#[test]
fn loading_a_listed_preset_selects_it_and_adopts_its_content() {
    let fake = FakeAnagram::new([(user_slot("02A"), document("LEAD", 2))]);
    let session = started(&fake);
    session.load(user_slot("02A"));
    host::run_until_idle();
    assert!(
        fake.calls()
            .iter()
            .any(|call| matches!(call, Call::Midi(_)))
    );
    assert_eq!(session.current.get_untracked(), user_slot("02A"));
    assert_eq!(
        session
            .draft
            .with_untracked(|preset| preset.name().to_owned()),
        "LEAD"
    );
}

#[test]
fn a_preset_changed_on_the_device_is_not_deleted() {
    let fake = FakeAnagram::new([(user_slot("03A"), document("OLD", 3))]);
    let session = started(&fake);
    fake.presets
        .borrow_mut()
        .insert(user_slot("03A"), document("NEW", 4));
    session.delete_presets(vec![user_slot("03A").slot]);
    host::run_until_idle();
    assert!(
        !fake
            .calls()
            .iter()
            .any(|call| matches!(call, Call::Delete(_)))
    );
    assert_eq!(fake.stored(user_slot("03A")), Some(document("NEW", 4)));
}

#[test]
fn saving_writes_the_draft_to_the_current_slot() {
    let fake = FakeAnagram::new([(user_slot("01A"), document("LEAD", 1))]);
    let session = started(&fake);
    session.rename("LEAD 2");
    session.save();
    host::run_until_idle();
    assert_eq!(
        fake.stored(user_slot("01A"))
            .and_then(|stored| stored.preset.name),
        Some("LEAD 2".to_owned())
    );
}

#[test]
fn saving_over_a_preset_replaced_on_the_device_asks_first() {
    let fake = FakeAnagram::new([(user_slot("01A"), document("LEAD", 1))]);
    let session = started(&fake);
    fake.presets
        .borrow_mut()
        .insert(user_slot("01A"), document("OTHER", 2));
    session.rename("LEAD 2");
    host::answer([false]);
    session.save();
    host::run_until_idle();
    assert!(
        host::questions()
            .iter()
            .any(|question| question.contains("now holds \"OTHER\""))
    );
    assert!(
        !fake
            .calls()
            .iter()
            .any(|call| matches!(call, Call::Save(..)))
    );
    assert_eq!(fake.stored(user_slot("01A")), Some(document("OTHER", 2)));
}

#[test]
fn a_new_preset_on_a_slot_filled_elsewhere_opens_that_preset() {
    let fake = FakeAnagram::new([]);
    let session = started(&fake);
    fake.presets
        .borrow_mut()
        .insert(user_slot("05A"), document("THEIRS", 5));
    session.new_preset(user_slot("05A"));
    host::run_until_idle();
    assert!(!session.is_new.get_untracked());
    assert_eq!(
        session
            .draft
            .with_untracked(|preset| preset.name().to_owned()),
        "THEIRS"
    );
}

#[test]
fn a_swap_brings_the_scene_back_to_the_stored_one() {
    let mut landing_on_three = document("LEAD", 1);
    landing_on_three.preset.scene = Some(2);
    let fake = FakeAnagram::new([
        (user_slot("01A"), landing_on_three),
        (user_slot("01B"), document("RHYTHM", 2)),
    ]);
    let session = started(&fake);
    let stored_scene = session.scene.get_untracked();
    session.scene.set(crate::model::slot::Slot::default());
    session.swap_presets(user_slot("01A").slot, user_slot("01B").slot);
    host::run_until_idle();
    assert_eq!(session.current.get_untracked(), user_slot("01B"));
    assert_eq!(session.scene.get_untracked(), stored_scene);
    assert_eq!(stored_scene.index(), 2);
}

#[test]
fn a_group_delete_removes_only_unchanged_presets() {
    let fake = FakeAnagram::new([
        (user_slot("02A"), document("ONE", 1)),
        (user_slot("02B"), document("TWO", 2)),
    ]);
    let session = started(&fake);
    fake.presets
        .borrow_mut()
        .insert(user_slot("02B"), document("CHANGED", 3));
    session.delete_presets(vec![user_slot("02A").slot, user_slot("02B").slot]);
    host::run_until_idle();
    assert_eq!(fake.stored(user_slot("02A")), None);
    assert_eq!(fake.stored(user_slot("02B")), Some(document("CHANGED", 3)));
}

#[test]
fn the_listing_poll_reloads_a_clean_draft_and_warns_once_about_edits() {
    let fake = FakeAnagram::new([(user_slot("01A"), document("LEAD", 1))]);
    let session = started(&fake);
    session.link.settling_until.set(0.0);
    fake.presets
        .borrow_mut()
        .insert(user_slot("01A"), document("SAVED ON UNIT", 2));
    host::spawn(async move { session.reconcile_listing().await });
    host::run_until_idle();
    assert_eq!(
        session
            .draft
            .with_untracked(|preset| preset.name().to_owned()),
        "SAVED ON UNIT"
    );

    session.link.settling_until.set(0.0);
    session.rename("MINE");
    fake.presets
        .borrow_mut()
        .insert(user_slot("01A"), document("AGAIN", 3));
    for _ in 0..2 {
        host::spawn(async move { session.reconcile_listing().await });
        host::run_until_idle();
    }
    assert_eq!(
        session
            .draft
            .with_untracked(|preset| preset.name().to_owned()),
        "MINE"
    );
    assert!(
        session
            .notice
            .get_untracked()
            .is_some_and(|notice| notice.is_error && notice.text.contains("\"AGAIN\""))
    );
}

#[test]
fn a_new_tempo_is_written_before_the_restart() {
    let fake = FakeAnagram::new([]);
    let session = started(&fake);
    let opened = session.device_settings.get_untracked();
    let mut target = opened.clone();
    target.general.bpm = 100.0;
    session.apply_device_settings(&opened, &target);
    host::run_until_idle();
    let calls = fake.calls();
    let written = calls.iter().position(|call| {
        matches!(call, Call::Settings(changes)
                if changes.contains_key("transport.bpm") && changes.contains_key("selected-preset"))
    });
    let restarted = calls.iter().position(|call| matches!(call, Call::Reboot));
    assert!(written.is_some() && restarted.is_some() && written < restarted);
}

#[test]
fn a_brightness_change_is_written_without_a_restart() {
    let fake = FakeAnagram::new([]);
    let session = started(&fake);
    let opened = session.device_settings.get_untracked();
    let mut target = opened.clone();
    target.general.brightness = 3;
    session.apply_device_settings(&opened, &target);
    host::run_until_idle();
    assert!(!fake.calls().iter().any(|call| matches!(call, Call::Reboot)));
    assert!(
        fake.calls()
            .iter()
            .any(|call| matches!(call, Call::Settings(_)))
    );
}

#[test]
fn trying_a_binding_sends_its_cc_only_while_cc_enable_is_on() {
    use crate::protocol::{actuator::Actuator, midi::Command};
    let fake = FakeAnagram::new([]);
    let session = started(&fake);
    assert!(session.test_binding(Actuator::Knob2, 90));
    assert_eq!(
        fake.calls().last(),
        Some(&Call::Midi(Command::BindingValue { cc: 21, value: 90 }))
    );
    session
        .device_settings
        .update(|settings| settings.midi.bindings.set_enabled(Actuator::Knob2, false));
    assert!(!session.test_binding(Actuator::Knob2, 10));
    assert_eq!(fake.calls().len(), 1);
}

#[test]
fn a_click_while_loading_leaves_only_the_loading_status() {
    let fake = FakeAnagram::new([
        (user_slot("01A"), document("LEAD", 1)),
        (user_slot("02A"), document("CLEAN", 2)),
    ]);
    let session = started(&fake);
    session.load(user_slot("02A"));
    session.load(user_slot("01A"));
    assert!(session.link.busy.get_untracked().is_some());
    assert_eq!(session.notice.get_untracked(), None);
    host::run_until_idle();
    assert_eq!(session.current.get_untracked(), user_slot("02A"));
}

#[test]
fn deleting_a_file_names_the_presets_that_use_it_first() {
    let mut user = document("LEAD", 2);
    user.preset.chains = serde_json::from_value(serde_json::json!({ "1": { "blocks": { "1": {
        "uri": "urn:darkglass:neural:amp",
        "properties": { "1": { "uri": "urn:model", "value": "/data/user-files/neural-models/C7.nam" } }
    } } } }))
    .expect("chains");
    let fake = FakeAnagram::new([
        (user_slot("01A"), document("CLEAN", 1)),
        (user_slot("02A"), user),
    ]);
    let session = started(&fake);
    let file: UserFile = serde_json::from_value(serde_json::json!({
        "id": 7, "name": "Amp", "file_name": "C7.nam", "dir_name": "neural-models"
    }))
    .expect("file");
    let is_removed = || {
        fake.calls()
            .iter()
            .any(|call| matches!(call, Call::RemoveFile(name) if name == "C7.nam"))
    };

    host::answer([false]);
    session.remove_user_file(file.clone());
    host::run_until_idle();
    assert_eq!(
        host::questions(),
        [
            "Delete \"Amp\" from the Anagram? It is used by 02A \"LEAD\"; those presets lose the file."
        ]
    );
    assert!(!is_removed());

    host::answer([true]);
    session.remove_user_file(file);
    host::run_until_idle();
    assert!(is_removed());
}

#[test]
fn a_curve_drag_that_moves_two_values_is_one_undo_step() {
    let mut lead = document("LEAD", 1);
    lead.preset.chains = serde_json::from_value(serde_json::json!({ "1": { "blocks": { "1": {
        "uri": crate::model::testing::DRIVE,
        "parameters": { "1": { "symbol": "drive", "value": 5.0 }, "2": { "symbol": "level", "value": 0.0 } }
    } } } }))
    .expect("chains");
    let fake = FakeAnagram::new([(user_slot("01A"), lead)]);
    let session = started(&fake);
    let cell = crate::model::preset::Cell { row: 0, column: 0 };
    let values = || {
        session
            .draft
            .with_untracked(|preset| (preset.value(cell, 0, None), preset.value(cell, 1, None)))
    };
    session.set_values_preview(cell, &[(0, 6.0), (1, 2.0)]);
    session.set_values_preview(cell, &[(0, 7.0), (1, 3.0)]);
    session.set_values(cell, &[(0, 8.0), (1, 4.0)]);
    host::run_until_idle();
    assert_eq!(values(), (Some(8.0), Some(4.0)));
    session.undo();
    assert_eq!(values(), (Some(5.0), Some(0.0)));
}
