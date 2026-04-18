use glam::Vec3;

// ---------------------------------------------------------------------------
// Enderman
// ---------------------------------------------------------------------------

/// Enderman behavioral state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndermanState {
    /// Passive — wandering, not aggravated.
    Idle,
    /// The player looked at the Enderman — it is now hostile.
    Aggravated,
    /// Currently teleporting away (e.g. hit by a projectile or taking damage).
    Teleporting,
}

/// Cone half-angle (in radians) for the "looking at" check.
/// Roughly 5 degrees — tight enough to require deliberate aim.
const LOOK_CONE_HALF_ANGLE: f32 = 0.0872665; // ~5 deg

/// Maximum distance at which looking at an Enderman triggers aggro.
const ENDERMAN_LOOK_RANGE: f32 = 64.0;

/// Maximum teleport offset per axis (blocks).
const TELEPORT_RANGE: f32 = 32.0;

/// Returns `true` if the player is looking at the Enderman.
///
/// * `player_pos` — the player's eye position.
/// * `look_dir` — the player's normalised look direction.
/// * `enderman_pos` — the Enderman's position (centre-mass).
///
/// The check uses a dot-product cone test: the angle between `look_dir` and the
/// vector from the player to the Enderman must be within [`LOOK_CONE_HALF_ANGLE`],
/// and the distance must be within [`ENDERMAN_LOOK_RANGE`].
pub fn is_player_looking_at(player_pos: Vec3, look_dir: Vec3, enderman_pos: Vec3) -> bool {
    let to_enderman = enderman_pos - player_pos;
    let dist_sq = to_enderman.length_squared();

    if dist_sq < f32::EPSILON || dist_sq > ENDERMAN_LOOK_RANGE * ENDERMAN_LOOK_RANGE {
        return false;
    }

    let dir_to_enderman = to_enderman.normalize();
    let look_norm = if look_dir.length_squared() < f32::EPSILON {
        return false;
    } else {
        look_dir.normalize()
    };

    let cos_angle = look_norm.dot(dir_to_enderman);
    cos_angle >= LOOK_CONE_HALF_ANGLE.cos()
}

/// Compute a deterministic teleport destination.
///
/// Uses a simple LCG-style hash on `seed` and `attempt` to produce a
/// pseudo-random offset in the range `[-TELEPORT_RANGE, +TELEPORT_RANGE]`
/// per axis, keeping Y non-negative.
pub fn teleport_away(pos: Vec3, seed: u64, attempt: u32) -> Vec3 {
    let hash = |n: u64| -> f32 {
        // Fowler-Noll-Vo style mixing to get a float in -1..1
        let h = n.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let bits = ((h >> 33) ^ h) as i64;
        (bits as f64 / i64::MAX as f64) as f32
    };

    let base = seed.wrapping_add(attempt as u64);
    let dx = hash(base) * TELEPORT_RANGE;
    let dy = hash(base.wrapping_add(1)).abs() * TELEPORT_RANGE * 0.5; // keep Y offset modest
    let dz = hash(base.wrapping_add(2)) * TELEPORT_RANGE;

    Vec3::new(pos.x + dx, (pos.y + dy).max(0.0), pos.z + dz)
}

// ---------------------------------------------------------------------------
// Slime
// ---------------------------------------------------------------------------

/// Returns the health of a slime of the given `size` (1 = tiny, 2 = small,
/// 4 = big in vanilla Minecraft). Health = size * size.
pub fn slime_health(size: u8) -> f32 {
    (size as f32) * (size as f32)
}

/// Returns the melee damage dealt by a slime of the given `size`.
/// Tiny slimes (size 1) deal 0 damage.
pub fn slime_damage(size: u8) -> f32 {
    if size <= 1 {
        0.0
    } else {
        size as f32
    }
}

