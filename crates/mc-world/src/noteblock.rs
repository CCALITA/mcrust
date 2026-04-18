use mc_core::block::BlockId;

/// The 15 instrument types a note block can produce, determined by the
/// block beneath it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instrument {
    Harp,
    Bass,
    Snare,
    Hat,
    BellBlock,
    Flute,
    Chime,
    Guitar,
    Xylophone,
    IronXylophone,
    CowBell,
    Didgeridoo,
    Bit,
    Banjo,
    Pling,
}

/// Maximum note value (exclusive). Notes wrap at this value back to 0.
const MAX_NOTE: u8 = 25;

/// Determines the instrument a note block should use based on the block
/// directly beneath it.
///
/// Mapping follows Minecraft conventions:
/// - Wood-family blocks -> Bass
/// - Stone-family blocks -> Snare (also includes netherrack, obsidian, etc.)
/// - Sand/gravel -> Hat
/// - Glass/ice -> Hat
/// - Gold ore -> BellBlock
/// - Clay -> Flute
/// - Packed ice -> Chime
/// - Wool-family blocks -> Guitar
/// - Glowstone -> Pling
/// - End stone -> Banjo (used as bone block proxy)
/// - Soul sand -> CowBell
/// - Iron ore -> IronXylophone
/// - Emerald ore -> Bit
/// - Pumpkin -> Didgeridoo
/// - Diamond ore -> Xylophone (used as emerald block proxy)
/// - Everything else -> Harp
pub fn instrument_from_block(below: BlockId) -> Instrument {
    match below {
        // Wood family -> Bass
        BlockId::OakLog
        | BlockId::OakPlanks
        | BlockId::BirchLog
        | BlockId::BirchPlanks
        | BlockId::SpruceLog
        | BlockId::SprucePlanks
        | BlockId::JungleLog
        | BlockId::JunglePlanks
        | BlockId::DarkOakLog
        | BlockId::DarkOakPlanks
        | BlockId::CraftingTable
        | BlockId::Bookshelf
        | BlockId::NoteBlock
        | BlockId::Chest => Instrument::Bass,

        // Stone family -> Snare
        BlockId::Stone
        | BlockId::Cobblestone
        | BlockId::MossyCobblestone
        | BlockId::StoneBricks
        | BlockId::Bricks
        | BlockId::Netherrack
        | BlockId::Obsidian
        | BlockId::Bedrock
        | BlockId::Furnace
        | BlockId::Dispenser
        | BlockId::Dropper => Instrument::Snare,

        // Sand / gravel -> Hat
        BlockId::Sand | BlockId::Gravel => Instrument::Hat,

        // Glass / ice -> Hat
        BlockId::Glass | BlockId::Ice => Instrument::Hat,

        // Gold ore -> BellBlock
        BlockId::GoldOre => Instrument::BellBlock,

        // Clay -> Flute
        BlockId::Clay => Instrument::Flute,

        // Packed ice -> Chime
        BlockId::PackedIce => Instrument::Chime,

        // Wool family -> Guitar
        BlockId::RedWool
        | BlockId::BlueWool
        | BlockId::GreenWool
        | BlockId::YellowWool
        | BlockId::WhiteWool
        | BlockId::BlackWool => Instrument::Guitar,

        // Glowstone -> Pling
        BlockId::Glowstone => Instrument::Pling,

        // End stone -> Banjo (bone block proxy)
        BlockId::EndStone => Instrument::Banjo,

        // Soul sand -> CowBell
        BlockId::SoulSand => Instrument::CowBell,

        // Iron ore -> IronXylophone
        BlockId::IronOre => Instrument::IronXylophone,

        // Emerald ore -> Bit
        BlockId::EmeraldOre => Instrument::Bit,

        // Pumpkin -> Didgeridoo
        BlockId::Pumpkin => Instrument::Didgeridoo,

        // Diamond ore -> Xylophone (emerald block proxy)
        BlockId::DiamondOre => Instrument::Xylophone,

        // Default -> Harp
        _ => Instrument::Harp,
    }
}

/// State of a note block in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteBlockState {
    pub note: u8,
    pub instrument: Instrument,
}

impl NoteBlockState {
    /// Creates a new note block state with the given instrument and note 0.
    pub fn new(instrument: Instrument) -> Self {
        Self {
            note: 0,
            instrument,
        }
    }
}

/// Plays the note block, returning the instrument, note index, and
/// computed pitch multiplier.
///
/// The pitch formula is `2^((note - 12) / 12)`, yielding a multiplier
/// relative to the base pitch (F#4 at note=12). Notes range from 0
/// (F#3, pitch=0.5) to 24 (F#5, pitch=2.0).
pub fn play_note(state: &NoteBlockState) -> (Instrument, u8, f32) {
    let clamped = state.note.min(24);
    let pitch = 2.0_f32.powf((clamped as f32 - 12.0) / 12.0);
    (state.instrument, clamped, pitch)
}

