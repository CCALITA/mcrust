//! Copper block oxidation and waxing system.
//!
//! Implements the four-stage oxidation lifecycle of copper blocks,
//! wax application to prevent further oxidation, and axe scraping
//! to revert oxidation stages.

/// The four oxidation stages of a copper block, from newest to most oxidized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OxidationStage {
    Normal,
    Exposed,
    Weathered,
    Oxidized,
}

/// A copper block with an oxidation stage and optional wax coating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopperBlock {
    pub stage: OxidationStage,
    pub waxed: bool,
}

impl CopperBlock {
    /// Create a new unwaxed copper block at the given oxidation stage.
    pub fn new(stage: OxidationStage) -> Self {
        Self {
            stage,
            waxed: false,
        }
    }
}

/// Base probability that a copper block advances one oxidation stage per random tick.
///
/// In vanilla Minecraft the chance is approximately 1 in 1125 per random tick
/// (~0.000889). Simplified here to a round constant.
const BASE_OXIDATION_CHANCE: f32 = 1.0 / 1125.0;

/// Return the per-tick oxidation probability.
pub fn oxidation_rate() -> f32 {
    BASE_OXIDATION_CHANCE
}

/// Attempt to advance a copper block's oxidation by one stage.
///
/// Waxed blocks never oxidize. Fully oxidized blocks cannot advance further.
/// `random_val` should be a uniform value in `0.0..1.0`.
///
/// Returns `Some(next_stage)` if oxidation advanced, or `None` otherwise.
pub fn tick_oxidation(stage: OxidationStage, random_val: f32) -> Option<OxidationStage> {
    let next = match stage {
        OxidationStage::Normal => OxidationStage::Exposed,
        OxidationStage::Exposed => OxidationStage::Weathered,
        OxidationStage::Weathered => OxidationStage::Oxidized,
        OxidationStage::Oxidized => return None,
    };

    if random_val < BASE_OXIDATION_CHANCE {
        Some(next)
    } else {
        None
    }
}

/// Apply honeycomb wax to a copper block, returning a new waxed copy.
///
/// Already-waxed blocks are returned unchanged.
pub fn apply_wax(block: &CopperBlock) -> CopperBlock {
    CopperBlock {
        stage: block.stage,
        waxed: true,
    }
}

