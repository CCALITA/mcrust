use mc_core::BlockId;

/// Effects that a beacon can grant to nearby players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeaconEffect {
    Speed,
    Haste,
    Resistance,
    JumpBoost,
    Strength,
    Regeneration,
}

/// Persistent state of a single beacon block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconState {
    pub level: u8,
    pub primary: Option<BeaconEffect>,
    pub secondary: Option<BeaconEffect>,
}

/// Returns `true` if the block is a valid beacon pyramid base material.
///
/// In vanilla Minecraft, valid blocks are Iron Block, Gold Block, Diamond Block,
/// and Emerald Block.  This codebase does not yet have those refined block
/// variants, so the corresponding ore blocks are used as placeholders.
pub fn is_beacon_base_block(block: BlockId) -> bool {
    matches!(
        block,
        BlockId::IronOre | BlockId::GoldOre | BlockId::DiamondOre | BlockId::EmeraldOre
    )
}

/// Scans the pyramid below a beacon and returns the highest complete level (0-4).
///
/// Layer sizes: L1 = 3x3, L2 = 5x5, L3 = 7x7, L4 = 9x9.
/// Each layer is centred directly below the beacon, one Y-level lower per layer.
/// All blocks in a layer must pass [`is_beacon_base_block`] for that layer to count.
pub fn scan_pyramid(
    beacon_pos: (i32, i32, i32),
    get_block: &dyn Fn(i32, i32, i32) -> BlockId,
) -> u8 {
    let (bx, by, bz) = beacon_pos;

    for layer in 1..=4u8 {
        let extent = layer as i32; // 1 → 3x3, 2 → 5x5, etc.
        let y = by - layer as i32;

        for dx in -extent..=extent {
            for dz in -extent..=extent {
                if !is_beacon_base_block(get_block(bx + dx, y, bz + dz)) {
                    return layer - 1;
                }
            }
        }
    }

    4
}

/// Returns the beacon effects that become available at a given pyramid level.
///
/// Level 0 grants no effects.  Higher levels cumulatively unlock more options:
/// - Level 1: Speed, Haste
/// - Level 2: + Resistance, JumpBoost
/// - Level 3: + Strength
/// - Level 4: + Regeneration
pub fn available_effects(level: u8) -> Vec<BeaconEffect> {
    let mut effects = Vec::new();
    if level >= 1 {
        effects.push(BeaconEffect::Speed);
        effects.push(BeaconEffect::Haste);
    }
    if level >= 2 {
        effects.push(BeaconEffect::Resistance);
        effects.push(BeaconEffect::JumpBoost);
    }
    if level >= 3 {
        effects.push(BeaconEffect::Strength);
    }
    if level >= 4 {
        effects.push(BeaconEffect::Regeneration);
    }
    effects
}

/// Returns the effect range (in blocks) for a beacon at the given pyramid level.
pub fn beacon_range(level: u8) -> f32 {
    match level {
        0 => 0.0,
        1 => 20.0,
        2 => 30.0,
        3 => 40.0,
        4 => 50.0,
        _ => 50.0, // clamp to max
    }
}