/// Tunes the note block by incrementing the note value. Wraps back to 0
/// after reaching 24 (i.e., at 25). Returns the new note value.
pub fn tune(state: &NoteBlockState) -> u8 {
    (state.note + 1) % MAX_NOTE
}

/// State of a jukebox block in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JukeboxState {
    pub disc: Option<u16>,
    pub playing: bool,
}

impl JukeboxState {
    /// Creates an empty jukebox with no disc.
    pub fn new() -> Self {
        Self {
            disc: None,
            playing: false,
        }
    }
}

impl Default for JukeboxState {
    fn default() -> Self {
        Self::new()
    }
}

/// Inserts a disc into the jukebox. Returns the updated state.
///
/// If a disc is already present, the jukebox is unchanged and the
/// original state is returned.
pub fn insert_disc(state: &JukeboxState, disc_id: u16) -> JukeboxState {
    if state.disc.is_some() {
        return *state;
    }
    JukeboxState {
        disc: Some(disc_id),
        playing: true,
    }
}

/// Ejects the disc from the jukebox, returning the updated state and
/// the ejected disc id (if any).
pub fn eject_disc(state: &JukeboxState) -> (JukeboxState, Option<u16>) {
    let ejected = state.disc;
    let new_state = JukeboxState {
        disc: None,
        playing: false,
    };
    (new_state, ejected)
}

