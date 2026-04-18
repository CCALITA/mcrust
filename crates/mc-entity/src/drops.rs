// ---------------------------------------------------------------------------
// Item drops & XP orb entities
// ---------------------------------------------------------------------------

use glam::Vec3;

/// Gravity acceleration applied to dropped items (blocks/s^2).
const DROP_GRAVITY: f32 = -20.0;

/// Maximum lifetime for item drops (seconds).
const DROP_MAX_LIFETIME: f32 = 300.0;

/// Maximum lifetime for XP orbs (seconds).
const XP_MAX_LIFETIME: f32 = 300.0;

/// Pickup radius for items (blocks).
const PICKUP_RADIUS: f32 = 1.5;

/// Pickup radius for XP orbs (blocks).
const XP_PICKUP_RADIUS: f32 = 1.5;

/// An item entity floating in the world, waiting to be picked up.
#[derive(Debug, Clone)]
pub struct ItemDrop {
    pub item_id: u16,
    pub count: u8,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub pickup_delay: f32,
}

/// An experience orb floating in the world.
#[derive(Debug, Clone)]
pub struct XpOrb {
    pub xp_amount: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
}

// ---------------------------------------------------------------------------
// Tick systems
// ---------------------------------------------------------------------------

/// Advance all item drops by `dt` seconds.
///
/// - Applies gravity to each drop's velocity and updates position.
/// - Counts down pickup delay.
/// - Removes drops whose lifetime exceeds `DROP_MAX_LIFETIME`.
/// - Returns a `Vec<(item_id, count)>` of items collected by the player
///   (within `PICKUP_RADIUS` blocks and past their pickup delay).
pub fn tick_drops(drops: &mut Vec<ItemDrop>, player_pos: Vec3, dt: f32) -> Vec<(u16, u8)> {
    let mut collected: Vec<(u16, u8)> = Vec::new();

    let mut i = 0;
    while i < drops.len() {
        let drop = &mut drops[i];

        // Physics
        drop.velocity.y += DROP_GRAVITY * dt;
        drop.position += drop.velocity * dt;

        // Clamp to ground (y >= 0)
        if drop.position.y < 0.0 {
            drop.position.y = 0.0;
            drop.velocity.y = 0.0;
        }

        // Advance timers
        drop.lifetime += dt;
        drop.pickup_delay = (drop.pickup_delay - dt).max(0.0);

        // Check expiry
        if drop.lifetime >= DROP_MAX_LIFETIME {
            drops.swap_remove(i);
            continue;
        }

        // Pickup check
        if drop.pickup_delay <= 0.0 {
            let dist = (drop.position - player_pos).length();
            if dist <= PICKUP_RADIUS {
                collected.push((drop.item_id, drop.count));
                drops.swap_remove(i);
                continue;
            }
        }

        i += 1;
    }

    collected
}

/// Advance all XP orbs by `dt` seconds.
///
/// - Applies gravity and updates position.
/// - Removes orbs whose lifetime exceeds `XP_MAX_LIFETIME`.
/// - Returns the total XP collected by the player this tick.
pub fn tick_xp(orbs: &mut Vec<XpOrb>, player_pos: Vec3, dt: f32) -> u32 {
    let mut total_xp: u32 = 0;

    let mut i = 0;
    while i < orbs.len() {
        let orb = &mut orbs[i];

        // Physics
        orb.velocity.y += DROP_GRAVITY * dt;
        orb.position += orb.velocity * dt;

        if orb.position.y < 0.0 {
            orb.position.y = 0.0;
            orb.velocity.y = 0.0;
        }

        orb.lifetime += dt;

        // Expiry
        if orb.lifetime >= XP_MAX_LIFETIME {
            orbs.swap_remove(i);
            continue;
        }

        // Pickup
        let dist = (orb.position - player_pos).length();
        if dist <= XP_PICKUP_RADIUS {
            total_xp += orb.xp_amount;
            orbs.swap_remove(i);
            continue;
        }

        i += 1;
    }

    total_xp
}

// ---------------------------------------------------------------------------
// Merge nearby drops
// ---------------------------------------------------------------------------

