//! Sound event system and background music player.
//!
//! Manages a [`SoundQueue`] of categorized [`SoundEvent`]s with distance attenuation,
//! a dimension-aware [`MusicPlayer`], and jukebox [`DiscPlayer`] for music disc playback.

pub mod disc;
pub mod music;
pub mod sound;

pub use disc::{DiscEvent, DiscPlayer, MusicDisc, disc_from_item_id};
pub use music::{
    DIMENSION_END, DIMENSION_NETHER, DIMENSION_OVERWORLD, MusicAction, MusicPlayer, MusicState,
    MusicTrack,
};
pub use sound::{
    SoundCategory, SoundEvent, SoundId, SoundProperties, SoundQueue, volume_at_distance,
};
