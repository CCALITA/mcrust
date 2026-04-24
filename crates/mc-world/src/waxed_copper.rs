//! Waxed copper block variants with oxidation, waxing, and lightning mechanics.
//!
//! Extends the base copper system with all copper block variants (stairs, slabs,
//! doors, etc.) and provides mutation-based state transitions for game tick updates.

/// Maximum oxidation level a copper block can reach.
pub const MAX_OXIDATION: u8 = 3;

/// All copper block shape variants available in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CopperVariant {
    Block,
    Cut,
    Stairs,
    Slab,
    Grate,
    Door,
    Trapdoor,
    Bulb,
    Chiseled,
}

impl CopperVariant {
    /// Returns the display name of this copper variant.
    pub fn name(&self) -> &'static str {
        match self {
            CopperVariant::Block => "block",
            CopperVariant::Cut => "cut",
            CopperVariant::Stairs => "stairs",
            CopperVariant::Slab => "slab",
            CopperVariant::Grate => "grate",
            CopperVariant::Door => "door",
            CopperVariant::Trapdoor => "trapdoor",
            CopperVariant::Bulb => "bulb",
            CopperVariant::Chiseled => "chiseled",
        }
    }
}

/// Full state of a waxed copper block, including variant, oxidation level, and wax coating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaxedCopperState {
    pub variant: CopperVariant,
    pub oxidation: u8,
    pub waxed: bool,
}

impl WaxedCopperState {
    /// Create a new unwaxed copper block at oxidation level 0.
    pub fn new(variant: CopperVariant) -> Self {
        Self {
            variant,
            oxidation: 0,
            waxed: false,
        }
    }
}

/// Apply honeycomb to wax a copper block, preventing further oxidation.
///
/// Returns `true` if wax was applied, `false` if the block was already waxed.
pub fn apply_honeycomb(state: &mut WaxedCopperState) -> bool {
    if state.waxed {
        return false;
    }
    state.waxed = true;
    true
}

/// Scrape a copper block with an axe.
///
/// - If waxed: removes the wax coating (oxidation unchanged).
/// - If unwaxed and oxidation > 0: decreases oxidation by one level.
/// - If unwaxed and oxidation == 0: no effect.
///
/// Returns `true` if the block state changed.
pub fn scrape_with_axe(state: &mut WaxedCopperState) -> bool {
    if state.waxed {
        state.waxed = false;
        return true;
    }
    if state.oxidation > 0 {
        state.oxidation -= 1;
        return true;
    }
    false
}

/// Attempt to advance oxidation during a random tick.
///
/// Unwaxed blocks have a 1-in-64 chance of gaining one oxidation level per tick,
/// up to [`MAX_OXIDATION`]. Waxed blocks and fully oxidized blocks are unaffected.
///
/// `seed` is used as a simple deterministic random source (the low bits are tested).
///
/// Returns `true` if oxidation advanced.
pub fn random_tick_oxidation(state: &mut WaxedCopperState, seed: u64) -> bool {
    if state.waxed || state.oxidation >= MAX_OXIDATION {
        return false;
    }
    // 1/64 chance: check if the lowest 6 bits are all zero
    if seed % 64 == 0 {
        state.oxidation += 1;
        return true;
    }
    false
}

