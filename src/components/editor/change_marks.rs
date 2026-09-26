use crate::protocol::actuator::Actuator;

#[must_use]
pub fn change_marks(is_changed: bool, is_scene_specific: bool) -> String {
    format!(
        "{} {}",
        if is_changed { "italic" } else { "" },
        if is_scene_specific {
            "underline decoration-[length:0.09em] underline-offset-[0.2em]"
        } else {
            ""
        },
    )
}

#[must_use]
pub fn bound_marks(actuators: impl Iterator<Item = Actuator>) -> String {
    actuators.map(Actuator::short).collect::<Vec<_>>().join(" ")
}
