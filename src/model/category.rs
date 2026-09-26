#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    AmpCab,
    Drive,
    Dynamics,
    Modulation,
    Ambience,
    Utility,
}

impl Category {
    pub const ALL: [Self; 6] = [
        Self::AmpCab,
        Self::Drive,
        Self::Dynamics,
        Self::Modulation,
        Self::Ambience,
        Self::Utility,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AmpCab => "Amps & Cabs",
            Self::Drive => "Overdrive & Distortion",
            Self::Dynamics => "EQs, Filters & Dynamics",
            Self::Modulation => "Modulation, Synths & Pitch",
            Self::Ambience => "Delays & Reverbs",
            Self::Utility => "Loaders & Utilities",
        }
    }
}