/// Returns `true` if `entity_pos` is within the beacon's effect range.
///
/// Uses Euclidean distance from the beacon centre to the entity position.
pub fn is_in_range(beacon_pos: (i32, i32, i32), entity_pos: (f64, f64, f64), level: u8) -> bool {
    let range = beacon_range(level) as f64;
    if range <= 0.0 {
        return false;
    }

    let dx = entity_pos.0 - beacon_pos.0 as f64;
    let dy = entity_pos.1 - beacon_pos.1 as f64;
    let dz = entity_pos.2 - beacon_pos.2 as f64;
    let dist_sq = dx * dx + dy * dy + dz * dz;

    dist_sq <= range * range
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a `get_block` closure from a filled block type at every position.
    fn uniform_world(block: BlockId) -> impl Fn(i32, i32, i32) -> BlockId {
        move |_x, _y, _z| block
    }

    // ------------------------------------------------------------------
    // Pyramid scanning
    // ------------------------------------------------------------------

    #[test]
    fn full_iron_pyramid_returns_level_4() {
        let get = uniform_world(BlockId::IronOre);
        assert_eq!(scan_pyramid((0, 10, 0), &get), 4);
    }

    #[test]
    fn full_gold_pyramid_returns_level_4() {
        let get = uniform_world(BlockId::GoldOre);
        assert_eq!(scan_pyramid((0, 10, 0), &get), 4);
    }

    #[test]
    fn full_diamond_pyramid_returns_level_4() {
        let get = uniform_world(BlockId::DiamondOre);
        assert_eq!(scan_pyramid((0, 10, 0), &get), 4);
    }

    #[test]
    fn full_emerald_pyramid_returns_level_4() {
        let get = uniform_world(BlockId::EmeraldOre);
        assert_eq!(scan_pyramid((0, 10, 0), &get), 4);
    }

    #[test]
    fn no_pyramid_returns_level_0() {
        let get = uniform_world(BlockId::Air);
        assert_eq!(scan_pyramid((0, 10, 0), &get), 0);
    }

    #[test]
    fn partial_pyramid_returns_lower_level() {
        // Only layer 1 (3x3) is valid; layer 2 (5x5) has air at corners.
        let get = |x: i32, y: i32, z: i32| -> BlockId {
            let _ = y;
            if x.abs() <= 1 && z.abs() <= 1 {
                BlockId::IronOre
            } else {
                BlockId::Air
            }
        };
        assert_eq!(scan_pyramid((0, 10, 0), &get), 1);
    }

    #[test]
    fn mixed_valid_blocks_count() {
        // Alternate iron and gold — both are valid base blocks.
        let get = |x: i32, _y: i32, _z: i32| -> BlockId {
            if x % 2 == 0 {
                BlockId::IronOre
            } else {
                BlockId::GoldOre
            }
        };
        assert_eq!(scan_pyramid((0, 10, 0), &get), 4);
    }

    #[test]
    fn single_invalid_block_in_layer_breaks_level() {
        // Full pyramid except one block in layer 3 (7x7 at y=7).
        let get = |x: i32, y: i32, z: i32| -> BlockId {
            if x == 3 && y == 7 && z == 3 {
                BlockId::Stone
            } else {
                BlockId::IronOre
            }
        };
        assert_eq!(scan_pyramid((0, 10, 0), &get), 2);
    }

    // ------------------------------------------------------------------
    // Base block validation
    // ------------------------------------------------------------------

    #[test]
    fn valid_base_blocks() {
        assert!(is_beacon_base_block(BlockId::IronOre));
        assert!(is_beacon_base_block(BlockId::GoldOre));
        assert!(is_beacon_base_block(BlockId::DiamondOre));
        assert!(is_beacon_base_block(BlockId::EmeraldOre));
    }

    #[test]
    fn invalid_base_blocks() {
        assert!(!is_beacon_base_block(BlockId::Stone));
        assert!(!is_beacon_base_block(BlockId::Air));
        assert!(!is_beacon_base_block(BlockId::Cobblestone));
        assert!(!is_beacon_base_block(BlockId::CoalOre));
    }

    // ------------------------------------------------------------------
    // Available effects
    // ------------------------------------------------------------------

    #[test]
    fn level_0_has_no_effects() {
        assert!(available_effects(0).is_empty());
    }

    #[test]
    fn level_1_effects() {
        let effects = available_effects(1);
        assert_eq!(effects, vec![BeaconEffect::Speed, BeaconEffect::Haste]);
    }

    #[test]
    fn level_2_effects() {
        let effects = available_effects(2);
        assert_eq!(
            effects,
            vec![
                BeaconEffect::Speed,
                BeaconEffect::Haste,
                BeaconEffect::Resistance,
                BeaconEffect::JumpBoost,
            ]
        );
    }

    #[test]
    fn level_3_effects() {
        let effects = available_effects(3);
        assert!(effects.contains(&BeaconEffect::Strength));
        assert_eq!(effects.len(), 5);
    }

    #[test]
    fn level_4_effects() {
        let effects = available_effects(4);
        assert!(effects.contains(&BeaconEffect::Regeneration));
        assert_eq!(effects.len(), 6);
    }

    // ------------------------------------------------------------------
    // Beacon range
    // ------------------------------------------------------------------

    #[test]
    fn range_values() {
        assert_eq!(beacon_range(0), 0.0);
        assert_eq!(beacon_range(1), 20.0);
        assert_eq!(beacon_range(2), 30.0);
        assert_eq!(beacon_range(3), 40.0);
        assert_eq!(beacon_range(4), 50.0);
    }

    // ------------------------------------------------------------------
    // Range check
    // ------------------------------------------------------------------

    #[test]
    fn entity_inside_range() {
        assert!(is_in_range((0, 10, 0), (10.0, 10.0, 0.0), 1));
    }

    #[test]
    fn entity_outside_range() {
        // Distance = sqrt(25^2 + 25^2 + 25^2) ≈ 43.3, range at L1 = 20
        assert!(!is_in_range((0, 10, 0), (25.0, 35.0, 25.0), 1));
    }

    #[test]
    fn entity_at_boundary() {
        // Exactly at range boundary should be in range (<=).
        assert!(is_in_range((0, 0, 0), (20.0, 0.0, 0.0), 1));
    }

    #[test]
    fn level_0_never_in_range() {
        assert!(!is_in_range((0, 0, 0), (0.0, 0.0, 0.0), 0));
    }

    // ------------------------------------------------------------------
    // BeaconState
    // ------------------------------------------------------------------

    #[test]
    fn beacon_state_default() {
        let state = BeaconState {
            level: 0,
            primary: None,
            secondary: None,
        };
        assert_eq!(state.level, 0);
        assert!(state.primary.is_none());
        assert!(state.secondary.is_none());
    }

    #[test]
    fn beacon_state_with_effects() {
        let state = BeaconState {
            level: 4,
            primary: Some(BeaconEffect::Speed),
            secondary: Some(BeaconEffect::Regeneration),
        };
        assert_eq!(state.level, 4);
        assert_eq!(state.primary, Some(BeaconEffect::Speed));
        assert_eq!(state.secondary, Some(BeaconEffect::Regeneration));
    }
}
