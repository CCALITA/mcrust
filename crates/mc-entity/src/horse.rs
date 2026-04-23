// ---------------------------------------------------------------------------
// Horse variants and stats
// ---------------------------------------------------------------------------

/// All horse-like mob variants in Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorseVariant {
    Horse,
    Donkey,
    Mule,
    Llama,
    SkeletonHorse,
    ZombieHorse,
}

/// Stats for a horse-like entity, governing movement and survivability.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorseStats {
    /// Movement speed in blocks per second.
    pub speed: f32,
    /// Jump strength (higher values = higher jumps).
    pub jump_strength: f32,
    /// Maximum health points.
    pub health: f32,
}

/// Minimum and maximum stat boundaries used during generation.
const MIN_SPEED: f32 = 4.74;
const MAX_SPEED: f32 = 14.23;
const MIN_JUMP: f32 = 0.4;
const MAX_JUMP: f32 = 1.0;
const MIN_HEALTH: f32 = 15.0;
const MAX_HEALTH: f32 = 30.0;

/// Generate random horse stats from a deterministic `seed`.
///
/// Each stat component uses a different portion of the seed to produce a value
/// within Minecraft-like ranges:
/// - **speed**: 4.74 -- 14.23 blocks/s
/// - **jump_strength**: 0.4 -- 1.0
/// - **health**: 15.0 -- 30.0 HP
pub fn random_stats(seed: u64) -> HorseStats {
    // Simple deterministic hash — different multipliers per stat to decorrelate.
    let speed_frac = hash_frac(seed, 0x9E37_79B9_7F4A_7C15);
    let jump_frac = hash_frac(seed, 0x6C62_272E_07BB_0142);
    let health_frac = hash_frac(seed, 0x517C_C1B7_2722_0A95);

    HorseStats {
        speed: lerp(MIN_SPEED, MAX_SPEED, speed_frac),
        jump_strength: lerp(MIN_JUMP, MAX_JUMP, jump_frac),
        health: lerp(MIN_HEALTH, MAX_HEALTH, health_frac),
    }
}