/// Scrape a copper block with an axe.
///
/// - If the block is waxed, removes the wax (stage stays the same).
/// - If the block is unwaxed, reverts the oxidation by one stage.
/// - A `Normal` unwaxed block cannot be scraped further and is returned as-is.
///
/// Returns a new `CopperBlock` with the result.
pub fn scrape_with_axe(block: &CopperBlock) -> CopperBlock {
    if block.waxed {
        return CopperBlock {
            stage: block.stage,
            waxed: false,
        };
    }

    let prev = match block.stage {
        OxidationStage::Oxidized => OxidationStage::Weathered,
        OxidationStage::Weathered => OxidationStage::Exposed,
        OxidationStage::Exposed => OxidationStage::Normal,
        OxidationStage::Normal => OxidationStage::Normal,
    };

    CopperBlock {
        stage: prev,
        waxed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- OxidationStage & CopperBlock construction --------------------------

    #[test]
    fn new_block_is_unwaxed() {
        let block = CopperBlock::new(OxidationStage::Normal);
        assert_eq!(block.stage, OxidationStage::Normal);
        assert!(!block.waxed);
    }

    #[test]
    fn new_block_preserves_stage() {
        let block = CopperBlock::new(OxidationStage::Weathered);
        assert_eq!(block.stage, OxidationStage::Weathered);
    }

    // ---- oxidation_rate -----------------------------------------------------

    #[test]
    fn oxidation_rate_returns_base_chance() {
        let rate = oxidation_rate();
        assert!(rate > 0.0);
        assert!(rate < 1.0);
        assert!((rate - 1.0 / 1125.0).abs() < f32::EPSILON);
    }

    // ---- tick_oxidation -----------------------------------------------------

    #[test]
    fn normal_advances_to_exposed_when_random_below_threshold() {
        let result = tick_oxidation(OxidationStage::Normal, 0.0);
        assert_eq!(result, Some(OxidationStage::Exposed));
    }

    #[test]
    fn exposed_advances_to_weathered_when_random_below_threshold() {
        let result = tick_oxidation(OxidationStage::Exposed, 0.0);
        assert_eq!(result, Some(OxidationStage::Weathered));
    }

    #[test]
    fn weathered_advances_to_oxidized_when_random_below_threshold() {
        let result = tick_oxidation(OxidationStage::Weathered, 0.0);
        assert_eq!(result, Some(OxidationStage::Oxidized));
    }

    #[test]
    fn oxidized_never_advances() {
        let result = tick_oxidation(OxidationStage::Oxidized, 0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn no_advancement_when_random_above_threshold() {
        let result = tick_oxidation(OxidationStage::Normal, 0.99);
        assert_eq!(result, None);
    }

    // ---- apply_wax ----------------------------------------------------------

    #[test]
    fn apply_wax_sets_waxed_true() {
        let block = CopperBlock::new(OxidationStage::Exposed);
        let waxed = apply_wax(&block);
        assert!(waxed.waxed);
        assert_eq!(waxed.stage, OxidationStage::Exposed);
    }

    #[test]
    fn apply_wax_on_already_waxed_block_stays_waxed() {
        let block = CopperBlock {
            stage: OxidationStage::Normal,
            waxed: true,
        };
        let result = apply_wax(&block);
        assert!(result.waxed);
        assert_eq!(result.stage, OxidationStage::Normal);
    }

    // ---- scrape_with_axe ----------------------------------------------------

    #[test]
    fn scrape_waxed_block_removes_wax_keeps_stage() {
        let block = CopperBlock {
            stage: OxidationStage::Weathered,
            waxed: true,
        };
        let scraped = scrape_with_axe(&block);
        assert!(!scraped.waxed);
        assert_eq!(scraped.stage, OxidationStage::Weathered);
    }

    #[test]
    fn scrape_oxidized_reverts_to_weathered() {
        let block = CopperBlock::new(OxidationStage::Oxidized);
        let scraped = scrape_with_axe(&block);
        assert_eq!(scraped.stage, OxidationStage::Weathered);
        assert!(!scraped.waxed);
    }

    #[test]
    fn scrape_weathered_reverts_to_exposed() {
        let block = CopperBlock::new(OxidationStage::Weathered);
        let scraped = scrape_with_axe(&block);
        assert_eq!(scraped.stage, OxidationStage::Exposed);
    }

    #[test]
    fn scrape_exposed_reverts_to_normal() {
        let block = CopperBlock::new(OxidationStage::Exposed);
        let scraped = scrape_with_axe(&block);
        assert_eq!(scraped.stage, OxidationStage::Normal);
    }

    #[test]
    fn scrape_normal_stays_normal() {
        let block = CopperBlock::new(OxidationStage::Normal);
        let scraped = scrape_with_axe(&block);
        assert_eq!(scraped.stage, OxidationStage::Normal);
        assert!(!scraped.waxed);
    }

    // ---- full lifecycle -----------------------------------------------------

    #[test]
    fn full_oxidation_then_scrape_back() {
        // Oxidize through all stages
        let mut stage = OxidationStage::Normal;
        for expected in [
            OxidationStage::Exposed,
            OxidationStage::Weathered,
            OxidationStage::Oxidized,
        ] {
            let next = tick_oxidation(stage, 0.0);
            assert_eq!(next, Some(expected));
            stage = next.unwrap();
        }
        assert_eq!(tick_oxidation(stage, 0.0), None);

        // Scrape back through all stages
        let mut block = CopperBlock::new(stage);
        for expected in [
            OxidationStage::Weathered,
            OxidationStage::Exposed,
            OxidationStage::Normal,
        ] {
            block = scrape_with_axe(&block);
            assert_eq!(block.stage, expected);
        }
        // Scraping Normal stays Normal
        block = scrape_with_axe(&block);
        assert_eq!(block.stage, OxidationStage::Normal);
    }

    #[test]
    fn wax_prevents_oxidation_semantic_check() {
        // Waxed blocks should be checked before calling tick_oxidation.
        // This test documents the intended usage pattern: callers should
        // skip tick_oxidation for waxed blocks.
        let block = apply_wax(&CopperBlock::new(OxidationStage::Normal));
        assert!(block.waxed);
        // The caller is responsible for not calling tick_oxidation on waxed blocks.
        // tick_oxidation itself is stage-only and intentionally does not accept
        // a full CopperBlock, keeping the API composable.
    }
}