/// When a slime dies, it splits into 2-4 smaller slimes (if size > 1).
///
/// Returns a list of `(position, new_size)` pairs. The positions are offset
/// deterministically from `pos` using `seed`. If `size <= 1` the slime is too
/// small to split and the returned vec is empty.
pub fn split_on_death(pos: Vec3, size: u8, seed: u64) -> Vec<(Vec3, u8)> {
    if size <= 1 {
        return Vec::new();
    }

    let child_size = size / 2;
    // Number of children: 2 + (seed % 3) gives 2, 3, or 4.
    let count = 2 + (seed % 3) as usize;

    (0..count)
        .map(|i| {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let offset = Vec3::new(angle.cos() * 0.5, 0.0, angle.sin() * 0.5);
            (pos + offset, child_size)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Witch
// ---------------------------------------------------------------------------

/// Potion types a Witch can throw or drink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PotionType {
    Healing,
    FireResistance,
    Swiftness,
    Harming,
    Poison,
    Weakness,
}

/// Action chosen by the Witch AI each tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WitchAction {
    /// Drink a potion (self-buff / healing).
    DrinkPotion(PotionType),
    /// Throw a potion at the player.
    ThrowPotion(PotionType),
    /// Walk toward the player to get in throwing range.
    Approach,
    /// Do nothing this tick.
    Idle,
}

/// Witch potion-throwing range (blocks).
const WITCH_THROW_RANGE: f32 = 10.0;

/// Health fraction below which the Witch prioritises self-healing.
const WITCH_HEAL_THRESHOLD: f32 = 0.5;

/// Choose the Witch's action for this tick.
///
/// Priority order (highest first):
/// 1. If on fire  -> drink fire resistance.
/// 2. If health < 50% of max -> drink healing potion.
/// 3. If player within throw range -> throw offensive potion.
/// 4. If player beyond throw range -> approach.
/// 5. Otherwise idle.
///
/// Offensive potion choice:
/// - If `distance <= 3.0` -> throw Poison (lingering area denial).
/// - Otherwise -> throw Harming (direct damage).
pub fn choose_witch_action(
    distance: f32,
    health: f32,
    max_health: f32,
    on_fire: bool,
) -> WitchAction {
    // 1. Fire resistance takes top priority.
    if on_fire {
        return WitchAction::DrinkPotion(PotionType::FireResistance);
    }

    // 2. Self-heal when low.
    if max_health > 0.0 && (health / max_health) < WITCH_HEAL_THRESHOLD {
        return WitchAction::DrinkPotion(PotionType::Healing);
    }

    // 3. Offensive potion if in range.
    if distance <= WITCH_THROW_RANGE {
        let potion = if distance <= 3.0 {
            PotionType::Poison
        } else {
            PotionType::Harming
        };
        return WitchAction::ThrowPotion(potion);
    }

    // 4. Close the gap.
    if distance > WITCH_THROW_RANGE {
        return WitchAction::Approach;
    }

    WitchAction::Idle
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Enderman: look detection -------------------------------------------

    #[test]
    fn player_looking_directly_at_enderman_triggers_detection() {
        let player_pos = Vec3::new(0.0, 1.6, 0.0);
        let enderman_pos = Vec3::new(10.0, 1.6, 0.0);
        let look_dir = Vec3::new(1.0, 0.0, 0.0); // looking straight at enderman

        assert!(is_player_looking_at(player_pos, look_dir, enderman_pos));
    }

    #[test]
    fn player_looking_away_does_not_trigger() {
        let player_pos = Vec3::new(0.0, 1.6, 0.0);
        let enderman_pos = Vec3::new(10.0, 1.6, 0.0);
        let look_dir = Vec3::new(-1.0, 0.0, 0.0); // looking opposite direction

        assert!(!is_player_looking_at(player_pos, look_dir, enderman_pos));
    }

    #[test]
    fn player_looking_perpendicular_does_not_trigger() {
        let player_pos = Vec3::ZERO;
        let enderman_pos = Vec3::new(10.0, 0.0, 0.0);
        let look_dir = Vec3::new(0.0, 0.0, 1.0); // 90 degrees off

        assert!(!is_player_looking_at(player_pos, look_dir, enderman_pos));
    }

    #[test]
    fn enderman_too_far_away_not_detected() {
        let player_pos = Vec3::ZERO;
        let enderman_pos = Vec3::new(100.0, 0.0, 0.0); // beyond 64 block range
        let look_dir = Vec3::new(1.0, 0.0, 0.0);

        assert!(!is_player_looking_at(player_pos, look_dir, enderman_pos));
    }

    #[test]
    fn enderman_at_same_position_not_detected() {
        let pos = Vec3::new(5.0, 5.0, 5.0);
        let look_dir = Vec3::new(1.0, 0.0, 0.0);

        assert!(!is_player_looking_at(pos, look_dir, pos));
    }

    #[test]
    fn zero_look_direction_not_detected() {
        let player_pos = Vec3::ZERO;
        let enderman_pos = Vec3::new(10.0, 0.0, 0.0);
        let look_dir = Vec3::ZERO;

        assert!(!is_player_looking_at(player_pos, look_dir, enderman_pos));
    }

    #[test]
    fn near_edge_of_cone_detects_correctly() {
        let player_pos = Vec3::ZERO;
        // Place enderman slightly off-axis, within ~5 degrees
        let angle: f32 = 0.04; // ~2.3 degrees (well within 5-deg cone)
        let enderman_pos = Vec3::new(10.0, 10.0 * angle.sin(), 0.0);
        let look_dir = Vec3::new(1.0, 0.0, 0.0);

        assert!(is_player_looking_at(player_pos, look_dir, enderman_pos));
    }

    // -- Enderman: teleportation --------------------------------------------

    #[test]
    fn teleport_away_produces_different_position() {
        let pos = Vec3::new(100.0, 65.0, 200.0);
        let result = teleport_away(pos, 42, 0);

        assert_ne!(pos, result, "teleported position should differ from origin");
    }

    #[test]
    fn teleport_away_y_is_non_negative() {
        // Start near Y=0 to test the clamping
        let pos = Vec3::new(0.0, 1.0, 0.0);
        for attempt in 0..20 {
            let result = teleport_away(pos, 123, attempt);
            assert!(
                result.y >= 0.0,
                "teleport Y should be >= 0, got {} at attempt {}",
                result.y,
                attempt
            );
        }
    }

    #[test]
    fn teleport_distance_is_within_range() {
        let pos = Vec3::new(50.0, 65.0, 50.0);
        for attempt in 0..20 {
            let result = teleport_away(pos, 999, attempt);
            let dx = (result.x - pos.x).abs();
            let dz = (result.z - pos.z).abs();
            assert!(
                dx <= TELEPORT_RANGE,
                "X offset {dx} exceeds {TELEPORT_RANGE}"
            );
            assert!(
                dz <= TELEPORT_RANGE,
                "Z offset {dz} exceeds {TELEPORT_RANGE}"
            );
        }
    }

    #[test]
    fn teleport_different_seeds_produce_different_results() {
        let pos = Vec3::new(0.0, 64.0, 0.0);
        let a = teleport_away(pos, 1, 0);
        let b = teleport_away(pos, 2, 0);

        assert_ne!(a, b, "different seeds should produce different positions");
    }

    #[test]
    fn teleport_different_attempts_produce_different_results() {
        let pos = Vec3::new(0.0, 64.0, 0.0);
        let a = teleport_away(pos, 42, 0);
        let b = teleport_away(pos, 42, 1);

        assert_ne!(a, b, "different attempts should produce different positions");
    }

    // -- Slime: health and damage -------------------------------------------

    #[test]
    fn slime_health_scales_with_size_squared() {
        assert!((slime_health(1) - 1.0).abs() < f32::EPSILON);
        assert!((slime_health(2) - 4.0).abs() < f32::EPSILON);
        assert!((slime_health(4) - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tiny_slime_deals_zero_damage() {
        assert!((slime_damage(1)).abs() < f32::EPSILON);
    }

    #[test]
    fn larger_slimes_deal_damage_equal_to_size() {
        assert!((slime_damage(2) - 2.0).abs() < f32::EPSILON);
        assert!((slime_damage(4) - 4.0).abs() < f32::EPSILON);
    }

    // -- Slime: splitting ---------------------------------------------------

    #[test]
    fn tiny_slime_does_not_split() {
        let children = split_on_death(Vec3::ZERO, 1, 0);
        assert!(children.is_empty());
    }

    #[test]
    fn small_slime_splits_into_tiny() {
        let children = split_on_death(Vec3::ZERO, 2, 0);
        assert!(
            (2..=4).contains(&children.len()),
            "expected 2-4 children, got {}",
            children.len()
        );
        for (_pos, size) in &children {
            assert_eq!(*size, 1, "children of size-2 slime should be size 1");
        }
    }

    #[test]
    fn big_slime_splits_into_small() {
        let children = split_on_death(Vec3::ZERO, 4, 42);
        assert!(
            (2..=4).contains(&children.len()),
            "expected 2-4 children, got {}",
            children.len()
        );
        for (_pos, size) in &children {
            assert_eq!(*size, 2, "children of size-4 slime should be size 2");
        }
    }

    #[test]
    fn split_children_are_offset_from_parent() {
        let parent_pos = Vec3::new(10.0, 65.0, 20.0);
        let children = split_on_death(parent_pos, 4, 7);

        for (child_pos, _) in &children {
            assert_ne!(
                *child_pos, parent_pos,
                "child should be offset from parent"
            );
            let dist = (*child_pos - parent_pos).length();
            assert!(
                dist < 2.0,
                "child should be close to parent, got {dist}"
            );
        }
    }

    #[test]
    fn split_count_varies_with_seed() {
        let counts: Vec<usize> = (0..10)
            .map(|s| split_on_death(Vec3::ZERO, 4, s).len())
            .collect();
        // At least two distinct counts among 10 different seeds
        let unique: std::collections::HashSet<_> = counts.iter().collect();
        assert!(
            unique.len() > 1,
            "expected varying child counts, got {counts:?}"
        );
    }

    // -- Witch: action priorities -------------------------------------------

    #[test]
    fn witch_drinks_fire_resistance_when_on_fire() {
        let action = choose_witch_action(5.0, 20.0, 26.0, true);
        assert_eq!(action, WitchAction::DrinkPotion(PotionType::FireResistance));
    }

    #[test]
    fn fire_resistance_takes_priority_over_healing() {
        // Both on fire AND low health: fire resistance wins.
        let action = choose_witch_action(5.0, 5.0, 26.0, true);
        assert_eq!(action, WitchAction::DrinkPotion(PotionType::FireResistance));
    }

    #[test]
    fn witch_heals_when_low_health() {
        let action = choose_witch_action(5.0, 10.0, 26.0, false);
        assert_eq!(action, WitchAction::DrinkPotion(PotionType::Healing));
    }

    #[test]
    fn witch_throws_harming_at_medium_range() {
        let action = choose_witch_action(7.0, 26.0, 26.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Harming));
    }

    #[test]
    fn witch_throws_poison_at_close_range() {
        let action = choose_witch_action(2.0, 26.0, 26.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Poison));
    }

    #[test]
    fn witch_approaches_when_out_of_range() {
        let action = choose_witch_action(20.0, 26.0, 26.0, false);
        assert_eq!(action, WitchAction::Approach);
    }

    #[test]
    fn witch_throws_at_boundary_of_range() {
        // Exactly at throw range boundary (10.0)
        let action = choose_witch_action(10.0, 26.0, 26.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Harming));
    }

    #[test]
    fn witch_poison_boundary_at_three_blocks() {
        // Exactly at 3.0 -> Poison
        let action = choose_witch_action(3.0, 26.0, 26.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Poison));
    }

    #[test]
    fn witch_harming_just_beyond_three_blocks() {
        let action = choose_witch_action(3.1, 26.0, 26.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Harming));
    }

    // -- EndermanState enum -------------------------------------------------

    #[test]
    fn enderman_state_variants_are_distinct() {
        assert_ne!(EndermanState::Idle, EndermanState::Aggravated);
        assert_ne!(EndermanState::Aggravated, EndermanState::Teleporting);
        assert_ne!(EndermanState::Idle, EndermanState::Teleporting);
    }
}
