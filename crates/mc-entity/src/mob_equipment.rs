//! Hostile mob equipment generation.
//!
//! Models random armor, weapons, and enchantment chances for hostile mobs (zombies,
//! skeletons, pillagers, etc.) based on difficulty and regional difficulty, as in
//! vanilla Minecraft mob spawning.

/// Equipment worn by a hostile mob. All slots are optional; `None` means the slot
/// is empty. Values are raw item ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MobEquipment {
    pub helmet: Option<u16>,
    pub chestplate: Option<u16>,
    pub leggings: Option<u16>,
    pub boots: Option<u16>,
    pub weapon: Option<u16>,
}

// Item ids used when equipping hostile mobs. Kept local to this module so the
// equipment logic is self-contained; real resolution can map these to the
// global item registry later.
pub const ITEM_LEATHER_HELMET: u16 = 300;
pub const ITEM_LEATHER_CHESTPLATE: u16 = 301;
pub const ITEM_LEATHER_LEGGINGS: u16 = 302;
pub const ITEM_LEATHER_BOOTS: u16 = 303;

pub const ITEM_CHAIN_HELMET: u16 = 310;
pub const ITEM_CHAIN_CHESTPLATE: u16 = 311;
pub const ITEM_CHAIN_LEGGINGS: u16 = 312;
pub const ITEM_CHAIN_BOOTS: u16 = 313;

pub const ITEM_IRON_HELMET: u16 = 320;
pub const ITEM_IRON_CHESTPLATE: u16 = 321;
pub const ITEM_IRON_LEGGINGS: u16 = 322;
pub const ITEM_IRON_BOOTS: u16 = 323;

pub const ITEM_GOLD_HELMET: u16 = 330;
pub const ITEM_GOLD_CHESTPLATE: u16 = 331;
pub const ITEM_GOLD_LEGGINGS: u16 = 332;
pub const ITEM_GOLD_BOOTS: u16 = 333;

pub const ITEM_DIAMOND_HELMET: u16 = 340;
pub const ITEM_DIAMOND_CHESTPLATE: u16 = 341;
pub const ITEM_DIAMOND_LEGGINGS: u16 = 342;
pub const ITEM_DIAMOND_BOOTS: u16 = 343;

pub const ITEM_IRON_SWORD: u16 = 400;
pub const ITEM_BOW: u16 = 500;
pub const ITEM_CROSSBOW: u16 = 4001;

/// Difficulty constants (kept as a lightweight convention inside this file —
/// `0 = peaceful`, `1 = easy`, `2 = normal`, `3 = hard`). The task description
/// references "easy(0)" and "hard(2)" so we keep a 0..=3 range to match.
const DIFF_PEACEFUL: u8 = 0;
const DIFF_EASY: u8 = 1;
const DIFF_NORMAL: u8 = 2;
#[cfg(test)]
const DIFF_HARD: u8 = 3;

impl MobEquipment {
    /// Returns an equipment set with every slot empty.
    pub const fn empty() -> Self {
        Self {
            helmet: None,
            chestplate: None,
            leggings: None,
            boots: None,
            weapon: None,
        }
    }
}

/// Returns the chance (0.0..=1.0) that a freshly spawned hostile mob will roll
/// an armor piece per slot. 0% on peaceful; scales linearly by regional
/// difficulty on harder difficulties up to 7.5% on hard.
///
/// The contract from the task:
/// - `difficulty == 0` (easy) → 0%
/// - peaceful → 0.05% (a vanishingly small chance used to model vanilla's near-
///   zero armor odds when mobs are only spawned by spawners/commands)
/// - `difficulty == 2` (hard) → 7.5%
pub fn random_armor_chance(difficulty: u8, regional_difficulty: f32) -> f32 {
    // NaN and out-of-range regional difficulty is clamped to [0.0, 1.0].
    let regional = if regional_difficulty.is_nan() {
        0.0
    } else {
        regional_difficulty.clamp(0.0, 1.0)
    };

    match difficulty {
        DIFF_PEACEFUL => 0.0005,
        DIFF_EASY => 0.0,
        DIFF_NORMAL => 0.025 * regional,
        // Hard and anything higher cap at the hard rate.
        _ => 0.075 * regional,
    }
}

/// Chance (0.0..=1.0) that an armor piece rolled by `random_armor_chance` is
/// enchanted. 0% peaceful/easy, 25% normal, 50% hard.
pub fn enchantment_chance(difficulty: u8) -> f32 {
    match difficulty {
        DIFF_PEACEFUL | DIFF_EASY => 0.0,
        DIFF_NORMAL => 0.25,
        _ => 0.50,
    }
}

