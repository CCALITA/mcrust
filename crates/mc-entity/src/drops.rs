use glam::Vec3;
use mc_core::BlockId;
use rand::Rng;

// ---------------------------------------------------------------------------
// Item IDs (u16) used by the drop system
// ---------------------------------------------------------------------------
// Block-derived items reuse BlockId discriminant values where possible.
// Non-block items use IDs in the 1000+ range to avoid collision.

const COBBLESTONE: u16 = BlockId::Cobblestone as u16;
const DIRT: u16 = BlockId::Dirt as u16;
const IRON_ORE: u16 = BlockId::IronOre as u16;
const GOLD_ORE: u16 = BlockId::GoldOre as u16;
const GRAVEL: u16 = BlockId::Gravel as u16;

// Material item IDs (matching mc-core ItemId discriminant offsets where known)
const STICK: u16 = 1000;
const COAL: u16 = 1001;
const DIAMOND: u16 = 1002;
const FLINT: u16 = 1003;

// Mob drop item IDs
const ROTTEN_FLESH: u16 = 2000;
const BONE: u16 = 2001;
const ARROW: u16 = 2002;
const GUNPOWDER: u16 = 2003;
const STRING_ITEM: u16 = 2004;
const RAW_PORKCHOP: u16 = 2005;
const LEATHER: u16 = 2006;
const RAW_BEEF: u16 = 2007;
const WOOL: u16 = 2008;
const FEATHER: u16 = 2009;
const RAW_CHICKEN: u16 = 2010;

// ---------------------------------------------------------------------------
// MobKind discriminants (matches component::MobKind repr order)
// ---------------------------------------------------------------------------

const MOB_ZOMBIE: u8 = 0;
const MOB_SKELETON: u8 = 1;
const MOB_CREEPER: u8 = 2;
const MOB_SPIDER: u8 = 3;
const MOB_PIG: u8 = 4;
const MOB_COW: u8 = 5;
const MOB_SHEEP: u8 = 6;
const MOB_CHICKEN: u8 = 7;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GRAVITY: f32 = -32.0;
const PICKUP_RANGE: f32 = 1.5;
const DEFAULT_MAX_LIFETIME: f32 = 300.0;
const DEFAULT_PICKUP_DELAY: f32 = 0.5;

// ---------------------------------------------------------------------------
// ItemDrop
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ItemDrop {
    pub item_id: u16,
    pub count: u8,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub pickup_delay: f32,
}

impl ItemDrop {
    pub fn new(item_id: u16, count: u8, pos: Vec3) -> Self {
        let mut rng = rand::rng();
        let vx = rng.random_range(-1.0..1.0);
        let vy = rng.random_range(2.0..4.0);
        let vz = rng.random_range(-1.0..1.0);

        Self {
            item_id,
            count,
            position: pos,
            velocity: Vec3::new(vx, vy, vz),
            lifetime: 0.0,
            max_lifetime: DEFAULT_MAX_LIFETIME,
            pickup_delay: DEFAULT_PICKUP_DELAY,
        }
    }
}

// ---------------------------------------------------------------------------
// XpOrb
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct XpOrb {
    pub xp_amount: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
}

