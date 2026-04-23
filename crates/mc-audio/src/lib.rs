//! Sound event system and background music player.
//!
//! Manages a [`SoundQueue`] of categorized [`SoundEvent`]s with distance attenuation,
//! a dimension-aware [`MusicPlayer`], and jukebox [`DiscPlayer`] for music disc playback.

pub mod ambient;
pub mod disc;
pub mod jukebox;
pub mod music;
pub mod noteblock;
pub mod sound;

pub use ambient::{AmbientConditions, AmbientSound, cave_ambience_chance, rain_volume, select_ambient};
pub use disc::{DiscEvent, DiscPlayer, MusicDisc, disc_from_item_id};
pub use music::{
    DIMENSION_END, DIMENSION_NETHER, DIMENSION_OVERWORLD, MusicAction, MusicPlayer, MusicState,
    MusicTrack,
};
pub use noteblock::{
    NoteBlockInstrument, TOTAL_NOTES, instrument_for_base_block, instrument_name, note_color,
    note_pitch,
};
pub use sound::{
    SoundCategory, SoundEvent, SoundId, SoundProperties, SoundQueue, volume_at_distance,
};
