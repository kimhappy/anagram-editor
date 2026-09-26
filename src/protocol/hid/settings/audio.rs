use serde_json::{Value, json};

use super::{
    Settings,
    value::{Bounds, float},
};
use crate::protocol::name::INPUT_GAIN_NAME;

pub const EQ_GAIN_BOUNDS: Bounds = Bounds::linear(-18.0, 18.0, 0.1, 1, "dB");
pub const EQ_WIDTH_BOUNDS: Bounds = Bounds::linear(0.0, 1.0, 0.01, 2, "");
pub const MIXER_VOLUME_BOUNDS: Bounds = Bounds::linear(-30.0, 6.0, 0.5, 1, "dB");
pub const INPUT_GAIN_DB: std::ops::RangeInclusive<i8> = -12..=12;
pub const INPUT_GAIN_BOUNDS: Bounds = Bounds::linear(-12.0, 12.0, 1.0, 0, "dB");
pub const INPUT_GAIN_PRESETS: usize = 7;

const ACTIVE_INPUT_GAIN: &str = "input-gain-preset";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EqBandId {
    LowShelf,
    Band1,
    Band2,
    Band3,
    Band4,
    HighShelf,
}

impl EqBandId {
    pub const ALL: [Self; 6] = [
        Self::LowShelf,
        Self::Band1,
        Self::Band2,
        Self::Band3,
        Self::Band4,
        Self::HighShelf,
    ];

