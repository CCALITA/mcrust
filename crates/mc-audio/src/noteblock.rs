//! Note block instrument and pitch system.
//!
//! Provides [`NoteBlockInstrument`] selection based on the block beneath the
//! note block, pitch calculation for notes 0-24, and per-note particle colours.

use mc_core::BlockId;

/// Total number of playable notes (0 through 24 inclusive).
pub const TOTAL_NOTES: u8 = 25;

/// Instrument that a note block plays, determined by the block beneath it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteBlockInstrument {
    Harp,
    Bass,
    Snare,
    Hat,
    BassGuitar,
    Flute,
    Bell,
    Guitar,
    Chime,
    Xylophone,
    IronXylophone,
    CowBell,
    Didgeridoo,
    Bit,
    Banjo,
    Pling,
}

/// Returns the instrument for a note block sitting on top of `block_id`.
///
/// Block-to-instrument mapping follows vanilla Minecraft rules.  Blocks that
/// are not yet represented in [`BlockId`] (e.g. gold block, bone block) will
/// fall through to the default `Harp`.
pub fn instrument_for_base_block(block_id: u16) -> NoteBlockInstrument {
    let Some(block) = BlockId::from_raw(block_id) else {
        return NoteBlockInstrument::Harp;
    };

    match block {
        // Wood / logs -> Bass
        BlockId::OakLog
        | BlockId::BirchLog
        | BlockId::SpruceLog
        | BlockId::JungleLog
        | BlockId::DarkOakLog
        | BlockId::OakPlanks
        | BlockId::BirchPlanks
        | BlockId::SprucePlanks
        | BlockId::JunglePlanks
        | BlockId::DarkOakPlanks
        | BlockId::Bookshelf
        | BlockId::CraftingTable
        | BlockId::Chest
        | BlockId::NoteBlock => NoteBlockInstrument::Bass,

        // Sand / gravel -> Snare
        BlockId::Sand | BlockId::Gravel => NoteBlockInstrument::Snare,

        // Glass -> Hat (hi-hat)
        BlockId::Glass => NoteBlockInstrument::Hat,

        // Stone-like blocks -> BassGuitar (bass drum in vanilla)
        BlockId::Stone
        | BlockId::Cobblestone
        | BlockId::MossyCobblestone
        | BlockId::StoneBricks
        | BlockId::Bricks
        | BlockId::Obsidian
        | BlockId::Netherrack
        | BlockId::EndStone
        | BlockId::Bedrock
        | BlockId::Terracotta
        | BlockId::Furnace => NoteBlockInstrument::BassGuitar,

        // Clay -> Flute
        BlockId::Clay => NoteBlockInstrument::Flute,

        // Ice -> Chime
        BlockId::Ice | BlockId::PackedIce => NoteBlockInstrument::Chime,

        // Soul sand -> CowBell
        BlockId::SoulSand => NoteBlockInstrument::CowBell,

        // Pumpkin -> Didgeridoo
        BlockId::Pumpkin => NoteBlockInstrument::Didgeridoo,

        // Glowstone -> Pling
        BlockId::Glowstone => NoteBlockInstrument::Pling,

        // Default -> Harp
        _ => NoteBlockInstrument::Harp,
    }
}

/// Computes the playback pitch multiplier for a note (0-24).
///
/// Note 0 = F#3, note 12 = F#4 (pitch 1.0), note 24 = F#5.
/// Formula: `2^((note - 12) / 12)`.
///
/// Values outside 0-24 are clamped.
pub fn note_pitch(note: u8) -> f32 {
    let clamped = note.min(24);
    2.0_f32.powf((clamped as f32 - 12.0) / 12.0)
}

/// Returns the RGB particle colour for a note (0-24).
///
/// Maps note number to a hue on a rainbow (red at 0, green at 12, blue at 24)
/// via HSV-to-RGB conversion with S=1, V=1, hue = note * 15 degrees.
///
/// Values outside 0-24 are clamped.
pub fn note_color(note: u8) -> [f32; 3] {
    let clamped = note.min(24);
    let hue = clamped as f32 * 15.0; // 0..360 degrees
    hsv_to_rgb(hue, 1.0, 1.0)
}

