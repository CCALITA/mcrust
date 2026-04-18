use mc_core::block::BlockId;

// ---------------------------------------------------------------------------
// Flammability helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the block can catch fire and burn.
pub fn is_flammable(block: BlockId) -> bool {
    flammability(block) > 0
}

/// Returns a flammability rating in the range 0..=300.
///
/// Higher values mean the block ignites more easily. Non-flammable blocks
/// return 0. Values are loosely inspired by vanilla Minecraft.
pub fn flammability(block: BlockId) -> u8 {
    match block {
        // Logs
        BlockId::OakLog
        | BlockId::BirchLog
        | BlockId::SpruceLog
        | BlockId::JungleLog
        | BlockId::DarkOakLog => 5,

        // Planks
        BlockId::OakPlanks
        | BlockId::BirchPlanks
        | BlockId::SprucePlanks
        | BlockId::JunglePlanks
        | BlockId::DarkOakPlanks => 20,

        // Leaves
        BlockId::OakLeaves
        | BlockId::BirchLeaves
        | BlockId::SpruceLeaves
        | BlockId::JungleLeaves
        | BlockId::DarkOakLeaves => 60,

        // Wool (all colours)
        BlockId::RedWool
        | BlockId::BlueWool
        | BlockId::GreenWool
        | BlockId::YellowWool
        | BlockId::WhiteWool
        | BlockId::BlackWool => 60,

        // Bookshelf
        BlockId::Bookshelf => 30,

        // TNT
        BlockId::TNT => 100,

        // Tall grass / flowers
        BlockId::TallGrass => 100,
        BlockId::Dandelion | BlockId::Poppy => 60,

        _ => 0,
    }
}

/// Returns the chance (0..=100) that a burning block is destroyed each fire
/// tick. Non-flammable blocks return 0.
pub fn burn_chance(block: BlockId) -> u8 {
    match block {
        // Logs — slow to burn through
        BlockId::OakLog
        | BlockId::BirchLog
        | BlockId::SpruceLog
        | BlockId::JungleLog
        | BlockId::DarkOakLog => 5,

        // Planks — moderate
        BlockId::OakPlanks
        | BlockId::BirchPlanks
        | BlockId::SprucePlanks
        | BlockId::JunglePlanks
        | BlockId::DarkOakPlanks => 20,

        // Leaves — burn fast
        BlockId::OakLeaves
        | BlockId::BirchLeaves
        | BlockId::SpruceLeaves
        | BlockId::JungleLeaves
        | BlockId::DarkOakLeaves => 60,

        // Wool
        BlockId::RedWool
        | BlockId::BlueWool
        | BlockId::GreenWool
        | BlockId::YellowWool
        | BlockId::WhiteWool
        | BlockId::BlackWool => 60,

        // Bookshelf
        BlockId::Bookshelf => 20,

        // TNT — high chance of being consumed
        BlockId::TNT => 100,

        // Vegetation
        BlockId::TallGrass => 100,
        BlockId::Dandelion | BlockId::Poppy => 60,

        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Fire state machine
// ---------------------------------------------------------------------------

/// Maximum fire age before the fire extinguishes itself.
const DEFAULT_MAX_AGE: u8 = 15;

/// Tracks the state of a fire block in the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireState {
    pub pos: (i32, i32, i32),
    pub age: u8,
    pub max_age: u8,
}

impl FireState {
    /// Create a new fire at the given position with age 0.
    pub fn new(pos: (i32, i32, i32)) -> Self {
        Self {
            pos,
            age: 0,
            max_age: DEFAULT_MAX_AGE,
        }
    }
}

/// Actions produced by a fire tick.
#[derive(Debug, Clone, PartialEq)]
pub enum FireAction {
    /// Fire continues to burn in place.
    Burning,
    /// Fire spreads to the listed neighbour positions.
    SpreadTo(Vec<(i32, i32, i32)>),
    /// Fire was extinguished (rain, age, or no fuel).
    Extinguished,
    /// The block at the given position was destroyed by fire.
    BlockDestroyed((i32, i32, i32)),
}

/// Advance a fire by one tick.
///
/// * `state`      — the current fire state (age is incremented in-place).
/// * `is_raining` — if `true`, the fire is immediately extinguished.
/// * `random`     — a pseudo-random value in `[0.0, 1.0)` used for spread
///   and destroy decisions.
///
/// Returns the [`FireAction`] that should be applied to the world.
pub fn tick_fire(state: &mut FireState, is_raining: bool, random: f32) -> FireAction {
    // Rain always extinguishes fire.
    if is_raining {
        return FireAction::Extinguished;
    }

    // Age the fire.
    state.age = state.age.saturating_add(1);

    // Fire that has exceeded its max age dies.
    if state.age >= state.max_age {
        return FireAction::Extinguished;
    }

    // Chance of destroying the block beneath the fire.
    let (x, y, z) = state.pos;
    let below = (x, y - 1, z);
    // Use the lower half of the random range for block destruction.
    if random < 0.3 {
        return FireAction::BlockDestroyed(below);
    }

    // Chance of spreading to adjacent blocks.
    if random < 0.6 {
        let mut targets = Vec::new();
        // Spread in the 6 cardinal directions.
        for &(dx, dy, dz) in &[
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            targets.push((x + dx, y + dy, z + dz));
        }
        return FireAction::SpreadTo(targets);
    }

    FireAction::Burning
}

// ---------------------------------------------------------------------------
// Entity burning
// ---------------------------------------------------------------------------

/// Tracks burning state for an entity (player, mob, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct BurningEntity {
    /// Remaining ticks the entity is on fire.
    pub burn_ticks: u32,
    /// Accumulated time since the last fire-damage pulse.
    pub fire_damage_timer: f32,
}