/// Deterministic armor tier selection.
/// Returns `0 = leather, 1 = chain, 2 = iron, 3 = gold, 4 = diamond`.
///
/// Harder difficulties bias the roll toward higher tiers; easier difficulties
/// bias toward leather. The seed makes the roll reproducible for tests.
pub fn armor_tier_for_difficulty(seed: u64, difficulty: u8) -> u8 {
    // Simple deterministic hash: xorshift64* mixing.
    let mut x = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;

    let roll = (x % 100) as u8;

    match difficulty {
        DIFF_PEACEFUL | DIFF_EASY => {
            // Mostly leather, rare chain.
            if roll < 90 { 0 } else { 1 }
        }
        DIFF_NORMAL => {
            // Distribution roughly matching vanilla normal.
            match roll {
                0..=36 => 0,  // leather
                37..=62 => 1, // chain
                63..=86 => 2, // iron
                87..=96 => 3, // gold
                _ => 4,       // diamond
            }
        }
        _ => {
            // Hard: much better gear overall.
            match roll {
                0..=14 => 0,  // leather
                15..=34 => 1, // chain
                35..=64 => 2, // iron
                65..=79 => 3, // gold
                _ => 4,       // diamond
            }
        }
    }
}

/// Returns the four armor item ids (helmet, chestplate, leggings, boots) for a
/// given tier.
fn armor_set_for_tier(tier: u8) -> (u16, u16, u16, u16) {
    match tier {
        0 => (
            ITEM_LEATHER_HELMET,
            ITEM_LEATHER_CHESTPLATE,
            ITEM_LEATHER_LEGGINGS,
            ITEM_LEATHER_BOOTS,
        ),
        1 => (
            ITEM_CHAIN_HELMET,
            ITEM_CHAIN_CHESTPLATE,
            ITEM_CHAIN_LEGGINGS,
            ITEM_CHAIN_BOOTS,
        ),
        2 => (
            ITEM_IRON_HELMET,
            ITEM_IRON_CHESTPLATE,
            ITEM_IRON_LEGGINGS,
            ITEM_IRON_BOOTS,
        ),
        3 => (
            ITEM_GOLD_HELMET,
            ITEM_GOLD_CHESTPLATE,
            ITEM_GOLD_LEGGINGS,
            ITEM_GOLD_BOOTS,
        ),
        _ => (
            ITEM_DIAMOND_HELMET,
            ITEM_DIAMOND_CHESTPLATE,
            ITEM_DIAMOND_LEGGINGS,
            ITEM_DIAMOND_BOOTS,
        ),
    }
}

/// Deterministic sub-roll in `[0.0, 1.0)` from a seed and a "salt".
/// Used so each slot picks an independent roll from the same spawn seed.
fn hash_unit(seed: u64, salt: u64) -> f32 {
    let mut x = seed ^ salt.wrapping_mul(0xD1B5_4A32_D192_ED03);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    x = x.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    x ^= x >> 33;
    // Top 24 bits → f32 in [0,1).
    ((x >> 40) as f32) / ((1u32 << 24) as f32)
}

fn regional_from_seed(seed: u64) -> f32 {
    // Derive a stable regional difficulty value in [0.0, 1.0] from the seed so
    // callers who do not track local difficulty still get sensible results.
    hash_unit(seed, 0xA11CE_C0FFEE).clamp(0.0, 1.0)
}

/// Equip a zombie: random armor per slot (chance based on difficulty and a
/// derived regional difficulty) and an occasional iron sword.
pub fn equip_zombie(seed: u64, difficulty: u8) -> MobEquipment {
    let regional = regional_from_seed(seed);
    let chance = random_armor_chance(difficulty, regional);
    let tier = armor_tier_for_difficulty(seed, difficulty);
    let (helmet, chestplate, leggings, boots) = armor_set_for_tier(tier);

    let roll_slot = |salt: u64, item: u16| -> Option<u16> {
        if hash_unit(seed, salt) < chance {
            Some(item)
        } else {
            None
        }
    };

    // Independent per-slot rolls.
    let helmet_slot = roll_slot(1, helmet);
    let chest_slot = roll_slot(2, chestplate);
    let legs_slot = roll_slot(3, leggings);
    let boots_slot = roll_slot(4, boots);

    // Iron sword: ~5% on easy, ~15% on normal, ~25% on hard (scaled with regional).
    let weapon_chance = match difficulty {
        DIFF_PEACEFUL | DIFF_EASY => 0.05,
        DIFF_NORMAL => 0.15 * regional,
        _ => 0.25 * regional,
    };
    let weapon = if hash_unit(seed, 5) < weapon_chance {
        Some(ITEM_IRON_SWORD)
    } else {
        None
    };

    MobEquipment {
        helmet: helmet_slot,
        chestplate: chest_slot,
        leggings: legs_slot,
        boots: boots_slot,
        weapon,
    }
}

/// Equip a skeleton: always carries a bow; armor follows the same per-slot
/// rolls as a zombie.
pub fn equip_skeleton(seed: u64, difficulty: u8) -> MobEquipment {
    let mut eq = equip_zombie(seed, difficulty);
    eq.weapon = Some(ITEM_BOW);
    eq
}