/// Map `seed` to a fraction in `[0.0, 1.0)` using a simple hash with the given
/// `salt` to decorrelate outputs.
fn hash_frac(seed: u64, salt: u64) -> f32 {
    let mut h = seed.wrapping_mul(salt);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    (h & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
}

/// Linear interpolation between `a` and `b` by `t` (clamped to `[0, 1]`).
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

/// Inventory attached to a horse-like entity.
///
/// * `saddle` -- whether a saddle is equipped.
/// * `armor` -- optional horse armor item ID.
/// * `chest` -- whether a chest is attached (donkey / llama only).
/// * `slots` -- up to 15 item slots when a chest is present.
#[derive(Debug, Clone, PartialEq)]
pub struct HorseInventory {
    pub saddle: bool,
    pub armor: Option<u16>,
    pub chest: bool,
    pub slots: Vec<Option<(u16, u8)>>,
}

impl HorseInventory {
    /// Maximum number of chest slots available on a donkey/llama.
    pub const MAX_CHEST_SLOTS: usize = 15;

    /// Create an empty inventory with no saddle, no armor, and no chest.
    pub fn new() -> Self {
        Self {
            saddle: false,
            armor: None,
            chest: false,
            slots: Vec::new(),
        }
    }

    /// Attach a chest, allocating `slot_count` inventory slots (capped at
    /// [`MAX_CHEST_SLOTS`]).
    pub fn attach_chest(&mut self, slot_count: usize) {
        let count = slot_count.min(Self::MAX_CHEST_SLOTS);
        self.chest = true;
        self.slots = vec![None; count];
    }
}

impl Default for HorseInventory {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Horse entity
// ---------------------------------------------------------------------------

/// A rideable horse-like entity.
#[derive(Debug, Clone)]
pub struct Horse {
    pub variant: HorseVariant,
    pub stats: HorseStats,
    pub inventory: HorseInventory,
    /// Entity id of the rider, if any.
    pub rider: Option<u64>,
}

impl Horse {
    /// Create a new horse with the given variant and stats.
    pub fn new(variant: HorseVariant, stats: HorseStats) -> Self {
        Self {
            variant,
            stats,
            inventory: HorseInventory::new(),
            rider: None,
        }
    }

    /// Mount a player (by entity id) onto this horse.
    ///
    /// Returns `false` if the horse already has a rider or has no saddle
    /// (undead horses are exempt from the saddle requirement).
    pub fn mount(&mut self, entity_id: u64) -> bool {
        if self.rider.is_some() {
            return false;
        }
        let needs_saddle = !matches!(
            self.variant,
            HorseVariant::SkeletonHorse | HorseVariant::ZombieHorse
        );
        if needs_saddle && !self.inventory.saddle {
            return false;
        }
        self.rider = Some(entity_id);
        true
    }

    /// Dismount the current rider, returning their entity id if one was riding.
    pub fn dismount(&mut self) -> Option<u64> {
        self.rider.take()
    }
}

// ---------------------------------------------------------------------------
// Breeding
// ---------------------------------------------------------------------------

/// Breed two horses, producing child stats that average each parent's stats
/// with a random offset derived from `seed`.
///
/// The child stat for each attribute is:
///
/// ```text
/// child = (parent1 + parent2) / 2 + noise
/// ```
///
/// where `noise` is in `[-1.0, 1.0]` (scaled per attribute) and the result
/// is clamped to the valid stat range.
pub fn breed(parent1: &HorseStats, parent2: &HorseStats, seed: u64) -> HorseStats {
    let noise = random_stats(seed);

    // Noise contribution: map random stats back to [-1, 1] range.
    let speed_noise = ((noise.speed - MIN_SPEED) / (MAX_SPEED - MIN_SPEED)) * 2.0 - 1.0;
    let jump_noise =
        ((noise.jump_strength - MIN_JUMP) / (MAX_JUMP - MIN_JUMP)) * 2.0 - 1.0;
    let health_noise =
        ((noise.health - MIN_HEALTH) / (MAX_HEALTH - MIN_HEALTH)) * 2.0 - 1.0;

    let raw_speed = (parent1.speed + parent2.speed) / 2.0 + speed_noise;
    let raw_jump = (parent1.jump_strength + parent2.jump_strength) / 2.0 + jump_noise;
    let raw_health = (parent1.health + parent2.health) / 2.0 + health_noise;

    HorseStats {
        speed: raw_speed.clamp(MIN_SPEED, MAX_SPEED),
        jump_strength: raw_jump.clamp(MIN_JUMP, MAX_JUMP),
        health: raw_health.clamp(MIN_HEALTH, MAX_HEALTH),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- random_stats -------------------------------------------------------

    #[test]
    fn random_stats_within_valid_ranges() {
        for seed in 0..100 {
            let stats = random_stats(seed);
            assert!(
                stats.speed >= MIN_SPEED && stats.speed <= MAX_SPEED,
                "seed {seed}: speed {} out of [{MIN_SPEED}, {MAX_SPEED}]",
                stats.speed,
            );
            assert!(
                stats.jump_strength >= MIN_JUMP && stats.jump_strength <= MAX_JUMP,
                "seed {seed}: jump {} out of [{MIN_JUMP}, {MAX_JUMP}]",
                stats.jump_strength,
            );
            assert!(
                stats.health >= MIN_HEALTH && stats.health <= MAX_HEALTH,
                "seed {seed}: health {} out of [{MIN_HEALTH}, {MAX_HEALTH}]",
                stats.health,
            );
        }
    }

    #[test]
    fn random_stats_deterministic() {
        let a = random_stats(42);
        let b = random_stats(42);
        assert_eq!(a, b, "same seed should produce identical stats");
    }

    #[test]
    fn different_seeds_produce_different_stats() {
        let a = random_stats(0);
        let b = random_stats(1);
        // Extremely unlikely (but technically possible) for all three components
        // to match — good enough for a unit test.
        assert!(
            a.speed != b.speed || a.jump_strength != b.jump_strength || a.health != b.health,
            "different seeds should (almost certainly) differ",
        );
    }

    // -- mount / dismount ---------------------------------------------------

    #[test]
    fn mount_succeeds_with_saddle() {
        let stats = random_stats(0);
        let mut horse = Horse::new(HorseVariant::Horse, stats);
        horse.inventory.saddle = true;
        assert!(horse.mount(1));
        assert_eq!(horse.rider, Some(1));
    }

    #[test]
    fn mount_fails_without_saddle() {
        let stats = random_stats(0);
        let mut horse = Horse::new(HorseVariant::Horse, stats);
        assert!(!horse.mount(1));
        assert_eq!(horse.rider, None);
    }

    #[test]
    fn mount_fails_when_occupied() {
        let stats = random_stats(0);
        let mut horse = Horse::new(HorseVariant::Horse, stats);
        horse.inventory.saddle = true;
        assert!(horse.mount(1));
        assert!(!horse.mount(2));
        assert_eq!(horse.rider, Some(1));
    }

    #[test]
    fn skeleton_horse_mounts_without_saddle() {
        let stats = random_stats(0);
        let mut horse = Horse::new(HorseVariant::SkeletonHorse, stats);
        assert!(horse.mount(5));
        assert_eq!(horse.rider, Some(5));
    }

    #[test]
    fn zombie_horse_mounts_without_saddle() {
        let stats = random_stats(0);
        let mut horse = Horse::new(HorseVariant::ZombieHorse, stats);
        assert!(horse.mount(7));
        assert_eq!(horse.rider, Some(7));
    }

    #[test]
    fn dismount_returns_rider() {
        let stats = random_stats(0);
        let mut horse = Horse::new(HorseVariant::Donkey, stats);
        horse.inventory.saddle = true;
        horse.mount(10);
        assert_eq!(horse.dismount(), Some(10));
        assert_eq!(horse.rider, None);
    }

    #[test]
    fn dismount_returns_none_when_empty() {
        let stats = random_stats(0);
        let mut horse = Horse::new(HorseVariant::Mule, stats);
        assert_eq!(horse.dismount(), None);
    }

    // -- breeding -----------------------------------------------------------

    #[test]
    fn breed_child_stats_within_valid_ranges() {
        let p1 = HorseStats {
            speed: 10.0,
            jump_strength: 0.7,
            health: 22.0,
        };
        let p2 = HorseStats {
            speed: 12.0,
            jump_strength: 0.9,
            health: 28.0,
        };
        for seed in 0..100 {
            let child = breed(&p1, &p2, seed);
            assert!(
                child.speed >= MIN_SPEED && child.speed <= MAX_SPEED,
                "seed {seed}: child speed {} out of range",
                child.speed,
            );
            assert!(
                child.jump_strength >= MIN_JUMP && child.jump_strength <= MAX_JUMP,
                "seed {seed}: child jump {} out of range",
                child.jump_strength,
            );
            assert!(
                child.health >= MIN_HEALTH && child.health <= MAX_HEALTH,
                "seed {seed}: child health {} out of range",
                child.health,
            );
        }
    }

    #[test]
    fn breed_is_deterministic() {
        let p1 = random_stats(100);
        let p2 = random_stats(200);
        let a = breed(&p1, &p2, 42);
        let b = breed(&p1, &p2, 42);
        assert_eq!(a, b, "same parents and seed should yield identical child");
    }

    // -- inventory ----------------------------------------------------------

    #[test]
    fn attach_chest_allocates_slots() {
        let mut inv = HorseInventory::new();
        assert!(!inv.chest);
        assert!(inv.slots.is_empty());

        inv.attach_chest(5);
        assert!(inv.chest);
        assert_eq!(inv.slots.len(), 5);
    }

    #[test]
    fn attach_chest_capped_at_max() {
        let mut inv = HorseInventory::new();
        inv.attach_chest(100);
        assert_eq!(inv.slots.len(), HorseInventory::MAX_CHEST_SLOTS);
    }

    // -- variant equality ---------------------------------------------------

    #[test]
    fn horse_variants_are_distinct() {
        let variants = [
            HorseVariant::Horse,
            HorseVariant::Donkey,
            HorseVariant::Mule,
            HorseVariant::Llama,
            HorseVariant::SkeletonHorse,
            HorseVariant::ZombieHorse,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }
}
