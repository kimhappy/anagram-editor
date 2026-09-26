pub use crate::protocol::midi::Mode;

impl Mode {
    pub const TABS: [(Self, &'static str); 3] = [
        (Self::Preset, "Preset"),
        (Self::Scene, "Scene"),
        (Self::Stomp, "Stomp"),
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneControl {
    #[default]
    All,
    Active,
}

impl SceneControl {
    pub const TABS: [(Self, &'static str); 2] =
        [(Self::All, "Control All"), (Self::Active, "Control Active")];
}