impl XpOrb {
    pub fn new(amount: u32, pos: Vec3) -> Self {
        Self {
            xp_amount: amount,
            position: pos,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// DropSystem
// ---------------------------------------------------------------------------

pub struct DropSystem;

impl DropSystem {
    /// Update item drops: apply gravity, advance timers, check pickup range.
    /// Returns a list of `(item_id, count)` for items picked up by the player.
    /// Removes expired and picked-up drops from the vector.
    pub fn tick_drops(drops: &mut Vec<ItemDrop>, player_pos: Vec3, dt: f32) -> Vec<(u16, u8)> {
        let mut picked_up = Vec::new();

        drops.retain_mut(|drop| {
            // Apply gravity
            drop.velocity.y += GRAVITY * dt;
            drop.position += drop.velocity * dt;

            // Advance timers
            drop.lifetime += dt;
            drop.pickup_delay = (drop.pickup_delay - dt).max(0.0);

            // Check expiry
            if drop.lifetime >= drop.max_lifetime {
                return false;
            }

            // Check pickup
            if drop.pickup_delay <= 0.0 {
                let distance = (drop.position - player_pos).length();
                if distance <= PICKUP_RANGE {
                    picked_up.push((drop.item_id, drop.count));
                    return false;
                }
            }

            true
        });

        picked_up
    }

    /// Update XP orbs: apply gravity, advance timers, check pickup range.
    /// Returns total XP collected by the player.
    /// Removes expired and collected orbs from the vector.
    pub fn tick_xp(orbs: &mut Vec<XpOrb>, player_pos: Vec3, dt: f32) -> u32 {
        let mut total_xp = 0u32;

        orbs.retain_mut(|orb| {
            // Apply gravity
            orb.velocity.y += GRAVITY * dt;
            orb.position += orb.velocity * dt;

            // Advance timer
            orb.lifetime += dt;

            // Check expiry
            if orb.lifetime >= DEFAULT_MAX_LIFETIME {
                return false;
            }

            // Check pickup (XP orbs have no pickup delay)
            let distance = (orb.position - player_pos).length();
            if distance <= PICKUP_RANGE {
                total_xp += orb.xp_amount;
                return false;
            }

            true
        });

        total_xp
    }

    /// Merge nearby item drops of the same type within `merge_range` blocks.
    /// The first drop in each group absorbs the count from later duplicates.
    pub fn merge_nearby_drops(drops: &mut Vec<ItemDrop>, merge_range: f32) {
        let mut absorbed = vec![false; drops.len()];

        for i in 0..drops.len() {
            if absorbed[i] {
                continue;
            }
            for j in (i + 1)..drops.len() {
                if absorbed[j] {
                    continue;
                }
                if drops[i].item_id == drops[j].item_id {
                    let distance = (drops[i].position - drops[j].position).length();
                    if distance <= merge_range {
                        // Absorb count from j into i
                        let added = drops[j].count;
                        drops[i].count = drops[i].count.saturating_add(added);
                        absorbed[j] = true;
                    }
                }
            }
        }

        // Remove absorbed drops (iterate in reverse to preserve indices)
        let mut idx = drops.len();
        while idx > 0 {
            idx -= 1;
            if absorbed[idx] {
                drops.swap_remove(idx);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Block drop tables
// ---------------------------------------------------------------------------

/// Returns a list of `(item_id, count)` dropped when the given block is broken.
pub fn spawn_block_drops(block: BlockId) -> Vec<(u16, u8)> {
    match block {
        BlockId::Stone => vec![(COBBLESTONE, 1)],
        BlockId::DiamondOre => vec![(DIAMOND, 1)],
        BlockId::CoalOre => vec![(COAL, 1)],
        BlockId::IronOre => vec![(IRON_ORE, 1)],
        BlockId::GoldOre => vec![(GOLD_ORE, 1)],
        BlockId::GrassBlock => vec![(DIRT, 1)],
        BlockId::OakLeaves => {
            let mut rng = rand::rng();
            if rng.random_range(0..10) == 0 {
                vec![(STICK, 1)]
            } else {
                vec![]
            }
        }
        BlockId::Gravel => {
            let mut rng = rand::rng();
            if rng.random_range(0..10) == 0 {
                vec![(FLINT, 1)]
            } else {
                vec![(GRAVEL, 1)]
            }
        }
        // Blocks that drop nothing
        BlockId::Air | BlockId::Bedrock | BlockId::Water | BlockId::Glass => vec![],
        // Most blocks drop themselves
        other => vec![(other as u16, 1)],
    }
}

// ---------------------------------------------------------------------------
// Mob drop tables
// ---------------------------------------------------------------------------

/// Returns a list of `(item_id, count)` dropped when a mob of the given kind dies.
/// `kind` is the u8 discriminant of `MobKind`.
pub fn spawn_mob_drops(kind: u8) -> Vec<(u16, u8)> {
    let mut rng = rand::rng();
    match kind {
        MOB_ZOMBIE => {
            let count: u8 = rng.random_range(0..=2);
            if count > 0 {
                vec![(ROTTEN_FLESH, count)]
            } else {
                vec![]
            }
        }
        MOB_SKELETON => {
            let bones: u8 = rng.random_range(0..=2);
            let arrows: u8 = rng.random_range(0..=2);
            let mut drops = Vec::new();
            if bones > 0 {
                drops.push((BONE, bones));
            }
            if arrows > 0 {
                drops.push((ARROW, arrows));
            }
            drops
        }
        MOB_CREEPER => {
            let count: u8 = rng.random_range(0..=2);
            if count > 0 {
                vec![(GUNPOWDER, count)]
            } else {
                vec![]
            }
        }
        MOB_SPIDER => {
            let count: u8 = rng.random_range(0..=2);
            if count > 0 {
                vec![(STRING_ITEM, count)]
            } else {
                vec![]
            }
        }
        MOB_PIG => {
            let count: u8 = rng.random_range(1..=3);
            vec![(RAW_PORKCHOP, count)]
        }
        MOB_COW => {
            let leather: u8 = rng.random_range(0..=2);
            let beef: u8 = rng.random_range(1..=3);
            let mut drops = vec![(RAW_BEEF, beef)];
            if leather > 0 {
                drops.push((LEATHER, leather));
            }
            drops
        }
        MOB_SHEEP => {
            vec![(WOOL, 1)]
        }
        MOB_CHICKEN => {
            let feathers: u8 = rng.random_range(0..=2);
            let chicken: u8 = 1; // always drops 1 raw chicken
            let mut drops = vec![(RAW_CHICKEN, chicken)];
            if feathers > 0 {
                drops.push((FEATHER, feathers));
            }
            drops
        }
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pickup_within_range_collects_item() {
        let mut drops = vec![ItemDrop {
            item_id: 1,
            count: 5,
            position: Vec3::new(0.0, 0.0, 0.0),
            velocity: Vec3::ZERO,
            lifetime: 0.0,
            max_lifetime: DEFAULT_MAX_LIFETIME,
            pickup_delay: 0.0, // no delay
        }];

        let player_pos = Vec3::new(0.5, 0.0, 0.0); // within 1.5 blocks
        let picked = DropSystem::tick_drops(&mut drops, player_pos, 0.0);

        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0], (1, 5));
        assert!(drops.is_empty());
    }

    #[test]
    fn pickup_delay_prevents_early_pickup() {
        let mut drops = vec![ItemDrop {
            item_id: 1,
            count: 1,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
            max_lifetime: DEFAULT_MAX_LIFETIME,
            pickup_delay: 1.0, // 1 second delay
        }];

        let player_pos = Vec3::new(0.5, 0.0, 0.0);

        // Tick with small dt — delay not yet expired
        let picked = DropSystem::tick_drops(&mut drops, player_pos, 0.1);
        assert!(picked.is_empty());
        assert_eq!(drops.len(), 1);

        // Tick with enough dt to clear delay (0.9 remaining after first tick)
        // Reset position and velocity — gravity will move it during tick,
        // so place player at the post-gravity position
        drops[0].position = Vec3::ZERO;
        drops[0].velocity = Vec3::ZERO;
        // After tick with dt=1.0: velocity.y = GRAVITY*1.0, pos.y = GRAVITY*1.0
        let expected_y = GRAVITY * 1.0;
        let player_pos = Vec3::new(0.0, expected_y, 0.0);
        let picked = DropSystem::tick_drops(&mut drops, player_pos, 1.0);
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0], (1, 1));
    }

    #[test]
    fn expired_drops_are_removed() {
        let mut drops = vec![ItemDrop {
            item_id: 1,
            count: 1,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 299.0,
            max_lifetime: DEFAULT_MAX_LIFETIME,
            pickup_delay: 0.0,
        }];

        // Player is far away so no pickup
        let player_pos = Vec3::new(100.0, 100.0, 100.0);
        let picked = DropSystem::tick_drops(&mut drops, player_pos, 2.0);

        assert!(picked.is_empty());
        assert!(drops.is_empty(), "expired drop should be removed");
    }

    #[test]
    fn merge_combines_same_items() {
        let mut drops = vec![
            ItemDrop {
                item_id: 10,
                count: 3,
                position: Vec3::new(0.0, 0.0, 0.0),
                velocity: Vec3::ZERO,
                lifetime: 0.0,
                max_lifetime: DEFAULT_MAX_LIFETIME,
                pickup_delay: 0.0,
            },
            ItemDrop {
                item_id: 10,
                count: 5,
                position: Vec3::new(0.5, 0.0, 0.0),
                velocity: Vec3::ZERO,
                lifetime: 0.0,
                max_lifetime: DEFAULT_MAX_LIFETIME,
                pickup_delay: 0.0,
            },
        ];

        DropSystem::merge_nearby_drops(&mut drops, 1.0);

        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].count, 8);
    }

    #[test]
    fn merge_does_not_combine_different_items() {
        let mut drops = vec![
            ItemDrop {
                item_id: 10,
                count: 3,
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                lifetime: 0.0,
                max_lifetime: DEFAULT_MAX_LIFETIME,
                pickup_delay: 0.0,
            },
            ItemDrop {
                item_id: 20,
                count: 5,
                position: Vec3::new(0.5, 0.0, 0.0),
                velocity: Vec3::ZERO,
                lifetime: 0.0,
                max_lifetime: DEFAULT_MAX_LIFETIME,
                pickup_delay: 0.0,
            },
        ];

        DropSystem::merge_nearby_drops(&mut drops, 1.0);
        assert_eq!(drops.len(), 2);
    }

    #[test]
    fn merge_does_not_combine_distant_items() {
        let mut drops = vec![
            ItemDrop {
                item_id: 10,
                count: 3,
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                lifetime: 0.0,
                max_lifetime: DEFAULT_MAX_LIFETIME,
                pickup_delay: 0.0,
            },
            ItemDrop {
                item_id: 10,
                count: 5,
                position: Vec3::new(10.0, 0.0, 0.0),
                velocity: Vec3::ZERO,
                lifetime: 0.0,
                max_lifetime: DEFAULT_MAX_LIFETIME,
                pickup_delay: 0.0,
            },
        ];

        DropSystem::merge_nearby_drops(&mut drops, 1.0);
        assert_eq!(drops.len(), 2);
    }

    #[test]
    fn block_drop_stone_gives_cobblestone() {
        let drops = spawn_block_drops(BlockId::Stone);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (COBBLESTONE, 1));
    }

    #[test]
    fn block_drop_diamond_ore_gives_diamond() {
        let drops = spawn_block_drops(BlockId::DiamondOre);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (DIAMOND, 1));
    }