/// Fire-damage interval in seconds (1 HP per second in vanilla).
const FIRE_DAMAGE_INTERVAL: f32 = 1.0;

/// Damage dealt per fire-damage pulse.
const FIRE_DAMAGE: f32 = 1.0;

/// Advance an entity's burning state by `dt` seconds.
///
/// Returns `(still_burning, damage_dealt)`.
///
/// * `in_water` — if `true`, fire is extinguished immediately.
/// * `fire_resist` — if `true`, no damage is dealt and ticks count down faster.
pub fn on_fire_tick(
    burning: &mut BurningEntity,
    in_water: bool,
    fire_resist: bool,
    dt: f32,
) -> (bool, f32) {
    if in_water || burning.burn_ticks == 0 {
        burning.burn_ticks = 0;
        burning.fire_damage_timer = 0.0;
        return (false, 0.0);
    }

    // Fire-resistance potion: still on fire visually, but no damage and
    // ticks decrement twice as fast.
    if fire_resist {
        burning.burn_ticks = burning.burn_ticks.saturating_sub(2);
        return (burning.burn_ticks > 0, 0.0);
    }

    burning.burn_ticks = burning.burn_ticks.saturating_sub(1);
    burning.fire_damage_timer += dt;

    let mut damage = 0.0;
    while burning.fire_damage_timer >= FIRE_DAMAGE_INTERVAL {
        burning.fire_damage_timer -= FIRE_DAMAGE_INTERVAL;
        damage += FIRE_DAMAGE;
    }

    (burning.burn_ticks > 0, damage)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Flammability -------------------------------------------------------

    #[test]
    fn wood_blocks_are_flammable() {
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
        ];
        for block in wood_blocks {
            assert!(is_flammable(block), "{:?} should be flammable", block);
            assert!(
                flammability(block) > 0,
                "{:?} flammability should be > 0",
                block
            );
        }
    }

    #[test]
    fn leaves_are_flammable() {
        let leaves = [
            BlockId::OakLeaves,
            BlockId::BirchLeaves,
            BlockId::SpruceLeaves,
            BlockId::JungleLeaves,
            BlockId::DarkOakLeaves,
        ];
        for leaf in leaves {
            assert!(is_flammable(leaf), "{:?} should be flammable", leaf);
        }
    }

    #[test]
    fn wool_is_flammable() {
        let wools = [
            BlockId::RedWool,
            BlockId::BlueWool,
            BlockId::GreenWool,
            BlockId::YellowWool,
            BlockId::WhiteWool,
            BlockId::BlackWool,
        ];
        for wool in wools {
            assert!(is_flammable(wool), "{:?} should be flammable", wool);
        }
    }

    #[test]
    fn bookshelf_and_tnt_are_flammable() {
        assert!(is_flammable(BlockId::Bookshelf));
        assert!(is_flammable(BlockId::TNT));
    }

    #[test]
    fn stone_and_water_are_not_flammable() {
        assert!(!is_flammable(BlockId::Stone));
        assert!(!is_flammable(BlockId::Water));
        assert!(!is_flammable(BlockId::Cobblestone));
        assert!(!is_flammable(BlockId::Bedrock));
    }

    #[test]
    fn burn_chance_matches_flammability() {
        // Every flammable block should have a nonzero burn chance.
        for id in 0..BlockId::COUNT as u16 {
            let block = BlockId::from_raw(id).unwrap();
            if is_flammable(block) {
                assert!(
                    burn_chance(block) > 0,
                    "{:?} is flammable but has 0 burn_chance",
                    block
                );
            } else {
                assert_eq!(
                    burn_chance(block),
                    0,
                    "{:?} is not flammable but has nonzero burn_chance",
                    block
                );
            }
        }
    }

    // -- FireState -----------------------------------------------------------

    #[test]
    fn fire_state_defaults() {
        let fire = FireState::new((10, 64, 10));
        assert_eq!(fire.pos, (10, 64, 10));
        assert_eq!(fire.age, 0);
        assert_eq!(fire.max_age, DEFAULT_MAX_AGE);
    }

    // -- tick_fire -----------------------------------------------------------

    #[test]
    fn rain_extinguishes_fire() {
        let mut fire = FireState::new((0, 64, 0));
        let action = tick_fire(&mut fire, true, 0.5);
        assert_eq!(action, FireAction::Extinguished);
    }

    #[test]
    fn fire_extinguishes_at_max_age() {
        let mut fire = FireState::new((0, 64, 0));
        fire.age = DEFAULT_MAX_AGE - 1;
        // random = 0.99 -> no spread/destroy path
        let action = tick_fire(&mut fire, false, 0.99);
        assert_eq!(action, FireAction::Extinguished);
        assert_eq!(fire.age, DEFAULT_MAX_AGE);
    }

    #[test]
    fn fire_destroys_block_on_low_random() {
        let mut fire = FireState::new((5, 64, 5));
        let action = tick_fire(&mut fire, false, 0.1);
        assert_eq!(action, FireAction::BlockDestroyed((5, 63, 5)));
    }

    #[test]
    fn fire_spreads_on_mid_random() {
        let mut fire = FireState::new((5, 64, 5));
        let action = tick_fire(&mut fire, false, 0.4);
        if let FireAction::SpreadTo(targets) = action {
            assert_eq!(targets.len(), 6, "should spread to 6 neighbours");
        } else {
            panic!("expected SpreadTo, got {:?}", action);
        }
    }

    #[test]
    fn fire_burns_on_high_random() {
        let mut fire = FireState::new((5, 64, 5));
        let action = tick_fire(&mut fire, false, 0.8);
        assert_eq!(action, FireAction::Burning);
    }

    #[test]
    fn fire_ages_each_tick() {
        let mut fire = FireState::new((0, 0, 0));
        assert_eq!(fire.age, 0);
        tick_fire(&mut fire, false, 0.8);
        assert_eq!(fire.age, 1);
        tick_fire(&mut fire, false, 0.8);
        assert_eq!(fire.age, 2);
    }

    // -- Entity burning ------------------------------------------------------

    #[test]
    fn entity_burning_damage() {
        let mut burning = BurningEntity {
            burn_ticks: 60,
            fire_damage_timer: 0.0,
        };
        // Simulate 1.5 seconds — should deal 1.0 damage (one full interval).
        let (still, damage) = on_fire_tick(&mut burning, false, false, 1.5);
        assert!(still);
        assert!((damage - 1.0).abs() < f32::EPSILON);
        // Timer should have 0.5 remaining.
        assert!((burning.fire_damage_timer - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn water_extinguishes_entity_fire() {
        let mut burning = BurningEntity {
            burn_ticks: 100,
            fire_damage_timer: 0.5,
        };
        let (still, damage) = on_fire_tick(&mut burning, true, false, 0.5);
        assert!(!still);
        assert!((damage).abs() < f32::EPSILON);
        assert_eq!(burning.burn_ticks, 0);
    }

    #[test]
    fn fire_resist_prevents_damage() {
        let mut burning = BurningEntity {
            burn_ticks: 20,
            fire_damage_timer: 0.0,
        };
        let (still, damage) = on_fire_tick(&mut burning, false, true, 1.5);
        assert!(still);
        assert!((damage).abs() < f32::EPSILON);
        // Ticks should decrement by 2 (fire resist fast-drain).
        assert_eq!(burning.burn_ticks, 18);
    }

    #[test]
    fn entity_fire_ends_when_ticks_reach_zero() {
        let mut burning = BurningEntity {
            burn_ticks: 1,
            fire_damage_timer: 0.0,
        };
        let (still, _damage) = on_fire_tick(&mut burning, false, false, 0.1);
        assert!(!still);
        assert_eq!(burning.burn_ticks, 0);
    }

    #[test]
    fn multiple_damage_pulses_in_one_tick() {
        let mut burning = BurningEntity {
            burn_ticks: 100,
            fire_damage_timer: 0.0,
        };
        // 3.0 seconds should yield 3 damage pulses.
        let (_still, damage) = on_fire_tick(&mut burning, false, false, 3.0);
        assert!((damage - 3.0).abs() < f32::EPSILON);
    }
}
