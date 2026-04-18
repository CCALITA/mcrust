//! Farming and crop growth system.
//!
//! Handles crop planting, growth ticking, hydration, and harvesting
//! for all Minecraft crop types.

use mc_core::{BlockId, ItemId};

/// All supported crop types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CropType {
    Wheat,
    Carrot,
    Potato,
    Beetroot,
    MelonStem,
    PumpkinStem,
    SugarCane,
    Cactus,
}

/// State of a single crop instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropState {
    pub crop: CropType,
    pub growth_stage: u8,
    pub max_stage: u8,
}

impl CropType {
    /// Maximum growth stage for each crop type.
    pub fn max_growth_stage(&self) -> u8 {
        match self {
            CropType::Wheat => 7,
            CropType::Carrot => 7,
            CropType::Potato => 7,
            CropType::Beetroot => 3,
            CropType::MelonStem => 7,
            CropType::PumpkinStem => 7,
            CropType::SugarCane => 3,
            CropType::Cactus => 3,
        }
    }
}

impl CropState {
    /// Create a new crop at growth stage 0.
    pub fn new(crop: CropType) -> Self {
        Self {
            crop,
            growth_stage: 0,
            max_stage: crop.max_growth_stage(),
        }
    }

    /// Whether this crop has reached full maturity.
    pub fn is_mature(&self) -> bool {
        self.growth_stage >= self.max_stage
    }
}

/// Base growth probability per random tick: 1 in 25 (4%).
const BASE_GROW_CHANCE: f32 = 1.0 / 25.0;

/// Additional growth probability when the crop is hydrated.
const HYDRATION_BONUS: f32 = 1.0 / 25.0;

/// Minimum light level required for crop growth.
const MIN_LIGHT_LEVEL: u8 = 9;

/// Attempt to advance a crop by one growth stage.
///
/// Growth requires `light_level >= 9`. The base chance is 1/25 per random
/// tick, doubled when the farmland is hydrated. `random_val` should be a
/// uniform value in `0.0..1.0`.
///
/// Returns `true` if the crop grew this tick.
pub fn tick_crop(state: &mut CropState, hydrated: bool, light_level: u8, random_val: f32) -> bool {
    if state.is_mature() {
        return false;
    }

    if light_level < MIN_LIGHT_LEVEL {
        return false;
    }

    let threshold = if hydrated {
        BASE_GROW_CHANCE + HYDRATION_BONUS
    } else {
        BASE_GROW_CHANCE
    };

    if random_val < threshold {
        state.growth_stage += 1;
        true
    } else {
        false
    }
}

/// Determine whether farmland within 4 blocks of water counts as hydrated.
pub fn is_hydrated(water_within_4: bool) -> bool {
    water_within_4
}

/// Harvest a crop, returning the item drops as `(item_id_raw, count)` pairs.
///
/// Only fully mature crops yield drops. Immature crops return nothing.
pub fn harvest(state: &CropState) -> Vec<(u16, u8)> {
    if !state.is_mature() {
        return Vec::new();
    }

    match state.crop {
        CropType::Wheat => {
            vec![
                (ItemId::WheatItem as u16, 1),
                (ItemId::WheatSeeds as u16, 1),
            ]
        }
        CropType::Carrot => {
            // Carrots drop 1-4 carrots at maturity (simplified: always 2)
            vec![(ItemId::Carrot as u16, 2)]
        }
        CropType::Potato => {
            // Potatoes drop 1-4 potatoes at maturity (simplified: always 2)
            vec![(ItemId::Potato as u16, 2)]
        }
        CropType::Beetroot => {
            vec![
                (ItemId::Beetroot as u16, 1),
                (ItemId::BeetrootSeeds as u16, 1),
            ]
        }
        CropType::MelonStem => {
            // Melon stems don't drop a block themselves; the melon grows adjacent.
            // For simplicity, yield melon seeds.
            vec![(ItemId::MelonSeeds as u16, 1)]
        }
        CropType::PumpkinStem => {
            // Same as melon stem.
            vec![(ItemId::PumpkinSeeds as u16, 1)]
        }
        CropType::SugarCane => {
            // Sugar cane drops itself (as a block item).
            vec![(BlockId::SugarCane as u16, 1)]
        }
        CropType::Cactus => {
            // Cactus drops itself.
            vec![(BlockId::Cactus as u16, 1)]
        }
    }
}