/// Lightning strike instantly removes all oxidation from a copper block.
pub fn lightning_deoxidize(state: &mut WaxedCopperState) {
    state.oxidation = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- CopperVariant ----------------------------------------------------------

    #[test]
    fn variant_name_returns_expected_strings() {
        assert_eq!(CopperVariant::Block.name(), "block");
        assert_eq!(CopperVariant::Cut.name(), "cut");
        assert_eq!(CopperVariant::Stairs.name(), "stairs");
        assert_eq!(CopperVariant::Slab.name(), "slab");
        assert_eq!(CopperVariant::Grate.name(), "grate");
        assert_eq!(CopperVariant::Door.name(), "door");
        assert_eq!(CopperVariant::Trapdoor.name(), "trapdoor");
        assert_eq!(CopperVariant::Bulb.name(), "bulb");
        assert_eq!(CopperVariant::Chiseled.name(), "chiseled");
    }

    // ---- WaxedCopperState::new --------------------------------------------------

    #[test]
    fn new_state_is_unwaxed_at_zero_oxidation() {
        let state = WaxedCopperState::new(CopperVariant::Block);
        assert_eq!(state.variant, CopperVariant::Block);
        assert_eq!(state.oxidation, 0);
        assert!(!state.waxed);
    }

    #[test]
    fn new_state_preserves_variant() {
        let state = WaxedCopperState::new(CopperVariant::Bulb);
        assert_eq!(state.variant, CopperVariant::Bulb);
    }

    // ---- apply_honeycomb --------------------------------------------------------

    #[test]
    fn apply_honeycomb_waxes_unwaxed_block() {
        let mut state = WaxedCopperState::new(CopperVariant::Stairs);
        assert!(apply_honeycomb(&mut state));
        assert!(state.waxed);
    }

    #[test]
    fn apply_honeycomb_returns_false_if_already_waxed() {
        let mut state = WaxedCopperState::new(CopperVariant::Cut);
        apply_honeycomb(&mut state);
        assert!(!apply_honeycomb(&mut state));
    }

    #[test]
    fn apply_honeycomb_preserves_oxidation() {
        let mut state = WaxedCopperState {
            variant: CopperVariant::Slab,
            oxidation: 2,
            waxed: false,
        };
        apply_honeycomb(&mut state);
        assert_eq!(state.oxidation, 2);
    }

    // ---- scrape_with_axe --------------------------------------------------------

    #[test]
    fn scrape_waxed_block_removes_wax() {
        let mut state = WaxedCopperState {
            variant: CopperVariant::Grate,
            oxidation: 1,
            waxed: true,
        };
        assert!(scrape_with_axe(&mut state));
        assert!(!state.waxed);
        assert_eq!(state.oxidation, 1);
    }

    #[test]
    fn scrape_unwaxed_reduces_oxidation() {
        let mut state = WaxedCopperState {
            variant: CopperVariant::Door,
            oxidation: 3,
            waxed: false,
        };
        assert!(scrape_with_axe(&mut state));
        assert_eq!(state.oxidation, 2);
    }

    #[test]
    fn scrape_unwaxed_at_zero_does_nothing() {
        let mut state = WaxedCopperState::new(CopperVariant::Trapdoor);
        assert!(!scrape_with_axe(&mut state));
        assert_eq!(state.oxidation, 0);
    }

    // ---- random_tick_oxidation --------------------------------------------------

    #[test]
    fn random_tick_advances_on_seed_divisible_by_64() {
        let mut state = WaxedCopperState::new(CopperVariant::Block);
        assert!(random_tick_oxidation(&mut state, 0));
        assert_eq!(state.oxidation, 1);
    }

    #[test]
    fn random_tick_does_not_advance_on_other_seeds() {
        let mut state = WaxedCopperState::new(CopperVariant::Block);
        assert!(!random_tick_oxidation(&mut state, 1));
        assert_eq!(state.oxidation, 0);
    }

    #[test]
    fn random_tick_skips_waxed_blocks() {
        let mut state = WaxedCopperState {
            variant: CopperVariant::Block,
            oxidation: 0,
            waxed: true,
        };
        assert!(!random_tick_oxidation(&mut state, 0));
        assert_eq!(state.oxidation, 0);
    }

    #[test]
    fn random_tick_caps_at_max_oxidation() {
        let mut state = WaxedCopperState {
            variant: CopperVariant::Chiseled,
            oxidation: MAX_OXIDATION,
            waxed: false,
        };
        assert!(!random_tick_oxidation(&mut state, 0));
        assert_eq!(state.oxidation, MAX_OXIDATION);
    }

    #[test]
    fn random_tick_advances_through_all_levels() {
        let mut state = WaxedCopperState::new(CopperVariant::Bulb);
        for expected in 1..=MAX_OXIDATION {
            assert!(random_tick_oxidation(&mut state, 64));
            assert_eq!(state.oxidation, expected);
        }
        assert!(!random_tick_oxidation(&mut state, 64));
    }

    // ---- lightning_deoxidize ----------------------------------------------------

    #[test]
    fn lightning_resets_oxidation_to_zero() {
        let mut state = WaxedCopperState {
            variant: CopperVariant::Stairs,
            oxidation: 3,
            waxed: false,
        };
        lightning_deoxidize(&mut state);
        assert_eq!(state.oxidation, 0);
    }

    #[test]
    fn lightning_on_already_zero_is_noop() {
        let mut state = WaxedCopperState::new(CopperVariant::Block);
        lightning_deoxidize(&mut state);
        assert_eq!(state.oxidation, 0);
    }

    #[test]
    fn lightning_preserves_wax_status() {
        let mut state = WaxedCopperState {
            variant: CopperVariant::Cut,
            oxidation: 2,
            waxed: true,
        };
        lightning_deoxidize(&mut state);
        assert_eq!(state.oxidation, 0);
        assert!(state.waxed);
    }

    // ---- full lifecycle ---------------------------------------------------------

    #[test]
    fn full_lifecycle_oxidize_wax_scrape_lightning() {
        let mut state = WaxedCopperState::new(CopperVariant::Block);

        // Oxidize to level 2
        random_tick_oxidation(&mut state, 0);
        random_tick_oxidation(&mut state, 0);
        assert_eq!(state.oxidation, 2);

        // Wax it
        assert!(apply_honeycomb(&mut state));
        assert!(state.waxed);

        // Waxed block does not oxidize
        assert!(!random_tick_oxidation(&mut state, 0));
        assert_eq!(state.oxidation, 2);

        // Scrape removes wax first
        assert!(scrape_with_axe(&mut state));
        assert!(!state.waxed);
        assert_eq!(state.oxidation, 2);

        // Scrape again reduces oxidation
        assert!(scrape_with_axe(&mut state));
        assert_eq!(state.oxidation, 1);

        // Lightning resets to zero
        lightning_deoxidize(&mut state);
        assert_eq!(state.oxidation, 0);
    }
}
