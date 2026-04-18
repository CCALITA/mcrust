use mc_core::block::BlockId;

// ---------------------------------------------------------------------------
// Power constants
// ---------------------------------------------------------------------------

/// TNT explosion power (vanilla Minecraft).
pub const TNT_POWER: f32 = 4.0;
/// Creeper explosion power.
pub const CREEPER_POWER: f32 = 3.0;
/// Charged creeper explosion power.
pub const CHARGED_CREEPER_POWER: f32 = 6.0;

// ---------------------------------------------------------------------------
// Block resistance
// ---------------------------------------------------------------------------

/// Returns the blast resistance for a given block.
///
/// * Bedrock (hardness -1) is indestructible (`f32::INFINITY`).
/// * Obsidian has a fixed resistance of 1200.
/// * Everything else uses `hardness * 5`.
pub fn block_resistance(block: BlockId) -> f32 {
    let hardness = block.properties().hardness;
    if hardness < 0.0 {
        return f32::INFINITY;
    }
    if block == BlockId::Obsidian {
        return 1200.0;
    }
    hardness * 5.0
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// The outcome of an explosion calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionResult {
    /// Block positions that were destroyed.
    pub destroyed_blocks: Vec<(i32, i32, i32)>,
    /// Pairs of `(world_position, damage)` for entities near the blast.
    pub damage_map: Vec<((f32, f32, f32), f32)>,
}

// ---------------------------------------------------------------------------
// Ray directions (uniformly distributed on a unit sphere via cube faces)
// ---------------------------------------------------------------------------

/// Number of subdivision steps per cube-face axis.
const RAY_STEPS: i32 = 8;

/// Build a list of normalised ray directions that cover the unit sphere.
fn ray_directions() -> Vec<(f32, f32, f32)> {
    let mut dirs = Vec::new();
    let step = 2.0 / RAY_STEPS as f32;

    for i in 0..RAY_STEPS {
        for j in 0..RAY_STEPS {
            let u = -1.0 + (i as f32 + 0.5) * step;
            let v = -1.0 + (j as f32 + 0.5) * step;

            // Six cube faces projected onto the unit sphere.
            for &(x, y, z) in &[
                (1.0, u, v),
                (-1.0, u, v),
                (u, 1.0, v),
                (u, -1.0, v),
                (u, v, 1.0),
                (u, v, -1.0),
            ] {
                let len = (x * x + y * y + z * z).sqrt();
                dirs.push((x / len, y / len, z / len));
            }
        }
    }
    dirs
}

// ---------------------------------------------------------------------------
// Core explosion calculation
// ---------------------------------------------------------------------------

/// Calculate which blocks are destroyed and produce a damage map.
///
/// * `center` — world-space position of the explosion origin.
/// * `power`  — explosion power (e.g. [`TNT_POWER`]).
/// * `get_block` — callback that returns the [`BlockId`] at integer coords.
///
/// The algorithm casts rays outward from `center`, degrading each ray's
/// remaining intensity by `(resistance + 0.3) * 0.3` per block step. When
/// intensity drops to zero the ray stops.
pub fn calculate_explosion(
    center: (f32, f32, f32),
    power: f32,
    get_block: &dyn Fn(i32, i32, i32) -> BlockId,
) -> ExplosionResult {
    let mut destroyed: Vec<(i32, i32, i32)> = Vec::new();
    let mut seen = std::collections::HashSet::<(i32, i32, i32)>::new();

    let dirs = ray_directions();
    let max_dist = power * 1.5; // maximum reach in blocks
    let step_size: f32 = 0.3;

    for (dx, dy, dz) in &dirs {
        let mut intensity = power * (0.7 + rand_simple(dx, dy, dz) * 0.6);
        let mut t: f32 = 0.0;

        while intensity > 0.0 && t < max_dist {
            let x = center.0 + dx * t;
            let y = center.1 + dy * t;
            let z = center.2 + dz * t;

            let bx = x.floor() as i32;
            let by = y.floor() as i32;
            let bz = z.floor() as i32;

            let block = get_block(bx, by, bz);

            if !block.is_air() {
                let resistance = block_resistance(block);
                intensity -= (resistance + 0.3) * 0.3;

                if intensity > 0.0 && resistance < f32::INFINITY && seen.insert((bx, by, bz)) {
                    destroyed.push((bx, by, bz));
                }
            }

            t += step_size;
        }
    }

    ExplosionResult {
        destroyed_blocks: destroyed,
        damage_map: Vec::new(),
    }
}