/// Returns `true` if the jukebox is currently playing a disc.
pub fn is_playing(state: &JukeboxState) -> bool {
    state.playing && state.disc.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Instrument mapping tests ---

    #[test]
    fn wood_blocks_produce_bass() {
        let wood_blocks = [
            BlockId::OakLog,
            BlockId::OakPlanks,
            BlockId::BirchLog,
            BlockId::BirchPlanks,
            BlockId::SpruceLog,
            BlockId::SprucePlanks,
            BlockId::JungleLog,
            BlockId::JunglePlanks,
            BlockId::DarkOakLog,
            BlockId::DarkOakPlanks,
            BlockId::CraftingTable,
            BlockId::Bookshelf,
        ];
        for block in wood_blocks {
            assert_eq!(
                instrument_from_block(block),
                Instrument::Bass,
                "{:?} should produce Bass",
                block,
            );
        }
    }

    #[test]
    fn stone_blocks_produce_snare() {
        let stone_blocks = [
            BlockId::Stone,
            BlockId::Cobblestone,
            BlockId::MossyCobblestone,
            BlockId::StoneBricks,
            BlockId::Bricks,
            BlockId::Netherrack,
            BlockId::Obsidian,
        ];
        for block in stone_blocks {
            assert_eq!(
                instrument_from_block(block),
                Instrument::Snare,
                "{:?} should produce Snare",
                block,
            );
        }
    }

    #[test]
    fn sand_and_glass_produce_hat() {
        assert_eq!(instrument_from_block(BlockId::Sand), Instrument::Hat);
        assert_eq!(instrument_from_block(BlockId::Gravel), Instrument::Hat);
        assert_eq!(instrument_from_block(BlockId::Glass), Instrument::Hat);
        assert_eq!(instrument_from_block(BlockId::Ice), Instrument::Hat);
    }

    #[test]
    fn gold_produces_bell_block() {
        assert_eq!(instrument_from_block(BlockId::GoldOre), Instrument::BellBlock);
    }

    #[test]
    fn special_instrument_blocks() {
        assert_eq!(instrument_from_block(BlockId::Clay), Instrument::Flute);
        assert_eq!(instrument_from_block(BlockId::PackedIce), Instrument::Chime);
        assert_eq!(instrument_from_block(BlockId::Glowstone), Instrument::Pling);
        assert_eq!(instrument_from_block(BlockId::EndStone), Instrument::Banjo);
        assert_eq!(instrument_from_block(BlockId::SoulSand), Instrument::CowBell);
        assert_eq!(instrument_from_block(BlockId::IronOre), Instrument::IronXylophone);
        assert_eq!(instrument_from_block(BlockId::EmeraldOre), Instrument::Bit);
        assert_eq!(instrument_from_block(BlockId::Pumpkin), Instrument::Didgeridoo);
        assert_eq!(instrument_from_block(BlockId::DiamondOre), Instrument::Xylophone);
    }

    #[test]
    fn wool_blocks_produce_guitar() {
        let wool_blocks = [
            BlockId::RedWool,
            BlockId::BlueWool,
            BlockId::GreenWool,
            BlockId::YellowWool,
            BlockId::WhiteWool,
            BlockId::BlackWool,
        ];
        for block in wool_blocks {
            assert_eq!(
                instrument_from_block(block),
                Instrument::Guitar,
                "{:?} should produce Guitar",
                block,
            );
        }
    }

    #[test]
    fn default_blocks_produce_harp() {
        let default_blocks = [
            BlockId::Dirt,
            BlockId::GrassBlock,
            BlockId::Air,
            BlockId::Farmland,
        ];
        for block in default_blocks {
            assert_eq!(
                instrument_from_block(block),
                Instrument::Harp,
                "{:?} should produce Harp",
                block,
            );
        }
    }

    // --- Note wrap / tune tests ---

    #[test]
    fn tune_increments_note() {
        let state = NoteBlockState::new(Instrument::Harp);
        assert_eq!(tune(&state), 1);
    }

    #[test]
    fn tune_wraps_at_25() {
        let state = NoteBlockState {
            note: 24,
            instrument: Instrument::Harp,
        };
        assert_eq!(tune(&state), 0);
    }

    #[test]
    fn tune_wraps_from_zero_through_full_cycle() {
        let mut note = 0u8;
        for expected in 1..=24 {
            let state = NoteBlockState {
                note,
                instrument: Instrument::Harp,
            };
            note = tune(&state);
            assert_eq!(note, expected);
        }
        // One more should wrap to 0
        let state = NoteBlockState {
            note,
            instrument: Instrument::Harp,
        };
        assert_eq!(tune(&state), 0);
    }

    // --- Pitch calculation tests ---

    #[test]
    fn pitch_at_note_zero() {
        let state = NoteBlockState {
            note: 0,
            instrument: Instrument::Harp,
        };
        let (_, _, pitch) = play_note(&state);
        // 2^(-12/12) = 0.5
        assert!((pitch - 0.5).abs() < 1e-6);
    }

    #[test]
    fn pitch_at_note_twelve_is_unity() {
        let state = NoteBlockState {
            note: 12,
            instrument: Instrument::Harp,
        };
        let (_, _, pitch) = play_note(&state);
        assert!((pitch - 1.0).abs() < 1e-6);
    }

    #[test]
    fn pitch_at_note_twenty_four() {
        let state = NoteBlockState {
            note: 24,
            instrument: Instrument::Harp,
        };
        let (_, _, pitch) = play_note(&state);
        // 2^(12/12) = 2.0
        assert!((pitch - 2.0).abs() < 1e-6);
    }

    #[test]
    fn play_note_returns_correct_instrument_and_note() {
        let state = NoteBlockState {
            note: 7,
            instrument: Instrument::Guitar,
        };
        let (instrument, note, _) = play_note(&state);
        assert_eq!(instrument, Instrument::Guitar);
        assert_eq!(note, 7);
    }

    #[test]
    fn pitch_is_monotonically_increasing() {
        let pitches: Vec<f32> = (0..=24)
            .map(|n| {
                let state = NoteBlockState {
                    note: n,
                    instrument: Instrument::Harp,
                };
                let (_, _, pitch) = play_note(&state);
                pitch
            })
            .collect();
        for window in pitches.windows(2) {
            assert!(
                window[1] > window[0],
                "pitches should be monotonically increasing"
            );
        }
    }

    // --- Jukebox tests ---

    #[test]
    fn jukebox_starts_empty() {
        let jukebox = JukeboxState::new();
        assert_eq!(jukebox.disc, None);
        assert!(!jukebox.playing);
        assert!(!is_playing(&jukebox));
    }

    #[test]
    fn jukebox_insert_disc() {
        let jukebox = JukeboxState::new();
        let updated = insert_disc(&jukebox, 2256);
        assert_eq!(updated.disc, Some(2256));
        assert!(updated.playing);
        assert!(is_playing(&updated));
    }

    #[test]
    fn jukebox_insert_disc_when_occupied_is_noop() {
        let jukebox = JukeboxState::new();
        let with_disc = insert_disc(&jukebox, 2256);
        let still_same = insert_disc(&with_disc, 2257);
        assert_eq!(still_same.disc, Some(2256));
        assert!(still_same.playing);
    }

    #[test]
    fn jukebox_eject_disc() {
        let jukebox = JukeboxState::new();
        let with_disc = insert_disc(&jukebox, 2256);
        let (ejected_state, ejected_disc) = eject_disc(&with_disc);
        assert_eq!(ejected_disc, Some(2256));
        assert_eq!(ejected_state.disc, None);
        assert!(!ejected_state.playing);
        assert!(!is_playing(&ejected_state));
    }

    #[test]
    fn jukebox_eject_when_empty() {
        let jukebox = JukeboxState::new();
        let (ejected_state, ejected_disc) = eject_disc(&jukebox);
        assert_eq!(ejected_disc, None);
        assert_eq!(ejected_state.disc, None);
        assert!(!ejected_state.playing);
    }

    #[test]
    fn jukebox_default_is_empty() {
        let jukebox = JukeboxState::default();
        assert_eq!(jukebox.disc, None);
        assert!(!jukebox.playing);
    }
}