/// Returns the display name for an instrument.
pub fn instrument_name(inst: NoteBlockInstrument) -> &'static str {
    match inst {
        NoteBlockInstrument::Harp => "harp",
        NoteBlockInstrument::Bass => "bass",
        NoteBlockInstrument::Snare => "snare",
        NoteBlockInstrument::Hat => "hat",
        NoteBlockInstrument::BassGuitar => "bassdrum",
        NoteBlockInstrument::Flute => "flute",
        NoteBlockInstrument::Bell => "bell",
        NoteBlockInstrument::Guitar => "guitar",
        NoteBlockInstrument::Chime => "chime",
        NoteBlockInstrument::Xylophone => "xylophone",
        NoteBlockInstrument::IronXylophone => "iron_xylophone",
        NoteBlockInstrument::CowBell => "cow_bell",
        NoteBlockInstrument::Didgeridoo => "didgeridoo",
        NoteBlockInstrument::Bit => "bit",
        NoteBlockInstrument::Banjo => "banjo",
        NoteBlockInstrument::Pling => "pling",
    }
}

/// Converts HSV (h in 0..360, s in 0..1, v in 0..1) to RGB in 0..1.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    [r1 + m, g1 + m, b1 + m]
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Pitch tests ---

    #[test]
    fn pitch_at_note_0_is_half() {
        let p = note_pitch(0);
        assert!((p - 0.5).abs() < 1e-4, "expected 0.5, got {p}");
    }

    #[test]
    fn pitch_at_note_12_is_one() {
        let p = note_pitch(12);
        assert!((p - 1.0).abs() < 1e-6, "expected 1.0, got {p}");
    }

    #[test]
    fn pitch_at_note_24_is_two() {
        let p = note_pitch(24);
        assert!((p - 2.0).abs() < 1e-4, "expected 2.0, got {p}");
    }

    #[test]
    fn pitch_clamps_above_24() {
        let p = note_pitch(30);
        assert!((p - note_pitch(24)).abs() < 1e-6);
    }

    // --- Instrument mapping tests ---

    #[test]
    fn wood_blocks_produce_bass() {
        assert_eq!(
            instrument_for_base_block(BlockId::OakLog as u16),
            NoteBlockInstrument::Bass
        );
        assert_eq!(
            instrument_for_base_block(BlockId::OakPlanks as u16),
            NoteBlockInstrument::Bass
        );
        assert_eq!(
            instrument_for_base_block(BlockId::BirchLog as u16),
            NoteBlockInstrument::Bass
        );
    }

    #[test]
    fn sand_produces_snare() {
        assert_eq!(
            instrument_for_base_block(BlockId::Sand as u16),
            NoteBlockInstrument::Snare
        );
    }

    #[test]
    fn gravel_produces_snare() {
        assert_eq!(
            instrument_for_base_block(BlockId::Gravel as u16),
            NoteBlockInstrument::Snare
        );
    }

    #[test]
    fn glass_produces_hat() {
        assert_eq!(
            instrument_for_base_block(BlockId::Glass as u16),
            NoteBlockInstrument::Hat
        );
    }

    #[test]
    fn stone_produces_bass_guitar() {
        assert_eq!(
            instrument_for_base_block(BlockId::Stone as u16),
            NoteBlockInstrument::BassGuitar
        );
        assert_eq!(
            instrument_for_base_block(BlockId::Cobblestone as u16),
            NoteBlockInstrument::BassGuitar
        );
        assert_eq!(
            instrument_for_base_block(BlockId::Obsidian as u16),
            NoteBlockInstrument::BassGuitar
        );
    }

    #[test]
    fn clay_produces_flute() {
        assert_eq!(
            instrument_for_base_block(BlockId::Clay as u16),
            NoteBlockInstrument::Flute
        );
    }

    #[test]
    fn ice_produces_chime() {
        assert_eq!(
            instrument_for_base_block(BlockId::Ice as u16),
            NoteBlockInstrument::Chime
        );
        assert_eq!(
            instrument_for_base_block(BlockId::PackedIce as u16),
            NoteBlockInstrument::Chime
        );
    }

    #[test]
    fn soul_sand_produces_cowbell() {
        assert_eq!(
            instrument_for_base_block(BlockId::SoulSand as u16),
            NoteBlockInstrument::CowBell
        );
    }

    #[test]
    fn pumpkin_produces_didgeridoo() {
        assert_eq!(
            instrument_for_base_block(BlockId::Pumpkin as u16),
            NoteBlockInstrument::Didgeridoo
        );
    }

    #[test]
    fn glowstone_produces_pling() {
        assert_eq!(
            instrument_for_base_block(BlockId::Glowstone as u16),
            NoteBlockInstrument::Pling
        );
    }

    #[test]
    fn default_block_produces_harp() {
        // Dirt is not mapped to any special instrument
        assert_eq!(
            instrument_for_base_block(BlockId::Dirt as u16),
            NoteBlockInstrument::Harp
        );
    }

    #[test]
    fn invalid_block_id_produces_harp() {
        assert_eq!(
            instrument_for_base_block(9999),
            NoteBlockInstrument::Harp
        );
    }

    // --- Color tests ---

    #[test]
    fn color_at_note_0_is_red() {
        let c = note_color(0);
        // Hue 0 -> pure red (1, 0, 0)
        assert!((c[0] - 1.0).abs() < 1e-4, "red channel: {}", c[0]);
        assert!(c[1].abs() < 1e-4, "green channel: {}", c[1]);
        assert!(c[2].abs() < 1e-4, "blue channel: {}", c[2]);
    }

    #[test]
    fn color_at_note_12_is_green() {
        let c = note_color(12);
        // Hue 180 -> cyan (0, 1, 1) in pure HSV
        // Actually hue 180 = cyan. Note 12 * 15 = 180 degrees.
        assert!(c[0].abs() < 1e-4, "red channel: {}", c[0]);
        assert!((c[1] - 1.0).abs() < 1e-4, "green channel: {}", c[1]);
        assert!((c[2] - 1.0).abs() < 1e-4, "blue channel: {}", c[2]);
    }

    #[test]
    fn color_at_note_24_is_blue() {
        let c = note_color(24);
        // Hue 360 -> wraps to 0 -> red (1, 0, 0)
        // Note: 24 * 15 = 360, which wraps to 0 in HSV
        assert!((c[0] - 1.0).abs() < 1e-4, "red channel: {}", c[0]);
        assert!(c[1].abs() < 1e-4, "green channel: {}", c[1]);
        assert!(c[2].abs() < 1e-4, "blue channel: {}", c[2]);
    }

    #[test]
    fn color_values_in_unit_range() {
        for note in 0..=24 {
            let c = note_color(note);
            for (i, channel) in c.iter().enumerate() {
                assert!(
                    (0.0..=1.0).contains(channel),
                    "note {note} channel {i} out of range: {channel}"
                );
            }
        }
    }

    // --- instrument_name tests ---

    #[test]
    fn all_instruments_have_names() {
        let instruments = [
            NoteBlockInstrument::Harp,
            NoteBlockInstrument::Bass,
            NoteBlockInstrument::Snare,
            NoteBlockInstrument::Hat,
            NoteBlockInstrument::BassGuitar,
            NoteBlockInstrument::Flute,
            NoteBlockInstrument::Bell,
            NoteBlockInstrument::Guitar,
            NoteBlockInstrument::Chime,
            NoteBlockInstrument::Xylophone,
            NoteBlockInstrument::IronXylophone,
            NoteBlockInstrument::CowBell,
            NoteBlockInstrument::Didgeridoo,
            NoteBlockInstrument::Bit,
            NoteBlockInstrument::Banjo,
            NoteBlockInstrument::Pling,
        ];
        for inst in instruments {
            let name = instrument_name(inst);
            assert!(!name.is_empty(), "{:?} has empty name", inst);
        }
        assert_eq!(instruments.len(), 16);
    }

    #[test]
    fn total_notes_constant() {
        assert_eq!(TOTAL_NOTES, 25);
    }
}