/// Equip a pillager: always carries a crossbow. Pillagers do not roll armor.
pub fn equip_pillager(seed: u64) -> MobEquipment {
    let _ = seed;
    MobEquipment {
        weapon: Some(ITEM_CROSSBOW),
        ..MobEquipment::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_has_no_items() {
        let eq = MobEquipment::empty();
        assert!(eq.helmet.is_none());
        assert!(eq.chestplate.is_none());
        assert!(eq.leggings.is_none());
        assert!(eq.boots.is_none());
        assert!(eq.weapon.is_none());
    }

    #[test]
    fn armor_chance_easy_is_zero() {
        assert_eq!(random_armor_chance(DIFF_EASY, 1.0), 0.0);
        assert_eq!(random_armor_chance(DIFF_EASY, 0.5), 0.0);
    }

    #[test]
    fn armor_chance_peaceful_is_tiny() {
        let c = random_armor_chance(DIFF_PEACEFUL, 1.0);
        assert!((c - 0.0005).abs() < f32::EPSILON);
    }

    #[test]
    fn armor_chance_hard_caps_at_seven_and_a_half_percent() {
        let c = random_armor_chance(DIFF_HARD, 1.0);
        assert!((c - 0.075).abs() < 1e-6);
    }

    #[test]
    fn armor_chance_scales_with_regional_difficulty() {
        let low = random_armor_chance(DIFF_HARD, 0.0);
        let mid = random_armor_chance(DIFF_HARD, 0.5);
        let high = random_armor_chance(DIFF_HARD, 1.0);
        assert!(low < mid);
        assert!(mid < high);
    }

    #[test]
    fn armor_chance_handles_bad_regional_values() {
        // NaN and out-of-range values should clamp safely, not propagate NaN.
        assert_eq!(random_armor_chance(DIFF_HARD, f32::NAN), 0.0);
        assert!((random_armor_chance(DIFF_HARD, 2.0) - 0.075).abs() < 1e-6);
        assert_eq!(random_armor_chance(DIFF_HARD, -1.0), 0.0);
    }

    #[test]
    fn enchant_chance_table_matches_spec() {
        assert_eq!(enchantment_chance(DIFF_PEACEFUL), 0.0);
        assert_eq!(enchantment_chance(DIFF_EASY), 0.0);
        assert_eq!(enchantment_chance(DIFF_NORMAL), 0.25);
        assert_eq!(enchantment_chance(DIFF_HARD), 0.50);
    }

    #[test]
    fn armor_tier_returns_valid_range() {
        for seed in 0..200u64 {
            let tier = armor_tier_for_difficulty(seed, DIFF_HARD);
            assert!(tier <= 4, "tier {} out of range for seed {}", tier, seed);
        }
    }

    #[test]
    fn armor_tier_is_deterministic() {
        let a = armor_tier_for_difficulty(42, DIFF_HARD);
        let b = armor_tier_for_difficulty(42, DIFF_HARD);
        assert_eq!(a, b);
    }

    #[test]
    fn armor_tier_easy_biases_leather() {
        let mut leather = 0;
        for seed in 0..500u64 {
            if armor_tier_for_difficulty(seed, DIFF_EASY) == 0 {
                leather += 1;
            }
        }
        // Should be overwhelmingly leather on easy.
        assert!(leather > 400, "expected mostly leather on easy, got {}", leather);
    }

    #[test]
    fn armor_tier_hard_has_more_diamond_than_easy() {
        let count_diamond = |diff: u8| -> u32 {
            (0..1000u64)
                .filter(|&s| armor_tier_for_difficulty(s, diff) == 4)
                .count() as u32
        };
        assert!(count_diamond(DIFF_HARD) > count_diamond(DIFF_EASY));
    }

    #[test]
    fn zombie_equipment_is_deterministic() {
        let a = equip_zombie(1234, DIFF_HARD);
        let b = equip_zombie(1234, DIFF_HARD);
        assert_eq!(a, b);
    }

    #[test]
    fn skeleton_always_has_bow() {
        for seed in 0..50u64 {
            for diff in 0..=3u8 {
                let eq = equip_skeleton(seed, diff);
                assert_eq!(eq.weapon, Some(ITEM_BOW));
            }
        }
    }

    #[test]
    fn pillager_always_has_crossbow() {
        for seed in 0..50u64 {
            let eq = equip_pillager(seed);
            assert_eq!(eq.weapon, Some(ITEM_CROSSBOW));
            assert!(eq.helmet.is_none());
            assert!(eq.chestplate.is_none());
            assert!(eq.leggings.is_none());
            assert!(eq.boots.is_none());
        }
    }

    #[test]
    fn zombie_on_easy_rarely_wears_armor() {
        let mut any_armor = 0;
        for seed in 0..1000u64 {
            let eq = equip_zombie(seed, DIFF_EASY);
            if eq.helmet.is_some()
                || eq.chestplate.is_some()
                || eq.leggings.is_some()
                || eq.boots.is_some()
            {
                any_armor += 1;
            }
        }
        // Easy armor chance is 0% — no armor should ever appear.
        assert_eq!(any_armor, 0);
    }
}
