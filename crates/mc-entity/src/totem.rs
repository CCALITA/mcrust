//! Totem of Undying death-prevention mechanic.
//!
//! When a player takes lethal damage while holding a Totem of Undying in either
//! hand, the totem activates: health is set to 1.0 HP and several beneficial
//! status effects are granted (Regeneration II, Fire Resistance, Absorption II).

// ---------------------------------------------------------------------------
// Effect type constants
// ---------------------------------------------------------------------------

/// Status-effect ID for Regeneration.
pub const REGEN_EFFECT: u8 = 10;

/// Status-effect ID for Fire Resistance.
pub const FIRE_RESIST_EFFECT: u8 = 12;

/// Status-effect ID for Absorption.
pub const ABSORPTION_EFFECT: u8 = 22;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single status effect granted by the totem upon activation.
#[derive(Debug, Clone, PartialEq)]
pub struct TotemEffect {
    pub effect_type: u8,
    pub amplifier: u8,
    pub duration_ticks: u32,
}

/// Result of checking whether a Totem of Undying prevents death.
#[derive(Debug, Clone, PartialEq)]
pub struct TotemSaveResult {
    pub saved: bool,
    pub health_restored: f32,
    pub effects: Vec<TotemEffect>,
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

/// Check whether a Totem of Undying activates to prevent death.
///
/// The totem activates when **all** of the following are true:
/// - The incoming `damage` is >= `current_health` (would be lethal).
/// - The player holds a totem in the main hand **or** off hand.
///
/// On activation the player's health is set to 1.0 HP and three effects are
/// granted:
/// - Regeneration II (amplifier 1) for 900 ticks (45 s)
/// - Fire Resistance (amplifier 0) for 800 ticks (40 s)
/// - Absorption II (amplifier 1) for 100 ticks (5 s)
pub fn check_totem_save(
    has_totem_mainhand: bool,
    has_totem_offhand: bool,
    damage: f32,
    current_health: f32,
) -> TotemSaveResult {
    let is_lethal = damage >= current_health;
    let has_totem = has_totem_mainhand || has_totem_offhand;

    if !is_lethal || !has_totem {
        return TotemSaveResult {
            saved: false,
            health_restored: 0.0,
            effects: Vec::new(),
        };
    }

    let effects = vec![
        TotemEffect {
            effect_type: REGEN_EFFECT,
            amplifier: 1,
            duration_ticks: 900,
        },
        TotemEffect {
            effect_type: FIRE_RESIST_EFFECT,
            amplifier: 0,
            duration_ticks: 800,
        },
        TotemEffect {
            effect_type: ABSORPTION_EFFECT,
            amplifier: 1,
            duration_ticks: 100,
        },
    ];

    TotemSaveResult {
        saved: true,
        health_restored: 1.0,
        effects,
    }
}

/// Returns the item ID for the Totem of Undying.
pub fn totem_item_id() -> u16 {
    4000
}

/// Returns the duration (in seconds) of the totem activation animation.
pub fn totem_animation_duration() -> f32 {
    1.5
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Saves from lethal damage -------------------------------------------

    #[test]
    fn saves_from_lethal_damage_mainhand() {
        let result = check_totem_save(true, false, 20.0, 5.0);
        assert!(result.saved);
        assert!((result.health_restored - 1.0).abs() < f32::EPSILON);
        assert_eq!(result.effects.len(), 3);
    }

    #[test]
    fn saves_from_lethal_damage_offhand() {
        let result = check_totem_save(false, true, 20.0, 5.0);
        assert!(result.saved);
        assert!((result.health_restored - 1.0).abs() < f32::EPSILON);
        assert_eq!(result.effects.len(), 3);
    }

    #[test]
    fn saves_from_exact_lethal_damage() {
        let result = check_totem_save(true, false, 10.0, 10.0);
        assert!(result.saved, "damage == health should be lethal");
        assert!((result.health_restored - 1.0).abs() < f32::EPSILON);
    }

    // -- Does not activate on non-lethal damage -----------------------------

    #[test]
    fn does_not_activate_on_non_lethal_damage() {
        let result = check_totem_save(true, true, 5.0, 20.0);
        assert!(!result.saved);
        assert!((result.health_restored).abs() < f32::EPSILON);
        assert!(result.effects.is_empty());
    }

    // -- Does not activate without totem ------------------------------------

    #[test]
    fn does_not_activate_without_totem() {
        let result = check_totem_save(false, false, 20.0, 5.0);
        assert!(!result.saved);
        assert!((result.health_restored).abs() < f32::EPSILON);
        assert!(result.effects.is_empty());
    }

    // -- Both hands ---------------------------------------------------------

    #[test]
    fn activates_with_both_hands() {
        let result = check_totem_save(true, true, 20.0, 5.0);
        assert!(result.saved);
        assert!((result.health_restored - 1.0).abs() < f32::EPSILON);
    }

    // -- Correct effects ----------------------------------------------------

    #[test]
    fn grants_regeneration_ii() {
        let result = check_totem_save(true, false, 20.0, 5.0);
        let regen = result
            .effects
            .iter()
            .find(|e| e.effect_type == REGEN_EFFECT)
            .expect("should grant Regeneration");
        assert_eq!(regen.amplifier, 1, "Regeneration should be level II (amplifier 1)");
        assert_eq!(regen.duration_ticks, 900);
    }

    #[test]
    fn grants_fire_resistance() {
        let result = check_totem_save(true, false, 20.0, 5.0);
        let fire_res = result
            .effects
            .iter()
            .find(|e| e.effect_type == FIRE_RESIST_EFFECT)
            .expect("should grant Fire Resistance");
        assert_eq!(fire_res.amplifier, 0);
        assert_eq!(fire_res.duration_ticks, 800);
    }

    #[test]
    fn grants_absorption_ii() {
        let result = check_totem_save(true, false, 20.0, 5.0);
        let absorption = result
            .effects
            .iter()
            .find(|e| e.effect_type == ABSORPTION_EFFECT)
            .expect("should grant Absorption");
        assert_eq!(absorption.amplifier, 1, "Absorption should be level II (amplifier 1)");
        assert_eq!(absorption.duration_ticks, 100);
    }

    // -- Health restored to 1.0 ---------------------------------------------

    #[test]
    fn health_set_to_one() {
        let result = check_totem_save(true, false, 100.0, 1.0);
        assert!(result.saved);
        assert!(
            (result.health_restored - 1.0).abs() < f32::EPSILON,
            "health should be restored to exactly 1.0"
        );
    }

    // -- Helper function results --------------------------------------------

    #[test]
    fn totem_item_id_is_4000() {
        assert_eq!(totem_item_id(), 4000);
    }

    #[test]
    fn totem_animation_is_1_5_seconds() {
        assert!((totem_animation_duration() - 1.5).abs() < f32::EPSILON);
    }
}