/// Deterministic pseudo-random in [0, 1) seeded from ray direction.
fn rand_simple(dx: &f32, dy: &f32, dz: &f32) -> f32 {
    let bits =
        (dx.to_bits() ^ dy.to_bits().wrapping_mul(2654435761) ^ dz.to_bits().wrapping_mul(40503))
            .wrapping_mul(2246822519);
    (bits & 0x00FF_FFFF) as f32 / 16_777_216.0
}

// ---------------------------------------------------------------------------
// Entity damage
// ---------------------------------------------------------------------------

/// Calculate damage dealt to an entity from an explosion.
///
/// Damage falls off linearly with distance and is zero beyond `power` blocks.
pub fn calculate_entity_damage(
    entity_pos: (f32, f32, f32),
    center: (f32, f32, f32),
    power: f32,
) -> f32 {
    let dx = entity_pos.0 - center.0;
    let dy = entity_pos.1 - center.1;
    let dz = entity_pos.2 - center.2;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    if distance >= power {
        return 0.0;
    }

    let exposure = 1.0 - distance / power;
    // Vanilla-inspired formula: (exposure^2 + exposure) * 7 * power + 1
    let raw = (exposure * exposure + exposure) * 7.0 * power + 1.0;
    raw.max(0.0)
}

// ---------------------------------------------------------------------------
// Apply explosion
// ---------------------------------------------------------------------------

