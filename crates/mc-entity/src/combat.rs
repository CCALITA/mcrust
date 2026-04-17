use glam::Vec3;
use mc_core::ToolTier;

use crate::component::Health;
use crate::entity::EntityId;

// ---------------------------------------------------------------------------
// Damage types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    Melee,
    Projectile,
    Fall,
    Explosion,
    Void,
    Drowning,
    Starving,
}

// ---------------------------------------------------------------------------
// Damage event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DamageEvent {
    pub source: Option<EntityId>,
    pub target: EntityId,
    pub amount: f32,
    pub damage_type: DamageType,
    pub knockback: Vec3,
}

// ---------------------------------------------------------------------------
// Melee damage calculation
// ---------------------------------------------------------------------------

/// Calculate melee damage based on the tool tier of the held item.
///
/// Swords deal higher damage; non-sword tools deal a lower baseline that
/// scales with tier. Bare fists deal 1.0 damage.
///
/// `tool_tier` is `None` for bare-fist attacks.
pub fn calculate_melee_damage(tool_tier: Option<ToolTier>, is_sword: bool) -> f32 {
    match tool_tier {
        None | Some(ToolTier::None) => 1.0, // fist
        Some(tier) if is_sword => match tier {
            ToolTier::Wood => 4.0,
            ToolTier::Stone => 5.0,
            ToolTier::Iron => 6.0,
            ToolTier::Gold => 4.0, // gold swords are weak
            ToolTier::Diamond => 7.0,
            ToolTier::None => 1.0,
        },
        Some(tier) => match tier {
            // Non-sword tools (pickaxe, axe, shovel, hoe)
            ToolTier::Wood => 2.0,
            ToolTier::Stone => 2.5,
            ToolTier::Iron => 2.5,
            ToolTier::Gold => 2.0,
            ToolTier::Diamond => 3.0,
            ToolTier::None => 1.0,
        },
    }
}

// ---------------------------------------------------------------------------
// Knockback
// ---------------------------------------------------------------------------

/// Calculate knockback vector away from the attacker with an upward component.
///
/// The returned vector is horizontal (XZ) away from the attacker plus a
/// fixed upward Y component. The horizontal magnitude equals `base_strength`.
pub fn calculate_knockback(attacker_pos: Vec3, target_pos: Vec3, base_strength: f32) -> Vec3 {
    let diff = target_pos - attacker_pos;
    let horizontal = Vec3::new(diff.x, 0.0, diff.z);

    let horizontal_dir = if horizontal.length_squared() > f32::EPSILON {
        horizontal.normalize()
    } else {
        // Entities at same XZ position -- knock backward along +Z
        Vec3::Z
    };

    horizontal_dir * base_strength + Vec3::new(0.0, 0.4 * base_strength, 0.0)
}

// ---------------------------------------------------------------------------
// Apply damage
// ---------------------------------------------------------------------------

/// Apply `amount` damage to `health`. Returns `true` if the entity died.
pub fn apply_damage(health: &mut Health, amount: f32) -> bool {
    health.damage(amount);
    health.is_dead()
}

// ---------------------------------------------------------------------------
// Attack cooldown
// ---------------------------------------------------------------------------

/// Minimum time (seconds) between successive melee attacks (Minecraft 1.9+).
pub fn attack_cooldown() -> f32 {
    0.5
}

// ---------------------------------------------------------------------------
// Fall damage
// ---------------------------------------------------------------------------