/// Check whether a crop type can be planted on the given block.
///
/// - Most crops require `Farmland`.
/// - Sugar cane requires `Sand` or `Dirt` (near water in practice).
/// - Cactus requires `Sand`.
pub fn can_plant_on(crop: CropType, block_id: u16) -> bool {
    let farmland_id = BlockId::Farmland as u16;
    let sand_id = BlockId::Sand as u16;
    let dirt_id = BlockId::Dirt as u16;

    match crop {
        CropType::Wheat
        | CropType::Carrot
        | CropType::Potato
        | CropType::Beetroot
        | CropType::MelonStem
        | CropType::PumpkinStem => block_id == farmland_id,
        CropType::SugarCane => block_id == sand_id || block_id == dirt_id,
        CropType::Cactus => block_id == sand_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- CropType::max_growth_stage -----------------------------------------

    #[test]
    fn wheat_max_stage_is_7() {
        assert_eq!(CropType::Wheat.max_growth_stage(), 7);
    }

    #[test]
    fn carrot_max_stage_is_7() {
        assert_eq!(CropType::Carrot.max_growth_stage(), 7);
    }

    #[test]
    fn beetroot_max_stage_is_3() {
        assert_eq!(CropType::Beetroot.max_growth_stage(), 3);
    }

    #[test]
    fn sugar_cane_max_stage_is_3() {
        assert_eq!(CropType::SugarCane.max_growth_stage(), 3);
    }

    #[test]
    fn cactus_max_stage_is_3() {
        assert_eq!(CropType::Cactus.max_growth_stage(), 3);
    }

    #[test]
    fn melon_stem_max_stage_is_7() {
        assert_eq!(CropType::MelonStem.max_growth_stage(), 7);
    }

    #[test]
    fn pumpkin_stem_max_stage_is_7() {
        assert_eq!(CropType::PumpkinStem.max_growth_stage(), 7);
    }

    // ---- CropState::new -----------------------------------------------------

    #[test]
    fn new_crop_starts_at_stage_zero() {
        let crop = CropState::new(CropType::Wheat);
        assert_eq!(crop.growth_stage, 0);
        assert_eq!(crop.max_stage, 7);
        assert!(!crop.is_mature());
    }

    #[test]
    fn new_beetroot_has_correct_max_stage() {
        let crop = CropState::new(CropType::Beetroot);
        assert_eq!(crop.max_stage, 3);
    }

    // ---- Growth stages (tick_crop) ------------------------------------------

    #[test]
    fn crop_grows_when_random_below_threshold() {
        let mut state = CropState::new(CropType::Wheat);
        // random_val = 0.0 is always below the 1/25 threshold
        let grew = tick_crop(&mut state, false, 15, 0.0);
        assert!(grew);
        assert_eq!(state.growth_stage, 1);
    }

    #[test]
    fn crop_does_not_grow_when_random_above_threshold() {
        let mut state = CropState::new(CropType::Wheat);
        // random_val = 0.99 is always above the threshold
        let grew = tick_crop(&mut state, false, 15, 0.99);
        assert!(!grew);
        assert_eq!(state.growth_stage, 0);
    }

    #[test]
    fn crop_does_not_grow_when_light_too_low() {
        let mut state = CropState::new(CropType::Wheat);
        let grew = tick_crop(&mut state, true, 8, 0.0);
        assert!(!grew);
        assert_eq!(state.growth_stage, 0);
    }

    #[test]
    fn crop_grows_at_minimum_light_level() {
        let mut state = CropState::new(CropType::Wheat);
        let grew = tick_crop(&mut state, false, 9, 0.0);
        assert!(grew);
        assert_eq!(state.growth_stage, 1);
    }

    #[test]
    fn mature_crop_does_not_grow_further() {
        let mut state = CropState {
            crop: CropType::Wheat,
            growth_stage: 7,
            max_stage: 7,
        };
        let grew = tick_crop(&mut state, true, 15, 0.0);
        assert!(!grew);
        assert_eq!(state.growth_stage, 7);
    }

    #[test]
    fn hydration_bonus_increases_grow_chance() {
        let mut state_dry = CropState::new(CropType::Wheat);
        let mut state_wet = CropState::new(CropType::Wheat);

        // A value between base chance (0.04) and base+bonus (0.08)
        let random_val = 0.05;
        let grew_dry = tick_crop(&mut state_dry, false, 15, random_val);
        let grew_wet = tick_crop(&mut state_wet, true, 15, random_val);

        assert!(!grew_dry, "dry crop should not grow at 0.05");
        assert!(grew_wet, "hydrated crop should grow at 0.05");
    }

    #[test]
    fn full_growth_cycle() {
        let mut state = CropState::new(CropType::Beetroot);
        // Grow through all 3 stages
        for _ in 0..3 {
            assert!(!state.is_mature());
            let grew = tick_crop(&mut state, true, 15, 0.0);
            assert!(grew);
        }
        assert!(state.is_mature());
        assert_eq!(state.growth_stage, 3);
    }

    // ---- Harvest ------------------------------------------------------------

    #[test]
    fn harvest_mature_wheat_yields_wheat_and_seeds() {
        let state = CropState {
            crop: CropType::Wheat,
            growth_stage: 7,
            max_stage: 7,
        };
        let drops = harvest(&state);
        assert_eq!(drops.len(), 2);
        assert_eq!(drops[0], (ItemId::WheatItem as u16, 1));
        assert_eq!(drops[1], (ItemId::WheatSeeds as u16, 1));
    }

    #[test]
    fn harvest_immature_wheat_yields_nothing() {
        let state = CropState {
            crop: CropType::Wheat,
            growth_stage: 3,
            max_stage: 7,
        };
        let drops = harvest(&state);
        assert!(drops.is_empty());
    }

    #[test]
    fn harvest_mature_carrot_yields_carrots() {
        let state = CropState {
            crop: CropType::Carrot,
            growth_stage: 7,
            max_stage: 7,
        };
        let drops = harvest(&state);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (ItemId::Carrot as u16, 2));
    }

    #[test]
    fn harvest_mature_potato_yields_potatoes() {
        let state = CropState {
            crop: CropType::Potato,
            growth_stage: 7,
            max_stage: 7,
        };
        let drops = harvest(&state);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (ItemId::Potato as u16, 2));
    }

    #[test]
    fn harvest_mature_beetroot_yields_beetroot_and_seeds() {
        let state = CropState {
            crop: CropType::Beetroot,
            growth_stage: 3,
            max_stage: 3,
        };
        let drops = harvest(&state);
        assert_eq!(drops.len(), 2);
        assert_eq!(drops[0], (ItemId::Beetroot as u16, 1));
        assert_eq!(drops[1], (ItemId::BeetrootSeeds as u16, 1));
    }

    #[test]
    fn harvest_immature_beetroot_yields_nothing() {
        let state = CropState {
            crop: CropType::Beetroot,
            growth_stage: 1,
            max_stage: 3,
        };
        let drops = harvest(&state);
        assert!(drops.is_empty());
    }

    #[test]
    fn harvest_sugar_cane_drops_itself() {
        let state = CropState {
            crop: CropType::SugarCane,
            growth_stage: 3,
            max_stage: 3,
        };
        let drops = harvest(&state);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (BlockId::SugarCane as u16, 1));
    }

    #[test]
    fn harvest_cactus_drops_itself() {
        let state = CropState {
            crop: CropType::Cactus,
            growth_stage: 3,
            max_stage: 3,
        };
        let drops = harvest(&state);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (BlockId::Cactus as u16, 1));
    }

    // ---- Planting conditions ------------------------------------------------

    #[test]
    fn wheat_can_plant_on_farmland() {
        assert!(can_plant_on(CropType::Wheat, BlockId::Farmland as u16));
    }

    #[test]
    fn wheat_cannot_plant_on_dirt() {
        assert!(!can_plant_on(CropType::Wheat, BlockId::Dirt as u16));
    }

    #[test]
    fn carrot_can_plant_on_farmland() {
        assert!(can_plant_on(CropType::Carrot, BlockId::Farmland as u16));
    }

    #[test]
    fn potato_can_plant_on_farmland() {
        assert!(can_plant_on(CropType::Potato, BlockId::Farmland as u16));
    }

    #[test]
    fn beetroot_can_plant_on_farmland() {
        assert!(can_plant_on(CropType::Beetroot, BlockId::Farmland as u16));
    }

    #[test]
    fn melon_stem_can_plant_on_farmland() {
        assert!(can_plant_on(CropType::MelonStem, BlockId::Farmland as u16));
    }

    #[test]
    fn pumpkin_stem_can_plant_on_farmland() {
        assert!(can_plant_on(
            CropType::PumpkinStem,
            BlockId::Farmland as u16
        ));
    }

    #[test]
    fn sugar_cane_can_plant_on_sand() {
        assert!(can_plant_on(CropType::SugarCane, BlockId::Sand as u16));
    }

    #[test]
    fn sugar_cane_can_plant_on_dirt() {
        assert!(can_plant_on(CropType::SugarCane, BlockId::Dirt as u16));
    }

    #[test]
    fn sugar_cane_cannot_plant_on_stone() {
        assert!(!can_plant_on(CropType::SugarCane, BlockId::Stone as u16));
    }

    #[test]
    fn cactus_can_plant_on_sand() {
        assert!(can_plant_on(CropType::Cactus, BlockId::Sand as u16));
    }

    #[test]
    fn cactus_cannot_plant_on_dirt() {
        assert!(!can_plant_on(CropType::Cactus, BlockId::Dirt as u16));
    }

    #[test]
    fn cactus_cannot_plant_on_farmland() {
        assert!(!can_plant_on(CropType::Cactus, BlockId::Farmland as u16));
    }

    // ---- Hydration ----------------------------------------------------------

    #[test]
    fn hydrated_when_water_nearby() {
        assert!(is_hydrated(true));
    }

    #[test]
    fn not_hydrated_when_no_water() {
        assert!(!is_hydrated(false));
    }
}