    #[test]
    fn block_drop_coal_ore_gives_coal() {
        let drops = spawn_block_drops(BlockId::CoalOre);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (COAL, 1));
    }

    #[test]
    fn block_drop_iron_ore_gives_iron_ore() {
        let drops = spawn_block_drops(BlockId::IronOre);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (IRON_ORE, 1));
    }

    #[test]
    fn block_drop_gold_ore_gives_gold_ore() {
        let drops = spawn_block_drops(BlockId::GoldOre);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (GOLD_ORE, 1));
    }

    #[test]
    fn block_drop_grass_gives_dirt() {
        let drops = spawn_block_drops(BlockId::GrassBlock);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (DIRT, 1));
    }

    #[test]
    fn block_drop_air_gives_nothing() {
        let drops = spawn_block_drops(BlockId::Air);
        assert!(drops.is_empty());
    }

    #[test]
    fn block_drop_self_for_generic_block() {
        let drops = spawn_block_drops(BlockId::OakPlanks);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (BlockId::OakPlanks as u16, 1));
    }

    #[test]
    fn mob_drop_zombie_non_empty_or_zero() {
        // Run multiple times to exercise randomness
        let mut found_nonzero = false;
        for _ in 0..100 {
            let drops = spawn_mob_drops(MOB_ZOMBIE);
            if !drops.is_empty() {
                found_nonzero = true;
                assert_eq!(drops[0].0, ROTTEN_FLESH);
                assert!(drops[0].1 >= 1 && drops[0].1 <= 2);
            }
        }
        // With 100 tries and 2/3 chance of nonzero, this should pass
        assert!(found_nonzero, "zombie should drop rotten flesh sometimes");
    }

    #[test]
    fn mob_drop_pig_always_drops() {
        for _ in 0..20 {
            let drops = spawn_mob_drops(MOB_PIG);
            assert!(!drops.is_empty(), "pig should always drop raw porkchop");
            assert_eq!(drops[0].0, RAW_PORKCHOP);
            assert!(drops[0].1 >= 1 && drops[0].1 <= 3);
        }
    }

    #[test]
    fn mob_drop_cow_always_drops_beef() {
        for _ in 0..20 {
            let drops = spawn_mob_drops(MOB_COW);
            assert!(!drops.is_empty());
            // First item should be raw beef
            assert_eq!(drops[0].0, RAW_BEEF);
            assert!(drops[0].1 >= 1 && drops[0].1 <= 3);
        }
    }

    #[test]
    fn mob_drop_sheep_always_drops_wool() {
        let drops = spawn_mob_drops(MOB_SHEEP);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (WOOL, 1));
    }

    #[test]
    fn mob_drop_chicken_always_drops_chicken() {
        for _ in 0..20 {
            let drops = spawn_mob_drops(MOB_CHICKEN);
            assert!(!drops.is_empty());
            assert_eq!(drops[0].0, RAW_CHICKEN);
            assert_eq!(drops[0].1, 1);
        }
    }

    #[test]
    fn mob_drop_unknown_kind_returns_empty() {
        let drops = spawn_mob_drops(255);
        assert!(drops.is_empty());
    }

    #[test]
    fn xp_orb_pickup_within_range() {
        let mut orbs = vec![XpOrb {
            xp_amount: 10,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
        }];

        let player_pos = Vec3::new(0.5, 0.0, 0.0);
        let xp = DropSystem::tick_xp(&mut orbs, player_pos, 0.0);

        assert_eq!(xp, 10);
        assert!(orbs.is_empty());
    }

    #[test]
    fn xp_orb_not_picked_up_when_far() {
        let mut orbs = vec![XpOrb {
            xp_amount: 10,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
        }];

        let player_pos = Vec3::new(100.0, 0.0, 0.0);
        let xp = DropSystem::tick_xp(&mut orbs, player_pos, 0.01);

        assert_eq!(xp, 0);
        assert_eq!(orbs.len(), 1);
    }

    #[test]
    fn xp_orb_expired_is_removed() {
        let mut orbs = vec![XpOrb {
            xp_amount: 10,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 299.0,
        }];

        let player_pos = Vec3::new(100.0, 0.0, 0.0);
        let xp = DropSystem::tick_xp(&mut orbs, player_pos, 2.0);

        assert_eq!(xp, 0);
        assert!(orbs.is_empty(), "expired orb should be removed");
    }

    #[test]
    fn item_drop_new_has_upward_velocity() {
        let drop = ItemDrop::new(1, 1, Vec3::ZERO);
        assert!(drop.velocity.y >= 2.0 && drop.velocity.y <= 4.0);
        assert!((drop.max_lifetime - DEFAULT_MAX_LIFETIME).abs() < f32::EPSILON);
        assert!((drop.pickup_delay - DEFAULT_PICKUP_DELAY).abs() < f32::EPSILON);
    }

    #[test]
    fn xp_orb_new_has_zero_velocity() {
        let orb = XpOrb::new(5, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(orb.velocity, Vec3::ZERO);
        assert_eq!(orb.xp_amount, 5);
        assert!((orb.lifetime - 0.0).abs() < f32::EPSILON);
    }
}
