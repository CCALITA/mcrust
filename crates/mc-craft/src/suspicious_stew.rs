// ── Suspicious Stew crafting ──────────────────────────────────────────────
//!
//! Each small flower grants a different status effect when used to craft
//! a suspicious stew.  The effect is stored as a raw `u8` protocol ID
//! so the struct stays independent of the `PotionType` enum.

use crate::SlotItem;

// ── Flower item IDs ──────────────────────────────────────────────────────
pub const FLOWER_DANDELION: SlotItem = 900;
pub const FLOWER_POPPY: SlotItem = 901;
pub const FLOWER_BLUE_ORCHID: SlotItem = 902;
pub const FLOWER_ALLIUM: SlotItem = 903;
pub const FLOWER_TULIP: SlotItem = 904;
pub const FLOWER_OXEYE_DAISY: SlotItem = 905;
pub const FLOWER_CORNFLOWER: SlotItem = 906;
pub const FLOWER_LILY_OF_VALLEY: SlotItem = 907;
pub const FLOWER_WITHER_ROSE: SlotItem = 908;
pub const FLOWER_TORCHFLOWER: SlotItem = 909;

// ── Status-effect protocol IDs (subset used by suspicious stew) ──────────
pub const EFFECT_SATURATION: u8 = 23;
pub const EFFECT_NIGHT_VISION: u8 = 16;
pub const EFFECT_FIRE_RESISTANCE: u8 = 12;
pub const EFFECT_WEAKNESS: u8 = 18;
pub const EFFECT_REGENERATION: u8 = 10;
pub const EFFECT_JUMP_BOOST: u8 = 8;
pub const EFFECT_POISON: u8 = 19;
pub const EFFECT_WITHER: u8 = 20;

// ── Duration constants (ticks, 20 ticks = 1 second) ─────────────────────
const SHORT_DURATION: u32 = 7 * 20; // 7 s  (Saturation)
const MEDIUM_DURATION: u32 = 5 * 20; // 5 s  (Poison, Weakness, Wither)
const LONG_DURATION: u32 = 8 * 20; // 8 s  (NightVision, Regen, JumpBoost, FireRes)

/// A suspicious stew carries a single status effect identified by its
/// protocol ID and a duration measured in game ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuspiciousStew {
    pub effect: u8,
    pub duration_ticks: u32,
}

/// Craft a suspicious stew from a flower.
///
/// Returns `Some(SuspiciousStew)` when `flower_id` is a recognised small
/// flower, or `None` for any other item.
#[must_use]
pub fn craft_stew(flower_id: u16) -> Option<SuspiciousStew> {
    let (effect, duration_ticks) = match flower_id {
        FLOWER_DANDELION => (EFFECT_SATURATION, SHORT_DURATION),
        FLOWER_POPPY => (EFFECT_NIGHT_VISION, LONG_DURATION),
        FLOWER_BLUE_ORCHID => (EFFECT_SATURATION, SHORT_DURATION),
        FLOWER_ALLIUM => (EFFECT_FIRE_RESISTANCE, LONG_DURATION),
        FLOWER_TULIP => (EFFECT_WEAKNESS, MEDIUM_DURATION),
        FLOWER_OXEYE_DAISY => (EFFECT_REGENERATION, LONG_DURATION),
        FLOWER_CORNFLOWER => (EFFECT_JUMP_BOOST, LONG_DURATION),
        FLOWER_LILY_OF_VALLEY => (EFFECT_POISON, MEDIUM_DURATION),
        FLOWER_WITHER_ROSE => (EFFECT_WITHER, MEDIUM_DURATION),
        FLOWER_TORCHFLOWER => (EFFECT_NIGHT_VISION, LONG_DURATION),
        _ => return None,
    };

    Some(SuspiciousStew {
        effect,
        duration_ticks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dandelion_gives_saturation() {
        let stew = craft_stew(FLOWER_DANDELION).expect("should craft");
        assert_eq!(stew.effect, EFFECT_SATURATION);
        assert_eq!(stew.duration_ticks, SHORT_DURATION);
    }

    #[test]
    fn poppy_gives_night_vision() {
        let stew = craft_stew(FLOWER_POPPY).expect("should craft");
        assert_eq!(stew.effect, EFFECT_NIGHT_VISION);
        assert_eq!(stew.duration_ticks, LONG_DURATION);
    }

    #[test]
    fn blue_orchid_gives_saturation() {
        let stew = craft_stew(FLOWER_BLUE_ORCHID).expect("should craft");
        assert_eq!(stew.effect, EFFECT_SATURATION);
        assert_eq!(stew.duration_ticks, SHORT_DURATION);
    }

    #[test]
    fn allium_gives_fire_resistance() {
        let stew = craft_stew(FLOWER_ALLIUM).expect("should craft");
        assert_eq!(stew.effect, EFFECT_FIRE_RESISTANCE);
        assert_eq!(stew.duration_ticks, LONG_DURATION);
    }

    #[test]
    fn tulip_gives_weakness() {
        let stew = craft_stew(FLOWER_TULIP).expect("should craft");
        assert_eq!(stew.effect, EFFECT_WEAKNESS);
        assert_eq!(stew.duration_ticks, MEDIUM_DURATION);
    }

    #[test]
    fn oxeye_daisy_gives_regeneration() {
        let stew = craft_stew(FLOWER_OXEYE_DAISY).expect("should craft");
        assert_eq!(stew.effect, EFFECT_REGENERATION);
        assert_eq!(stew.duration_ticks, LONG_DURATION);
    }

    #[test]
    fn cornflower_gives_jump_boost() {
        let stew = craft_stew(FLOWER_CORNFLOWER).expect("should craft");
        assert_eq!(stew.effect, EFFECT_JUMP_BOOST);
        assert_eq!(stew.duration_ticks, LONG_DURATION);
    }

    #[test]
    fn lily_of_valley_gives_poison() {
        let stew = craft_stew(FLOWER_LILY_OF_VALLEY).expect("should craft");
        assert_eq!(stew.effect, EFFECT_POISON);
        assert_eq!(stew.duration_ticks, MEDIUM_DURATION);
    }

    #[test]
    fn wither_rose_gives_wither() {
        let stew = craft_stew(FLOWER_WITHER_ROSE).expect("should craft");
        assert_eq!(stew.effect, EFFECT_WITHER);
        assert_eq!(stew.duration_ticks, MEDIUM_DURATION);
    }

    #[test]
    fn torchflower_gives_night_vision() {
        let stew = craft_stew(FLOWER_TORCHFLOWER).expect("should craft");
        assert_eq!(stew.effect, EFFECT_NIGHT_VISION);
        assert_eq!(stew.duration_ticks, LONG_DURATION);
    }

    #[test]
    fn unknown_item_returns_none() {
        assert!(craft_stew(0).is_none());
        assert!(craft_stew(9999).is_none());
    }

    #[test]
    fn stew_struct_is_copy_and_eq() {
        let a = SuspiciousStew {
            effect: EFFECT_POISON,
            duration_ticks: 100,
        };
        let b = a; // Copy
        assert_eq!(a, b);
    }
}
