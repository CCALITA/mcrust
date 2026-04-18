use glam::Vec3;

// ---------------------------------------------------------------------------
// Enderman
// ---------------------------------------------------------------------------

/// Behavioral state of an Enderman.
#[derive(Debug, Clone, PartialEq)]
pub enum EndermanState {
    /// Standing around passively.
    Idle,
    /// Aggravated — chasing a target position.
    Aggravated { target_pos: Vec3 },
    /// Mid-teleport (visual / cooldown phase).
    Teleporting,
}

/// Maximum distance at which a player's gaze can aggravate an Enderman.
const ENDERMAN_LOOK_RANGE: f32 = 64.0;

/// Dot-product threshold for "looking at" detection (≈ 8° cone).
const LOOK_DOT_THRESHOLD: f32 = 0.99;

/// Height of an Enderman's head hitbox measured from its feet (top 0.5 blocks).
const HEAD_TOP_OFFSET: f32 = 0.0;
const HEAD_BOTTOM_OFFSET: f32 = 0.5;

/// Returns `true` if the player's look direction points at the Enderman's head
/// (top 0.5 blocks of its body). Uses dot-product > 0.99 and distance < 64.
pub fn is_player_looking_at(
    player_pos: Vec3,
    player_look_dir: Vec3,
    enderman_pos: Vec3,
    enderman_height: f32,
) -> bool {
    // Head occupies the top 0.5 blocks of the Enderman.
    let head_top = enderman_pos + Vec3::new(0.0, enderman_height - HEAD_TOP_OFFSET, 0.0);
    let head_bottom = enderman_pos + Vec3::new(0.0, enderman_height - HEAD_BOTTOM_OFFSET, 0.0);
    let head_center = (head_top + head_bottom) * 0.5;

    let to_head = head_center - player_pos;
    let distance = to_head.length();

    if distance > ENDERMAN_LOOK_RANGE || distance < f32::EPSILON {
        return false;
    }

    let direction_to_head = to_head / distance;
    let look_normalized = player_look_dir.normalize_or_zero();

    look_normalized.dot(direction_to_head) > LOOK_DOT_THRESHOLD
}

