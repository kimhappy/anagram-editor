use leptos::prelude::*;
use lucide_leptos::{
    Activity, AudioWaveform, BoomBox, Check, ChevronDown, ChevronLeft, ChevronRight,
    ClipboardPaste, Copy, Flame, FolderOpen, LoaderCircle, Plus, Save, Search, Settings,
    SlidersHorizontal, SlidersVertical, SquareStack, Star, Trash, TriangleAlert, Unlink, Upload,
    Usb, WavesHorizontal, X,
};
use tw_merge::tw_merge;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconKind {
    Activity,
    AudioWaveform,
    BoomBox,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ClipboardPaste,
    Copy,
    Flame,
    FolderOpen,
    LoaderCircle,
    Plus,
    Save,
    Search,
    Settings,
    SlidersHorizontal,
    SlidersVertical,
    SquareStackFlipped,
    Star,
    Trash,
    TriangleAlert,
    Unlink,
    Upload,
    Usb,
    WavesHorizontal,
    Close,
}

#[component]
pub fn Icon(kind: IconKind, #[prop(into, optional)] class: String) -> impl IntoView {
    let class = tw_merge!("size-4 shrink-0", flip_class(kind), class.as_str());
    let icon = match kind {
        IconKind::Activity => view! { <Activity /> }.into_any(),
        IconKind::AudioWaveform => view! { <AudioWaveform /> }.into_any(),
        IconKind::BoomBox => view! { <BoomBox /> }.into_any(),
        IconKind::Check => view! { <Check /> }.into_any(),
        IconKind::ChevronDown => view! { <ChevronDown /> }.into_any(),
        IconKind::ChevronLeft => view! { <ChevronLeft /> }.into_any(),
        IconKind::ChevronRight => view! { <ChevronRight /> }.into_any(),
        IconKind::ClipboardPaste => view! { <ClipboardPaste /> }.into_any(),
        IconKind::Copy => view! { <Copy /> }.into_any(),
        IconKind::Flame => view! { <Flame /> }.into_any(),
        IconKind::FolderOpen => view! { <FolderOpen /> }.into_any(),
        IconKind::LoaderCircle => view! { <LoaderCircle /> }.into_any(),
        IconKind::Plus => view! { <Plus /> }.into_any(),
        IconKind::Save => view! { <Save /> }.into_any(),
        IconKind::Search => view! { <Search /> }.into_any(),
        IconKind::Settings => view! { <Settings /> }.into_any(),
        IconKind::Star => view! { <Star /> }.into_any(),
        IconKind::SlidersHorizontal => view! { <SlidersHorizontal /> }.into_any(),
        IconKind::SlidersVertical => view! { <SlidersVertical /> }.into_any(),
        IconKind::SquareStackFlipped => view! { <SquareStack /> }.into_any(),
        IconKind::Trash => view! { <Trash /> }.into_any(),
        IconKind::TriangleAlert => view! { <TriangleAlert /> }.into_any(),
        IconKind::Unlink => view! { <Unlink /> }.into_any(),
        IconKind::Upload => view! { <Upload /> }.into_any(),
        IconKind::Usb => view! { <Usb /> }.into_any(),
        IconKind::WavesHorizontal => view! { <WavesHorizontal /> }.into_any(),
        IconKind::Close => view! { <X /> }.into_any(),
    };
    view! { {icon} }
        .attr("class", class)
        .attr("aria-hidden", "true")
}

const fn flip_class(kind: IconKind) -> &'static str {
    match kind {
        IconKind::SquareStackFlipped => "-scale-y-100",
        _ => "",
    }
}