/// Merge item drops of the same `item_id` that are within `range` blocks of
/// each other. The merged stack accumulates into the first drop found; the
/// duplicate is removed.
///
/// This reduces entity count without changing the total number of items.
pub fn merge_nearby_drops(drops: &mut Vec<ItemDrop>, range: f32) {
    let mut i = 0;
    while i < drops.len() {
        let mut j = i + 1;
        while j < drops.len() {
            let same_item = drops[i].item_id == drops[j].item_id;
            let dist = (drops[i].position - drops[j].position).length();

            if same_item && dist <= range {
                // Merge j into i (saturating at u8::MAX)
                drops[i].count = drops[i].count.saturating_add(drops[j].count);
                drops.swap_remove(j);
                // Don't increment j — the swapped element needs checking
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Drop tables
// ---------------------------------------------------------------------------

/// Item IDs used for drop results.
///
/// These mirror the `BlockId` discriminants from `mc-core` where the dropped
/// item is the block itself, and use offsets above `BlockId::COUNT` (84) for
/// non-block items.
pub mod item_ids {
    // Block-as-item (same ID as BlockId discriminant)
    pub const COBBLESTONE: u16 = 11;
    pub const DIRT: u16 = 2;
    pub const SAND: u16 = 6;
    pub const GRAVEL: u16 = 7;
    pub const OAK_LOG: u16 = 8;
    pub const OAK_PLANKS: u16 = 10;
    pub const COAL_ORE: u16 = 12;
    pub const GLASS: u16 = 16;

    // Non-block items (offset from BlockId::COUNT)
    pub const DIAMOND: u16 = 100;
    pub const COAL: u16 = 101;
    pub const IRON_INGOT: u16 = 102;
    pub const GOLD_INGOT: u16 = 103;
    pub const EMERALD: u16 = 104;
    pub const LAPIS_LAZULI: u16 = 105;
    pub const REDSTONE_DUST_ITEM: u16 = 106;
    pub const COPPER_INGOT: u16 = 107;
    pub const ROTTEN_FLESH: u16 = 110;
    pub const BONE: u16 = 111;
    pub const GUNPOWDER: u16 = 112;
    pub const STRING: u16 = 113;
    pub const RAW_PORKCHOP: u16 = 114;
    pub const RAW_BEEF: u16 = 115;
    pub const RAW_MUTTON: u16 = 116;
    pub const RAW_CHICKEN: u16 = 117;
    pub const FEATHER: u16 = 118;
}

/// Returns the items dropped when a block is broken.
///
/// `block` is the `BlockId` discriminant (`u16`). Returns a `Vec` of
/// `(item_id, count)` tuples — empty for blocks that drop nothing (e.g. air).
pub fn spawn_block_drops(block: u16) -> Vec<(u16, u8)> {
    match block {
        0 => vec![],                                  // Air
        1 => vec![(item_ids::COBBLESTONE, 1)],        // Stone -> cobblestone
        2 => vec![(item_ids::DIRT, 1)],               // Dirt
        6 => vec![(item_ids::SAND, 1)],               // Sand
        7 => vec![(item_ids::GRAVEL, 1)],             // Gravel
        8 => vec![(item_ids::OAK_LOG, 1)],            // OakLog
        10 => vec![(item_ids::OAK_PLANKS, 1)],        // OakPlanks
        11 => vec![(item_ids::COBBLESTONE, 1)],       // Cobblestone
        12 => vec![(item_ids::COAL, 1)],              // CoalOre -> coal
        13 => vec![(item_ids::IRON_INGOT, 1)],        // IronOre -> iron ingot (raw)
        14 => vec![(item_ids::GOLD_INGOT, 1)],        // GoldOre -> gold ingot (raw)
        15 => vec![(item_ids::DIAMOND, 1)],           // DiamondOre -> diamond
        16 => vec![],                                 // Glass -> nothing (no silk touch)
        32 => vec![(item_ids::COPPER_INGOT, 1)],      // CopperOre
        33 => vec![(item_ids::LAPIS_LAZULI, 4)],      // LapisOre -> 4-9 lapis
        34 => vec![(item_ids::EMERALD, 1)],           // EmeraldOre -> emerald
        35 => vec![(item_ids::REDSTONE_DUST_ITEM, 4)], // RedstoneOre -> 4-5 redstone
        _ => {
            // Default: the block drops itself (1:1 mapping, count 1).
            // Air (0) is already handled above.
            vec![(block, 1)]
        }
    }
}

/// Returns the items dropped when a mob is killed.
///
/// `kind` is the `MobKind` discriminant (`u8`).
pub fn spawn_mob_drops(kind: u8) -> Vec<(u16, u8)> {
    match kind {
        0 => vec![(item_ids::ROTTEN_FLESH, 2)],      // Zombie -> rotten flesh
        1 => vec![(item_ids::BONE, 2)],               // Skeleton -> bone
        2 => vec![(item_ids::GUNPOWDER, 2)],          // Creeper -> gunpowder
        3 => vec![(item_ids::STRING, 2)],              // Spider -> string
        4 => vec![(item_ids::RAW_PORKCHOP, 2)],       // Pig -> raw porkchop
        5 => vec![(item_ids::RAW_BEEF, 2)],            // Cow -> raw beef
        6 => vec![(item_ids::RAW_MUTTON, 2)],          // Sheep -> raw mutton
        7 => vec![(item_ids::RAW_CHICKEN, 1), (item_ids::FEATHER, 2)], // Chicken
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helpers ------------------------------------------------------------

    fn make_drop(item_id: u16, count: u8, position: Vec3) -> ItemDrop {
        ItemDrop {
            item_id,
            count,
            position,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
            pickup_delay: 0.0,
        }
    }

    fn make_xp_orb(xp: u32, position: Vec3) -> XpOrb {
        XpOrb {
            xp_amount: xp,
            position,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
        }
    }

    // -- tick_drops pickup --------------------------------------------------

    #[test]
    fn player_picks_up_drop_within_radius() {
        let player = Vec3::new(0.0, 0.0, 0.0);
        let mut drops = vec![make_drop(1, 5, Vec3::new(1.0, 0.0, 0.0))];

        let collected = tick_drops(&mut drops, player, 0.05);

        assert_eq!(collected, vec![(1, 5)]);
        assert!(drops.is_empty());
    }

    #[test]
    fn player_does_not_pick_up_drop_outside_radius() {
        let player = Vec3::new(0.0, 0.0, 0.0);
        let mut drops = vec![make_drop(1, 5, Vec3::new(10.0, 0.0, 0.0))];

        let collected = tick_drops(&mut drops, player, 0.05);

        assert!(collected.is_empty());
        assert_eq!(drops.len(), 1);
    }

    #[test]
    fn pickup_delay_prevents_collection() {
        let player = Vec3::new(0.0, 0.0, 0.0);
        let mut drops = vec![ItemDrop {
            item_id: 1,
            count: 1,
            position: Vec3::new(0.5, 0.0, 0.0),
            velocity: Vec3::ZERO,
            lifetime: 0.0,
            pickup_delay: 2.0,
        }];

        // Tick with small dt — delay still active
        let collected = tick_drops(&mut drops, player, 0.05);
        assert!(collected.is_empty());
        assert_eq!(drops.len(), 1);
    }

    #[test]
    fn drop_expires_after_max_lifetime() {
        let player = Vec3::new(100.0, 0.0, 0.0); // Far away
        let mut drops = vec![ItemDrop {
            item_id: 1,
            count: 1,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: DROP_MAX_LIFETIME - 0.01,
            pickup_delay: 0.0,
        }];

        let collected = tick_drops(&mut drops, player, 0.05);

        assert!(collected.is_empty());
        assert!(drops.is_empty(), "expired drop should be removed");
    }

    #[test]
    fn gravity_pulls_drops_down() {
        let player = Vec3::new(100.0, 0.0, 0.0);
        let mut drops = vec![ItemDrop {
            item_id: 1,
            count: 1,
            position: Vec3::new(0.0, 10.0, 0.0),
            velocity: Vec3::ZERO,
            lifetime: 0.0,
            pickup_delay: 0.0,
        }];

        tick_drops(&mut drops, player, 1.0);

        assert!(!drops.is_empty());
        assert!(drops[0].position.y < 10.0, "drop should fall due to gravity");
    }

    #[test]
    fn drop_position_clamped_at_ground() {
        let player = Vec3::new(100.0, 0.0, 0.0);
        let mut drops = vec![ItemDrop {
            item_id: 1,
            count: 1,
            position: Vec3::new(0.0, 0.5, 0.0),
            velocity: Vec3::new(0.0, -100.0, 0.0),
            lifetime: 0.0,
            pickup_delay: 0.0,
        }];

        tick_drops(&mut drops, player, 1.0);

        assert!(!drops.is_empty());
        assert!(
            drops[0].position.y >= 0.0,
            "drop position should not go below ground"
        );
    }

    // -- tick_xp pickup -----------------------------------------------------

    #[test]
    fn player_collects_xp_orb_within_radius() {
        let player = Vec3::new(0.0, 0.0, 0.0);
        let mut orbs = vec![make_xp_orb(10, Vec3::new(1.0, 0.0, 0.0))];

        let xp = tick_xp(&mut orbs, player, 0.05);

        assert_eq!(xp, 10);
        assert!(orbs.is_empty());
    }

    #[test]
    fn xp_orb_outside_radius_not_collected() {
        let player = Vec3::new(0.0, 0.0, 0.0);
        let mut orbs = vec![make_xp_orb(10, Vec3::new(10.0, 0.0, 0.0))];

        let xp = tick_xp(&mut orbs, player, 0.05);

        assert_eq!(xp, 0);
        assert_eq!(orbs.len(), 1);
    }

    #[test]
    fn multiple_xp_orbs_sum_collected() {
        let player = Vec3::new(0.0, 0.0, 0.0);
        let mut orbs = vec![
            make_xp_orb(5, Vec3::new(0.5, 0.0, 0.0)),
            make_xp_orb(3, Vec3::new(0.0, 0.0, 0.5)),
        ];

        let xp = tick_xp(&mut orbs, player, 0.05);

        assert_eq!(xp, 8);
        assert!(orbs.is_empty());
    }

    #[test]
    fn xp_orb_expires_after_max_lifetime() {
        let player = Vec3::new(100.0, 0.0, 0.0);
        let mut orbs = vec![XpOrb {
            xp_amount: 10,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: XP_MAX_LIFETIME - 0.01,
        }];

        let xp = tick_xp(&mut orbs, player, 0.05);

        assert_eq!(xp, 0);
        assert!(orbs.is_empty(), "expired orb should be removed");
    }

    // -- merge_nearby_drops -------------------------------------------------

    #[test]
    fn merges_same_item_within_range() {
        let mut drops = vec![
            make_drop(1, 3, Vec3::new(0.0, 0.0, 0.0)),
            make_drop(1, 5, Vec3::new(0.5, 0.0, 0.0)),
        ];

        merge_nearby_drops(&mut drops, 1.0);

        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].count, 8);
    }

    #[test]
    fn does_not_merge_different_items() {
        let mut drops = vec![
            make_drop(1, 3, Vec3::new(0.0, 0.0, 0.0)),
            make_drop(2, 5, Vec3::new(0.5, 0.0, 0.0)),
        ];

        merge_nearby_drops(&mut drops, 1.0);

        assert_eq!(drops.len(), 2);
    }

    #[test]
    fn does_not_merge_outside_range() {
        let mut drops = vec![
            make_drop(1, 3, Vec3::new(0.0, 0.0, 0.0)),
            make_drop(1, 5, Vec3::new(10.0, 0.0, 0.0)),
        ];

        merge_nearby_drops(&mut drops, 1.0);

        assert_eq!(drops.len(), 2);
    }

    #[test]
    fn merge_saturates_at_max_u8() {
        let mut drops = vec![
            make_drop(1, 200, Vec3::ZERO),
            make_drop(1, 200, Vec3::ZERO),
        ];

        merge_nearby_drops(&mut drops, 1.0);

        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].count, 255);
    }

    // -- spawn_block_drops --------------------------------------------------

    #[test]
    fn stone_drops_cobblestone() {
        let drops = spawn_block_drops(1);
        assert_eq!(drops, vec![(item_ids::COBBLESTONE, 1)]);
    }

    #[test]
    fn diamond_ore_drops_diamond() {
        let drops = spawn_block_drops(15);
        assert_eq!(drops, vec![(item_ids::DIAMOND, 1)]);
    }

    #[test]
    fn coal_ore_drops_coal() {
        let drops = spawn_block_drops(12);
        assert_eq!(drops, vec![(item_ids::COAL, 1)]);
    }

    #[test]
    fn glass_drops_nothing() {
        let drops = spawn_block_drops(16);
        assert!(drops.is_empty());
    }

    #[test]
    fn air_drops_nothing() {
        let drops = spawn_block_drops(0);
        assert!(drops.is_empty());
    }

    #[test]
    fn unknown_block_drops_itself() {
        let drops = spawn_block_drops(50);
        assert_eq!(drops, vec![(50, 1)]);
    }

    // -- spawn_mob_drops ----------------------------------------------------

    #[test]
    fn zombie_drops_rotten_flesh() {
        let drops = spawn_mob_drops(0);
        assert_eq!(drops, vec![(item_ids::ROTTEN_FLESH, 2)]);
    }

    #[test]
    fn skeleton_drops_bone() {
        let drops = spawn_mob_drops(1);
        assert_eq!(drops, vec![(item_ids::BONE, 2)]);
    }

    #[test]
    fn creeper_drops_gunpowder() {
        let drops = spawn_mob_drops(2);
        assert_eq!(drops, vec![(item_ids::GUNPOWDER, 2)]);
    }

    #[test]
    fn spider_drops_string() {
        let drops = spawn_mob_drops(3);
        assert_eq!(drops, vec![(item_ids::STRING, 2)]);
    }

    #[test]
    fn chicken_drops_chicken_and_feather() {
        let drops = spawn_mob_drops(7);
        assert_eq!(drops.len(), 2);
        assert!(drops.contains(&(item_ids::RAW_CHICKEN, 1)));
        assert!(drops.contains(&(item_ids::FEATHER, 2)));
    }

    #[test]
    fn unknown_mob_drops_nothing() {
        let drops = spawn_mob_drops(255);
        assert!(drops.is_empty());
    }
}