/// Calculate fall damage.
/// The first 3 blocks of falling are free; each additional block deals 1.0 HP.
pub fn calculate_fall_damage(fall_distance: f32) -> f32 {
    (fall_distance - 3.0).max(0.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Melee damage -------------------------------------------------------

    #[test]
    fn fist_deals_one_damage() {
        assert!((calculate_melee_damage(None, false) - 1.0).abs() < f32::EPSILON);
        assert!((calculate_melee_damage(Some(ToolTier::None), false) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wood_sword_deals_four_damage() {
        let dmg = calculate_melee_damage(Some(ToolTier::Wood), true);
        assert!((dmg - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stone_sword_deals_five_damage() {
        let dmg = calculate_melee_damage(Some(ToolTier::Stone), true);
        assert!((dmg - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn iron_sword_deals_six_damage() {
        let dmg = calculate_melee_damage(Some(ToolTier::Iron), true);
        assert!((dmg - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn diamond_sword_deals_seven_damage() {
        let dmg = calculate_melee_damage(Some(ToolTier::Diamond), true);
        assert!((dmg - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn non_sword_tools_deal_two_to_three_damage() {
        let wood = calculate_melee_damage(Some(ToolTier::Wood), false);
        let diamond = calculate_melee_damage(Some(ToolTier::Diamond), false);

        assert!(
            wood >= 2.0 && wood <= 3.0,
            "wood tool damage {wood} out of range"
        );
        assert!(
            diamond >= 2.0 && diamond <= 3.0,
            "diamond tool damage {diamond} out of range"
        );
    }

    // -- Knockback ----------------------------------------------------------

    #[test]
    fn knockback_direction_is_away_from_attacker() {
        let attacker = Vec3::new(0.0, 0.0, 0.0);
        let target = Vec3::new(5.0, 0.0, 0.0);
        let kb = calculate_knockback(attacker, target, 1.0);

        // Horizontal component should be positive X (away from attacker)
        assert!(kb.x > 0.0, "knockback X should be positive, got {}", kb.x);
        // Should have upward component
        assert!(kb.y > 0.0, "knockback should have upward Y, got {}", kb.y);
    }

    #[test]
    fn knockback_zero_distance_uses_fallback_direction() {
        let pos = Vec3::new(5.0, 0.0, 5.0);
        let kb = calculate_knockback(pos, pos, 1.0);

        // Should use fallback direction (+Z), not produce NaN
        assert!(!kb.x.is_nan() && !kb.y.is_nan() && !kb.z.is_nan());
        assert!(kb.z > 0.0, "fallback knockback should be +Z, got {}", kb.z);
    }

    #[test]
    fn knockback_scales_with_strength() {
        let attacker = Vec3::ZERO;
        let target = Vec3::new(1.0, 0.0, 0.0);

        let weak = calculate_knockback(attacker, target, 0.5);
        let strong = calculate_knockback(attacker, target, 2.0);

        assert!(
            strong.length() > weak.length(),
            "stronger knockback should have greater magnitude"
        );
    }

    // -- Apply damage -------------------------------------------------------

    #[test]
    fn apply_damage_reduces_health() {
        let mut health = Health {
            current: 20.0,
            max: 20.0,
        };
        let died = apply_damage(&mut health, 5.0);
        assert!(!died);
        assert!((health.current - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_damage_returns_true_on_death() {
        let mut health = Health {
            current: 3.0,
            max: 20.0,
        };
        let died = apply_damage(&mut health, 10.0);
        assert!(died);
        assert!(health.is_dead());
    }

    #[test]
    fn apply_damage_clamps_at_zero() {
        let mut health = Health {
            current: 2.0,
            max: 20.0,
        };
        apply_damage(&mut health, 100.0);
        assert!((health.current).abs() < f32::EPSILON);
    }

    // -- Attack cooldown ----------------------------------------------------

    #[test]
    fn attack_cooldown_is_half_second() {
        assert!((attack_cooldown() - 0.5).abs() < f32::EPSILON);
    }

    // -- Fall damage --------------------------------------------------------

    #[test]
    fn fall_damage_below_threshold_is_zero() {
        assert!((calculate_fall_damage(0.0)).abs() < f32::EPSILON);
        assert!((calculate_fall_damage(1.0)).abs() < f32::EPSILON);
        assert!((calculate_fall_damage(3.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn fall_damage_above_threshold() {
        assert!((calculate_fall_damage(4.0) - 1.0).abs() < f32::EPSILON);
        assert!((calculate_fall_damage(10.0) - 7.0).abs() < f32::EPSILON);
        assert!((calculate_fall_damage(23.0) - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fall_damage_negative_distance_is_zero() {
        assert!((calculate_fall_damage(-5.0)).abs() < f32::EPSILON);
    }

    // -- DamageEvent construction -------------------------------------------

    #[test]
    fn damage_event_can_be_constructed() {
        let event = DamageEvent {
            source: Some(EntityId(1)),
            target: EntityId(2),
            amount: 6.0,
            damage_type: DamageType::Melee,
            knockback: Vec3::new(0.5, 0.4, 0.0),
        };
        assert_eq!(event.damage_type, DamageType::Melee);
        assert!((event.amount - 6.0).abs() < f32::EPSILON);
        assert_eq!(event.source, Some(EntityId(1)));
        assert_eq!(event.target, EntityId(2));
    }

    #[test]
    fn damage_event_without_source() {
        let event = DamageEvent {
            source: None,
            target: EntityId(5),
            amount: 3.0,
            damage_type: DamageType::Fall,
            knockback: Vec3::ZERO,
        };
        assert!(event.source.is_none());
        assert_eq!(event.damage_type, DamageType::Fall);
    }
}