/// Simple hash-based pseudo-random number derived from `seed` and `attempt`.
fn pseudo_random(seed: u64, attempt: u32) -> u64 {
    let mut h = seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(attempt as u64);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// Maps a hash value to a float in `[min, max)`.
fn hash_to_range(hash: u64, min: f32, max: f32) -> f32 {
    let t = (hash & 0xFFFF_FFFF) as f32 / u32::MAX as f32; // 0.0..1.0
    min + t * (max - min)
}

/// Compute a teleport destination 16-32 blocks away from `current_pos`.
///
/// Uses a deterministic hash derived from `seed` and `attempt` so the result
/// is reproducible without requiring mutable RNG state.
pub fn teleport_away(current_pos: Vec3, seed: u64, attempt: u32) -> Vec3 {
    let h1 = pseudo_random(seed, attempt.wrapping_mul(3));
    let h2 = pseudo_random(seed, attempt.wrapping_mul(3).wrapping_add(1));
    let h3 = pseudo_random(seed, attempt.wrapping_mul(3).wrapping_add(2));

    // Distance in [16, 32].
    let distance = hash_to_range(h1, 16.0, 32.0);

    // Angle around Y axis in [0, 2*PI).
    let angle = hash_to_range(h2, 0.0, std::f32::consts::TAU);

    // Vertical offset in [-8, 8].
    let y_offset = hash_to_range(h3, -8.0, 8.0);

    let dx = distance * angle.cos();
    let dz = distance * angle.sin();

    Vec3::new(
        current_pos.x + dx,
        current_pos.y + y_offset,
        current_pos.z + dz,
    )
}

// ---------------------------------------------------------------------------
// Slime
// ---------------------------------------------------------------------------

/// Returns the health of a slime given its size.
///
/// Health = size * size (small=1, medium=4, large=16).
pub fn slime_health(size: u8) -> f32 {
    let s = size as f32;
    s * s
}

/// Returns the contact damage of a slime given its size.
///
/// Damage = size (small=1, medium=2, large=4).
pub fn slime_damage(size: u8) -> f32 {
    size as f32
}

/// Splits a slime on death into smaller slimes.
///
/// * Large (4) -> 2-4 medium (2)
/// * Medium (2) -> 2-4 small (1)
/// * Small (1) -> nothing
///
/// Each child is offset slightly from `pos` using deterministic hashing.
pub fn split_on_death(pos: Vec3, size: u8, seed: u64) -> Vec<(Vec3, u8)> {
    let new_size = match size {
        4 => 2u8,
        2 => 1u8,
        _ => return Vec::new(), // small slimes don't split
    };

    // Determine count in [2, 4] from seed.
    let count_hash = pseudo_random(seed, 0);
    let count = 2 + (count_hash % 3) as usize; // 2, 3, or 4

    (0..count)
        .map(|i| {
            let hx = pseudo_random(seed, (i as u32 + 1) * 2);
            let hz = pseudo_random(seed, (i as u32 + 1) * 2 + 1);
            let offset_x = hash_to_range(hx, -1.0, 1.0);
            let offset_z = hash_to_range(hz, -1.0, 1.0);
            let child_pos = Vec3::new(pos.x + offset_x, pos.y, pos.z + offset_z);
            (child_pos, new_size)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Witch
// ---------------------------------------------------------------------------

/// Types of potions a Witch can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionType {
    Healing,
    FireResistance,
    Poison,
    Harming,
    Slowness,
}

/// Action chosen by the Witch AI each tick.
#[derive(Debug, Clone, PartialEq)]
pub enum WitchAction {
    /// Do nothing.
    Idle,
    /// Throw a potion at the player.
    ThrowPotion(PotionType),
    /// Drink a potion (self-buff / heal).
    DrinkPotion(PotionType),
}

/// Choose the Witch's action based on current conditions.
///
/// Priority order:
/// 1. If health < 50%: drink Healing
/// 2. If on fire: drink FireResistance
/// 3. If distance < 8: throw Poison or Harming (Harming when very close)
/// 4. If distance < 16: throw Slowness
/// 5. Else: Idle
pub fn choose_witch_action(
    distance_to_player: f32,
    self_health: f32,
    max_health: f32,
    is_on_fire: bool,
) -> WitchAction {
    // Priority 1 — self-preservation
    if max_health > 0.0 && self_health / max_health < 0.5 {
        return WitchAction::DrinkPotion(PotionType::Healing);
    }

    // Priority 2 — extinguish fire
    if is_on_fire {
        return WitchAction::DrinkPotion(PotionType::FireResistance);
    }

    // Priority 3 — close-range offense
    if distance_to_player < 8.0 {
        // Very close → Harming; otherwise Poison for DoT.
        if distance_to_player < 4.0 {
            return WitchAction::ThrowPotion(PotionType::Harming);
        }
        return WitchAction::ThrowPotion(PotionType::Poison);
    }

    // Priority 4 — medium-range crowd control
    if distance_to_player < 16.0 {
        return WitchAction::ThrowPotion(PotionType::Slowness);
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
    fn detects_player_looking_directly_at_enderman_head() {
        let player_pos = Vec3::new(0.0, 1.6, 0.0); // eye height
        let enderman_pos = Vec3::new(0.0, 0.0, 10.0);
        let enderman_height = 2.9;
        // Look direction pointing straight at the head center
        let head_center = Vec3::new(0.0, enderman_height - 0.25, 10.0);
        let look_dir = (head_center - player_pos).normalize();

        assert!(is_player_looking_at(
            player_pos,
            look_dir,
            enderman_pos,
            enderman_height,
        ));
    }

    #[test]
    fn does_not_detect_when_looking_away() {
        let player_pos = Vec3::new(0.0, 1.6, 0.0);
        let enderman_pos = Vec3::new(0.0, 0.0, 10.0);
        let enderman_height = 2.9;
        // Looking in the opposite direction
        let look_dir = Vec3::new(0.0, 0.0, -1.0);

        assert!(!is_player_looking_at(
            player_pos,
            look_dir,
            enderman_pos,
            enderman_height,
        ));
    }

    #[test]
    fn does_not_detect_beyond_range() {
        let player_pos = Vec3::ZERO;
        let enderman_pos = Vec3::new(0.0, 0.0, 100.0); // > 64 blocks
        let enderman_height = 2.9;
        let head_center = Vec3::new(0.0, enderman_height - 0.25, 100.0);
        let look_dir = (head_center - player_pos).normalize();

        assert!(!is_player_looking_at(
            player_pos,
            look_dir,
            enderman_pos,
            enderman_height,
        ));
    }

    #[test]
    fn does_not_detect_looking_at_body_not_head() {
        let player_pos = Vec3::new(0.0, 1.6, 0.0);
        let enderman_pos = Vec3::new(0.0, 0.0, 10.0);
        let enderman_height = 2.9;
        // Look at the feet instead of the head
        let look_dir = (enderman_pos - player_pos).normalize();

        assert!(!is_player_looking_at(
            player_pos,
            look_dir,
            enderman_pos,
            enderman_height,
        ));
    }

    // -- Enderman: teleport -------------------------------------------------

    #[test]
    fn teleport_distance_is_within_range() {
        let origin = Vec3::new(100.0, 64.0, 200.0);

        for attempt in 0..20 {
            let dest = teleport_away(origin, 42, attempt);
            let horiz = Vec3::new(dest.x - origin.x, 0.0, dest.z - origin.z).length();
            assert!(
                horiz >= 16.0 - 0.01 && horiz <= 32.0 + 0.01,
                "horizontal distance {horiz} out of [16, 32] for attempt {attempt}"
            );
        }
    }

    #[test]
    fn teleport_is_deterministic() {
        let pos = Vec3::new(10.0, 20.0, 30.0);
        let a = teleport_away(pos, 123, 0);
        let b = teleport_away(pos, 123, 0);
        assert_eq!(a, b, "same seed+attempt should produce identical result");
    }

    #[test]
    fn different_attempts_produce_different_positions() {
        let pos = Vec3::ZERO;
        let a = teleport_away(pos, 99, 0);
        let b = teleport_away(pos, 99, 1);
        assert_ne!(a, b);
    }

    // -- Slime: health & damage ---------------------------------------------

    #[test]
    fn slime_health_scales_quadratically() {
        assert!((slime_health(1) - 1.0).abs() < f32::EPSILON);
        assert!((slime_health(2) - 4.0).abs() < f32::EPSILON);
        assert!((slime_health(4) - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn slime_damage_scales_linearly() {
        assert!((slime_damage(1) - 1.0).abs() < f32::EPSILON);
        assert!((slime_damage(2) - 2.0).abs() < f32::EPSILON);
        assert!((slime_damage(4) - 4.0).abs() < f32::EPSILON);
    }

    // -- Slime: splitting ---------------------------------------------------

    #[test]
    fn large_slime_splits_into_medium() {
        let children = split_on_death(Vec3::ZERO, 4, 777);
        assert!(
            children.len() >= 2 && children.len() <= 4,
            "expected 2-4 children, got {}",
            children.len()
        );
        for (_, size) in &children {
            assert_eq!(*size, 2);
        }
    }

    #[test]
    fn medium_slime_splits_into_small() {
        let children = split_on_death(Vec3::ZERO, 2, 888);
        assert!(
            children.len() >= 2 && children.len() <= 4,
            "expected 2-4 children, got {}",
            children.len()
        );
        for (_, size) in &children {
            assert_eq!(*size, 1);
        }
    }

    #[test]
    fn small_slime_does_not_split() {
        let children = split_on_death(Vec3::ZERO, 1, 999);
        assert!(children.is_empty());
    }

    #[test]
    fn split_children_are_offset_from_parent() {
        let parent_pos = Vec3::new(50.0, 64.0, 50.0);
        let children = split_on_death(parent_pos, 4, 12345);
        for (child_pos, _) in &children {
            assert_ne!(
                *child_pos, parent_pos,
                "child should be offset from parent"
            );
            // Y should remain the same
            assert!((child_pos.y - parent_pos.y).abs() < f32::EPSILON);
        }
    }

    // -- Witch: action priorities -------------------------------------------

    #[test]
    fn witch_drinks_healing_when_low_health() {
        let action = choose_witch_action(5.0, 4.0, 20.0, false);
        assert_eq!(action, WitchAction::DrinkPotion(PotionType::Healing));
    }

    #[test]
    fn witch_drinks_fire_resistance_when_on_fire() {
        let action = choose_witch_action(10.0, 18.0, 20.0, true);
        assert_eq!(action, WitchAction::DrinkPotion(PotionType::FireResistance));
    }

    #[test]
    fn witch_healing_takes_priority_over_fire_resistance() {
        // Both low health AND on fire — healing wins.
        let action = choose_witch_action(10.0, 5.0, 20.0, true);
        assert_eq!(action, WitchAction::DrinkPotion(PotionType::Healing));
    }

    #[test]
    fn witch_throws_harming_at_very_close_range() {
        let action = choose_witch_action(3.0, 20.0, 20.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Harming));
    }

    #[test]
    fn witch_throws_poison_at_close_range() {
        let action = choose_witch_action(6.0, 20.0, 20.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Poison));
    }

    #[test]
    fn witch_throws_slowness_at_medium_range() {
        let action = choose_witch_action(12.0, 20.0, 20.0, false);
        assert_eq!(action, WitchAction::ThrowPotion(PotionType::Slowness));
    }

    #[test]
    fn witch_idles_when_far_away() {
        let action = choose_witch_action(30.0, 20.0, 20.0, false);
        assert_eq!(action, WitchAction::Idle);
    }
}