/// Replace all destroyed blocks with [`BlockId::Air`].
pub fn apply_explosion(
    result: &ExplosionResult,
    set_block: &mut dyn FnMut(i32, i32, i32, BlockId),
) {
    for &(x, y, z) in &result.destroyed_blocks {
        set_block(x, y, z, BlockId::Air);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: world filled with a single block type.
    fn uniform_world(block: BlockId) -> impl Fn(i32, i32, i32) -> BlockId {
        move |_x, _y, _z| block
    }

    #[test]
    fn bedrock_resistance_is_infinite() {
        assert!(block_resistance(BlockId::Bedrock).is_infinite());
    }

    #[test]
    fn obsidian_resistance_is_1200() {
        assert!((block_resistance(BlockId::Obsidian) - 1200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stone_resistance_equals_hardness_times_five() {
        let expected = BlockId::Stone.properties().hardness * 5.0;
        assert!((block_resistance(BlockId::Stone) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn air_resistance_is_zero() {
        assert!((block_resistance(BlockId::Air)).abs() < f32::EPSILON);
    }

    #[test]
    fn tnt_destroys_blocks_within_radius() {
        let result = calculate_explosion((0.0, 0.0, 0.0), TNT_POWER, &uniform_world(BlockId::Dirt));
        assert!(
            !result.destroyed_blocks.is_empty(),
            "TNT should destroy dirt blocks"
        );

        // All destroyed blocks should be within a reasonable radius.
        let max_radius_sq = (TNT_POWER * 2.0) * (TNT_POWER * 2.0);
        for (x, y, z) in &result.destroyed_blocks {
            let dist_sq = (*x as f32).powi(2) + (*y as f32).powi(2) + (*z as f32).powi(2);
            assert!(
                dist_sq <= max_radius_sq,
                "block ({x},{y},{z}) too far from center"
            );
        }
    }

    #[test]
    fn bedrock_survives_explosion() {
        let result =
            calculate_explosion((0.0, 0.0, 0.0), TNT_POWER, &uniform_world(BlockId::Bedrock));
        assert!(
            result.destroyed_blocks.is_empty(),
            "bedrock must not be destroyed"
        );
    }

    #[test]
    fn obsidian_survives_tnt() {
        let result = calculate_explosion(
            (0.0, 0.0, 0.0),
            TNT_POWER,
            &uniform_world(BlockId::Obsidian),
        );
        assert!(
            result.destroyed_blocks.is_empty(),
            "obsidian should survive a TNT explosion"
        );
    }

    #[test]
    fn larger_power_destroys_more() {
        let small = calculate_explosion(
            (0.0, 0.0, 0.0),
            CREEPER_POWER,
            &uniform_world(BlockId::Dirt),
        );
        let large = calculate_explosion(
            (0.0, 0.0, 0.0),
            CHARGED_CREEPER_POWER,
            &uniform_world(BlockId::Dirt),
        );
        assert!(
            large.destroyed_blocks.len() > small.destroyed_blocks.len(),
            "charged creeper ({}) should destroy more than creeper ({})",
            large.destroyed_blocks.len(),
            small.destroyed_blocks.len(),
        );
    }

    #[test]
    fn entity_damage_falloff() {
        let center = (0.0, 0.0, 0.0);
        let near = calculate_entity_damage((1.0, 0.0, 0.0), center, TNT_POWER);
        let far = calculate_entity_damage((3.0, 0.0, 0.0), center, TNT_POWER);
        assert!(near > far, "damage should decrease with distance");
    }

    #[test]
    fn entity_damage_zero_beyond_power() {
        let center = (0.0, 0.0, 0.0);
        let damage = calculate_entity_damage((TNT_POWER, 0.0, 0.0), center, TNT_POWER);
        assert!(
            damage.abs() < f32::EPSILON,
            "damage at edge should be zero, got {damage}"
        );

        let beyond = calculate_entity_damage((TNT_POWER + 1.0, 0.0, 0.0), center, TNT_POWER);
        assert!(
            beyond.abs() < f32::EPSILON,
            "damage beyond radius should be zero, got {beyond}"
        );
    }

    #[test]
    fn entity_damage_at_center_is_maximum() {
        let center = (5.0, 10.0, 5.0);
        let at_center = calculate_entity_damage(center, center, TNT_POWER);
        let nearby = calculate_entity_damage((6.0, 10.0, 5.0), center, TNT_POWER);
        assert!(at_center > nearby, "damage at center should be maximum");
    }

    #[test]
    fn apply_explosion_sets_blocks_to_air() {
        let result = ExplosionResult {
            destroyed_blocks: vec![(0, 0, 0), (1, 0, 0), (0, 1, 0)],
            damage_map: Vec::new(),
        };
        let mut set_calls: Vec<(i32, i32, i32, BlockId)> = Vec::new();
        apply_explosion(&result, &mut |x, y, z, block| {
            set_calls.push((x, y, z, block));
        });
        assert_eq!(set_calls.len(), 3);
        for (_, _, _, block) in &set_calls {
            assert_eq!(*block, BlockId::Air);
        }
    }

    #[test]
    fn no_duplicate_destroyed_blocks() {
        let result = calculate_explosion((0.0, 0.0, 0.0), TNT_POWER, &uniform_world(BlockId::Dirt));
        let mut seen = std::collections::HashSet::new();
        for pos in &result.destroyed_blocks {
            assert!(seen.insert(pos), "duplicate destroyed block at {pos:?}");
        }
    }

    #[test]
    fn explosion_in_air_destroys_nothing() {
        let result = calculate_explosion((0.0, 0.0, 0.0), TNT_POWER, &uniform_world(BlockId::Air));
        assert!(
            result.destroyed_blocks.is_empty(),
            "explosion in air should destroy no blocks"
        );
    }

    #[test]
    fn power_constants_are_correct() {
        assert!((TNT_POWER - 4.0).abs() < f32::EPSILON);
        assert!((CREEPER_POWER - 3.0).abs() < f32::EPSILON);
        assert!((CHARGED_CREEPER_POWER - 6.0).abs() < f32::EPSILON);
    }
}
