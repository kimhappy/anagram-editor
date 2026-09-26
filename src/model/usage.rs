use crate::protocol::hid::preset::{BlockDoc, PresetDocument};

const NAMED_USERS: usize = 6;

fn blocks(document: &PresetDocument) -> impl Iterator<Item = &BlockDoc> {
    document
        .preset
        .chains
        .values()
        .flat_map(|chain| chain.blocks.values())
}

#[must_use]
pub fn uses_file(document: &PresetDocument, device_path: &str) -> bool {
    blocks(document).any(|block| {
        block
            .properties
            .values()
            .any(|property| property.value == device_path)
    })
}

#[must_use]
pub fn uses_plugin(document: &PresetDocument, is_plugin: impl Fn(&str) -> bool) -> bool {
    blocks(document).any(|block| is_plugin(&block.uri))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub users: Vec<String>,
    pub unread: usize,
}

impl Usage {
    #[must_use]
    pub fn question(&self, subject: &str, consequence: &str) -> String {
        let found = match self.users.as_slice() {
            [] => "No preset uses it.".to_owned(),
            users => format!("It is used by {}; {consequence}", named(users)),
        };
        let unread = match self.unread {
            0 => String::new(),
            1 => " 1 preset could not be read.".to_owned(),
            count => format!(" {count} presets could not be read."),
        };
        format!("Delete {subject} from the Anagram? {found}{unread}")
    }
}

fn named(users: &[String]) -> String {
    let shown = users.iter().take(NAMED_USERS).cloned().collect::<Vec<_>>();
    let hidden = users.len().saturating_sub(NAMED_USERS);
    match (shown.as_slice(), hidden) {
        ([only], _) => only.clone(),
        ([rest @ .., last], 0) => format!("{} and {last}", rest.join(", ")),
        (_, hidden) => format!("{} and {hidden} more", shown.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Usage, uses_file, uses_plugin};
    use crate::protocol::hid::preset::PresetDocument;

    fn document() -> PresetDocument {
        serde_json::from_value(json!({
            "preset": { "chains": { "2": { "blocks": { "4": {
                "uri": "urn:darkglass:neural:amp",
                "properties": { "1": { "uri": "urn:model", "value": "/data/user-files/neural-models/C217.nam" } }
            } } } } },
            "type": "preset",
            "version": 1
        }))
        .expect("parses")
    }

    #[test]
    fn a_preset_uses_the_files_its_properties_point_at_and_the_plugins_of_its_blocks() {
        let document = document();
        assert!(uses_file(
            &document,
            "/data/user-files/neural-models/C217.nam"
        ));
        assert!(!uses_file(
            &document,
            "/data/user-files/neural-models/C21.nam"
        ));
        assert!(uses_plugin(&document, |uri| uri == "urn:darkglass:neural:amp"));
        assert!(!uses_plugin(&document, |uri| uri == "urn:darkglass:FETComp"));
    }

    #[test]
    fn the_question_names_the_users_and_counts_the_rest() {
        let users = |count: usize| Usage {
            users: (1..=count).map(|number| format!("P{number}")).collect(),
            unread: 0,
        };
        assert_eq!(
            users(0).question("\"C217\"", "they lose the file."),
            "Delete \"C217\" from the Anagram? No preset uses it."
        );
        assert_eq!(
            users(2).question("\"C217\"", "they lose the file."),
            "Delete \"C217\" from the Anagram? It is used by P1 and P2; they lose the file."
        );
        assert_eq!(
            Usage {
                unread: 2,
                ..users(8)
            }
            .question("\"C217\"", "they lose the file."),
            "Delete \"C217\" from the Anagram? It is used by P1, P2, P3, P4, P5, P6 and 2 more; they lose the file. 2 presets could not be read."
        );
    }
}