    const fn key(self) -> &'static str {
        match self {
            Self::LowShelf => "ls",
            Self::Band1 => "1",
            Self::Band2 => "2",
            Self::Band3 => "3",
            Self::Band4 => "4",
            Self::HighShelf => "hs",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LowShelf => "Low",
            Self::Band1 => "1",
            Self::Band2 => "2",
            Self::Band3 => "3",
            Self::Band4 => "4",
            Self::HighShelf => "High",
        }
    }

    #[must_use]
    pub const fn freq_bounds(self) -> Bounds {
        let (min, max) = match self {
            Self::LowShelf => (25.0, 300.0),
            Self::Band1 => (20.0, 320.0),
            Self::Band2 => (80.0, 1250.0),
            Self::Band3 => (320.0, 5000.0),
            Self::Band4 => (1250.0, 20000.0),
            Self::HighShelf => (1500.0, 16000.0),
        };
        Bounds::logarithmic(min, max, 1.0, "Hz")
    }

    const fn default_band(self) -> EqBand {
        let (freq, width) = match self {
            Self::LowShelf => (80.0, 1.0),
            Self::Band1 => (160.0, 0.5),
            Self::Band2 => (397.0, 0.5),
            Self::Band3 => (1250.0, 0.5),
            Self::Band4 => (2500.0, 0.5),
            Self::HighShelf => (8000.0, 1.0),
        };
        EqBand {
            freq,
            gain: 0.0,
            width,
        }
    }

    fn setting(self, field: &str) -> String {
        format!("globaleq.{}.{field}", self.key())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqBand {
    pub freq: f64,
    pub gain: f64,
    pub width: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlobalEq {
    pub bands: [EqBand; 6],
}

impl Default for GlobalEq {
    fn default() -> Self {
        Self {
            bands: EqBandId::ALL.map(EqBandId::default_band),
        }
    }
}

impl GlobalEq {
    #[must_use]
    pub fn read(settings: &Settings) -> Self {
        Self {
            bands: EqBandId::ALL.map(|id| {
                let fallback = id.default_band();
                EqBand {
                    freq: id
                        .freq_bounds()
                        .clamp(settings.f64_or(&id.setting("freq"), fallback.freq)),
                    gain: EQ_GAIN_BOUNDS.clamp(settings.f64_or(&id.setting("gain"), fallback.gain)),
                    width: EQ_WIDTH_BOUNDS
                        .clamp(settings.f64_or(&id.setting("width"), fallback.width)),
                }
            }),
        }
    }

    #[must_use]
    pub fn band(&self, id: EqBandId) -> EqBand {
        EqBandId::ALL
            .iter()
            .zip(self.bands)
            .find_map(|(candidate, band)| (*candidate == id).then_some(band))
            .unwrap_or_else(|| id.default_band())
    }

    pub fn update(&mut self, id: EqBandId, change: impl FnOnce(&mut EqBand)) {
        if let Some(band) = EqBandId::ALL
            .iter()
            .zip(self.bands.iter_mut())
            .find_map(|(candidate, band)| (*candidate == id).then_some(band))
        {
            change(band);
        }
    }

    #[must_use]
    pub fn entries(&self) -> Vec<(String, Value)> {
        EqBandId::ALL
            .iter()
            .zip(self.bands)
            .flat_map(|(id, band)| {
                [
                    (id.setting("freq"), float(band.freq)),
                    (id.setting("gain"), float(band.gain)),
                    (id.setting("width"), float(band.width)),
                ]
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixerOutput {
    JackLeft,
    JackRight,
    XlrLeft,
    XlrRight,
    Headphones,
    Speaker,
}

impl MixerOutput {
    pub const ALL: [Self; 6] = [
        Self::JackLeft,
        Self::JackRight,
        Self::XlrLeft,
        Self::XlrRight,
        Self::Headphones,
        Self::Speaker,
    ];

    const fn key(self) -> &'static str {
        match self {
            Self::JackLeft => "mixer.jack.left",
            Self::JackRight => "mixer.jack.right",
            Self::XlrLeft => "mixer.xlr.left",
            Self::XlrRight => "mixer.xlr.right",
            Self::Headphones => "mixer.headphones",
            Self::Speaker => "mixer.speaker",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::JackLeft => "Jack L",
            Self::JackRight => "Jack R",
            Self::XlrLeft => "XLR L",
            Self::XlrRight => "XLR R",
            Self::Headphones => "Headphones",
            Self::Speaker => "Speaker",
        }
    }

    #[must_use]
    pub const fn follows_master_volume(self) -> bool {
        !matches!(self, Self::Speaker)
    }

    #[must_use]
    pub const fn pair(self) -> Option<StereoPair> {
        match self {
            Self::JackLeft | Self::JackRight => Some(StereoPair::Jack),
            Self::XlrLeft | Self::XlrRight => Some(StereoPair::Xlr),
            Self::Headphones | Self::Speaker => None,
        }
    }

    const fn partner(self) -> Option<Self> {
        match self {
            Self::JackLeft => Some(Self::JackRight),
            Self::JackRight => Some(Self::JackLeft),
            Self::XlrLeft => Some(Self::XlrRight),
            Self::XlrRight => Some(Self::XlrLeft),
            Self::Headphones | Self::Speaker => None,
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::JackLeft => 0,
            Self::JackRight => 1,
            Self::XlrLeft => 2,
            Self::XlrRight => 3,
            Self::Headphones => 4,
            Self::Speaker => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StereoPair {
    Jack,
    Xlr,
}

impl StereoPair {
    pub const ALL: [Self; 2] = [Self::Jack, Self::Xlr];

    const fn key(self) -> &'static str {
        match self {
            Self::Jack => "mixer.jack.left-right-link",
            Self::Xlr => "mixer.xlr.left-right-link",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Jack => "Link jack L and R",
            Self::Xlr => "Link XLR L and R",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Jack => 0,
            Self::Xlr => 1,
        }
    }

    const fn outputs(self) -> (MixerOutput, MixerOutput) {
        match self {
            Self::Jack => (MixerOutput::JackLeft, MixerOutput::JackRight),
            Self::Xlr => (MixerOutput::XlrLeft, MixerOutput::XlrRight),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixerChannel {
    pub volume: f64,
    pub follows_master: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mixer {
    channels: [MixerChannel; 6],
    stereo_links: [bool; 2],
}

impl Default for Mixer {
    fn default() -> Self {
        Self {
            channels: [MixerChannel {
                volume: 0.0,
                follows_master: true,
            }; 6],
            stereo_links: [true; 2],
        }
    }
}

impl Mixer {
    #[must_use]
    pub fn read(settings: &Settings) -> Self {
        let defaults = Self::default();
        Self {
            channels: MixerOutput::ALL.map(|output| MixerChannel {
                volume: MIXER_VOLUME_BOUNDS
                    .clamp(settings.f64_or(&format!("{}.vol", output.key()), 0.0)),
                follows_master: settings.bool_or(&format!("{}.link", output.key()), true),
            }),
            stereo_links: StereoPair::ALL
                .map(|pair| settings.bool_or(pair.key(), defaults.is_linked(pair))),
        }
    }

    #[must_use]
    pub fn channel(&self, output: MixerOutput) -> MixerChannel {
        self.channels
            .get(output.index())
            .copied()
            .unwrap_or(MixerChannel {
                volume: 0.0,
                follows_master: true,
            })
    }

    #[must_use]
    pub fn is_linked(&self, pair: StereoPair) -> bool {
        self.stereo_links.get(pair.index()).copied().unwrap_or(true)
    }

    pub fn set_linked(&mut self, pair: StereoPair, is_linked: bool) {
        if let Some(link) = self.stereo_links.get_mut(pair.index()) {
            *link = is_linked;
        }
        if is_linked {
            let (left, right) = pair.outputs();
            let leading = self.channel(left);
            if let Some(following) = self.channels.get_mut(right.index()) {
                *following = leading;
            }
        }
    }

    pub fn update(&mut self, output: MixerOutput, change: impl Fn(&mut MixerChannel)) {
        let partner = output
            .partner()
            .filter(|_| output.pair().is_some_and(|pair| self.is_linked(pair)));
        for member in std::iter::once(output).chain(partner) {
            if let Some(channel) = self.channels.get_mut(member.index()) {
                change(channel);
            }
        }
    }

    #[must_use]
    pub fn entries(&self) -> Vec<(String, Value)> {
        let channels = MixerOutput::ALL.into_iter().flat_map(|output| {
            let channel = self.channel(output);
            let volume = (format!("{}.vol", output.key()), float(channel.volume));
            let link = output.follows_master_volume().then(|| {
                (
                    format!("{}.link", output.key()),
                    json!(channel.follows_master),
                )
            });
            std::iter::once(volume).chain(link)
        });
        let pairs = StereoPair::ALL
            .into_iter()
            .map(|pair| (pair.key().to_owned(), json!(self.is_linked(pair))));
        channels.chain(pairs).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputGainPreset {
    pub db: i8,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputGain {
    pub active: usize,
    pub presets: Vec<InputGainPreset>,
}

impl Default for InputGain {
    fn default() -> Self {
        Self {
            active: 0,
            presets: (0..INPUT_GAIN_PRESETS)
                .map(|index| InputGainPreset {
                    db: 0,
                    name: default_input_gain_name(index),
                })
                .collect(),
        }
    }
}

fn default_input_gain_name(index: usize) -> String {
    format!("INSTRUMENT {}", index + 1)
}

fn input_gain_key(index: usize, field: &str) -> String {
    format!("{ACTIVE_INPUT_GAIN}{index}.{field}")
}

#[must_use]
pub fn is_input_gain_name(key: &str) -> bool {
    (0..INPUT_GAIN_PRESETS).any(|index| key == input_gain_key(index, "name"))
}

impl InputGain {
    #[must_use]
    pub fn read(settings: &Settings) -> Self {
        let active = usize::try_from(settings.u64_or(ACTIVE_INPUT_GAIN, 0))
            .unwrap_or(0)
            .min(INPUT_GAIN_PRESETS - 1);
        let presets = (0..INPUT_GAIN_PRESETS)
            .map(|index| InputGainPreset {
                db: i8::try_from(settings.i64_or(&input_gain_key(index, "db"), 0))
                    .unwrap_or(0)
                    .clamp(*INPUT_GAIN_DB.start(), *INPUT_GAIN_DB.end()),
                name: settings
                    .str_or(&input_gain_key(index, "name"))
                    .map(|name| INPUT_GAIN_NAME.sanitize(name))
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| default_input_gain_name(index)),
            })
            .collect();
        Self { active, presets }
    }

    #[must_use]
    pub fn entries(&self) -> Vec<(String, Value)> {
        let active = (ACTIVE_INPUT_GAIN.to_owned(), json!(self.active));
        let presets = self.presets.iter().enumerate().flat_map(|(index, preset)| {
            [
                (input_gain_key(index, "db"), json!(preset.db)),
                (input_gain_key(index, "name"), json!(preset.name)),
            ]
        });
        std::iter::once(active).chain(presets).collect()
    }
}
