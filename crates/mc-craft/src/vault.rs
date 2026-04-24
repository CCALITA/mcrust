//! Vault loot pool and interaction logic.
//!
//! Implements the trial vault reward system: players use a trial key to open
//! a vault once, which ejects randomized loot based on weighted entries.
//! Each vault tracks which players have already claimed their reward.

use crate::item_ids::{ITEM_ARROW, ITEM_BOOK, ITEM_DIAMOND, ITEM_GOLD_INGOT, ITEM_IRON_INGOT};

// ── Item IDs not yet in item_ids.rs ──────────────────────────────────────
const ITEM_EMERALD: u16 = 850;
const ITEM_EXP_BOTTLE: u16 = 851;
const ITEM_TOTEM: u16 = 852;
const ITEM_NETHERITE_SCRAP: u16 = 853;
const ITEM_TRIAL_KEY: u16 = 854;
const ITEM_ENCHANTED_GOLDEN_APPLE: u16 = 855;
const ITEM_DIAMOND_BLOCK: u16 = 856;
const ITEM_NETHERITE_INGOT: u16 = 901;

/// A single weighted entry in a vault loot table.
#[derive(Debug, Clone)]
pub struct VaultLootEntry {
    pub item_id: u16,
    pub min: u8,
    pub max: u8,
    pub weight: f32,
}

/// A complete loot table for a vault, with an ominous flag.
#[derive(Debug, Clone)]
pub struct VaultLoot {
    pub entries: Vec<VaultLootEntry>,
    pub ominous: bool,
}

/// Tracks vault interaction state: ejection animation and per-player usage.
#[derive(Debug, Clone)]
pub struct VaultState {
    pub ejecting: bool,
    pub eject_timer: f32,
    pub players_used: Vec<u64>,
}

impl VaultState {
    pub fn new() -> Self {
        Self {
            ejecting: false,
            eject_timer: 0.0,
            players_used: Vec::new(),
        }
    }
}

/// Returns the default vault loot table (normal difficulty).
pub fn default_vault_loot() -> VaultLoot {
    VaultLoot {
        entries: vec![
            VaultLootEntry { item_id: ITEM_DIAMOND, min: 1, max: 3, weight: 1.0 },
            VaultLootEntry { item_id: ITEM_EMERALD, min: 1, max: 3, weight: 1.0 },
            VaultLootEntry { item_id: ITEM_IRON_INGOT, min: 2, max: 5, weight: 2.0 },
            VaultLootEntry { item_id: ITEM_GOLD_INGOT, min: 1, max: 3, weight: 1.5 },
            VaultLootEntry { item_id: ITEM_ARROW, min: 4, max: 8, weight: 1.0 },
            VaultLootEntry { item_id: ITEM_BOOK, min: 1, max: 1, weight: 0.3 },
            VaultLootEntry { item_id: ITEM_EXP_BOTTLE, min: 2, max: 4, weight: 0.8 },
            VaultLootEntry { item_id: ITEM_TOTEM, min: 1, max: 1, weight: 0.05 },
        ],
        ominous: false,
    }
}

/// Returns the ominous vault loot table (better drops, includes netherite).
pub fn default_ominous_loot() -> VaultLoot {
    VaultLoot {
        entries: vec![
            VaultLootEntry { item_id: ITEM_DIAMOND, min: 2, max: 6, weight: 1.0 },
            VaultLootEntry { item_id: ITEM_EMERALD, min: 4, max: 8, weight: 1.0 },
            VaultLootEntry { item_id: ITEM_NETHERITE_SCRAP, min: 1, max: 2, weight: 0.5 },
            VaultLootEntry { item_id: ITEM_NETHERITE_INGOT, min: 1, max: 1, weight: 0.2 },
            VaultLootEntry { item_id: ITEM_GOLD_INGOT, min: 4, max: 8, weight: 1.5 },
            VaultLootEntry { item_id: ITEM_ENCHANTED_GOLDEN_APPLE, min: 1, max: 1, weight: 0.1 },
            VaultLootEntry { item_id: ITEM_DIAMOND_BLOCK, min: 1, max: 2, weight: 0.3 },
            VaultLootEntry { item_id: ITEM_TOTEM, min: 1, max: 1, weight: 0.15 },
        ],
        ominous: true,
    }
}

/// Attempt to open a vault for the given player with a trial key.
///
/// Returns a deterministic seed on success, or an error if the player
/// has already used this vault or is holding the wrong key item.
pub fn try_open_vault(
    state: &mut VaultState,
    player_id: u64,
    key_item: u16,
) -> Result<u32, &'static str> {
    if key_item != ITEM_TRIAL_KEY {
        return Err("wrong key item");
    }
    if state.players_used.contains(&player_id) {
        return Err("player already used this vault");
    }
    state.players_used.push(player_id);
    state.ejecting = true;
    state.eject_timer = vault_eject_duration();

    // Deterministic seed from player id
    let seed = (player_id.wrapping_mul(2654435761)) as u32;
    Ok(seed)
}

/// Roll loot from a vault loot table using a seed.
///
/// Picks 3-5 entries by weight, returning `(item_id, count)` pairs.
pub fn roll_loot(loot: &VaultLoot, seed: u64) -> Vec<(u16, u8)> {
    if loot.entries.is_empty() {
        return Vec::new();
    }

    let total_weight: f32 = loot.entries.iter().map(|e| e.weight).sum();
    if total_weight <= 0.0 {
        return Vec::new();
    }

    // Simple deterministic PRNG (xorshift-based)
    let mut rng_state = seed;
    let mut next_u64 = || -> u64 {
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        rng_state
    };

    // Pick 3-5 entries
    let num_picks = 3 + (next_u64() % 3) as usize; // 3, 4, or 5

    let mut results = Vec::with_capacity(num_picks);
    for _ in 0..num_picks {
        let roll = (next_u64() % 10000) as f32 / 10000.0 * total_weight;
        let mut cumulative = 0.0_f32;
        let mut picked = &loot.entries[0];
        for entry in &loot.entries {
            cumulative += entry.weight;
            if roll < cumulative {
                picked = entry;
                break;
            }
        }

        // Determine count within [min, max]
        let range = (picked.max as u64).saturating_sub(picked.min as u64) + 1;
        let count = picked.min + (next_u64() % range) as u8;
        results.push((picked.item_id, count));
    }

    results
}

/// Duration in seconds for the vault eject animation.
pub fn vault_eject_duration() -> f32 {
    3.0
}

/// Cooldown in ticks before a vault can be activated again (24000 = one Minecraft day).
pub fn vault_cooldown_ticks() -> u32 {
    24000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_state_new_defaults() {
        let state = VaultState::new();
        assert!(!state.ejecting);
        assert_eq!(state.eject_timer, 0.0);
        assert!(state.players_used.is_empty());
    }

    #[test]
    fn default_vault_loot_has_eight_entries() {
        let loot = default_vault_loot();
        assert_eq!(loot.entries.len(), 8);
        assert!(!loot.ominous);
    }

    #[test]
    fn default_ominous_loot_is_ominous() {
        let loot = default_ominous_loot();
        assert!(loot.ominous);
        assert!(!loot.entries.is_empty());
        // Should contain netherite_scrap
        assert!(loot.entries.iter().any(|e| e.item_id == ITEM_NETHERITE_SCRAP));
    }

    #[test]
    fn try_open_vault_success() {
        let mut state = VaultState::new();
        let result = try_open_vault(&mut state, 42, ITEM_TRIAL_KEY);
        assert!(result.is_ok());
        assert!(state.ejecting);
        assert_eq!(state.eject_timer, 3.0);
        assert_eq!(state.players_used, vec![42]);
    }

    #[test]
    fn try_open_vault_wrong_key() {
        let mut state = VaultState::new();
        let result = try_open_vault(&mut state, 42, 999);
        assert_eq!(result, Err("wrong key item"));
        assert!(!state.ejecting);
        assert!(state.players_used.is_empty());
    }

    #[test]
    fn try_open_vault_already_used() {
        let mut state = VaultState::new();
        let _ = try_open_vault(&mut state, 42, ITEM_TRIAL_KEY);
        let result = try_open_vault(&mut state, 42, ITEM_TRIAL_KEY);
        assert_eq!(result, Err("player already used this vault"));
    }

    #[test]
    fn try_open_vault_different_players() {
        let mut state = VaultState::new();
        assert!(try_open_vault(&mut state, 1, ITEM_TRIAL_KEY).is_ok());
        assert!(try_open_vault(&mut state, 2, ITEM_TRIAL_KEY).is_ok());
        assert_eq!(state.players_used.len(), 2);
    }

    #[test]
    fn roll_loot_returns_three_to_five_items() {
        let loot = default_vault_loot();
        for seed in 0..20 {
            let items = roll_loot(&loot, seed + 1);
            assert!(
                (3..=5).contains(&items.len()),
                "seed {} gave {} items",
                seed,
                items.len()
            );
        }
    }

    #[test]
    fn roll_loot_respects_min_max() {
        let loot = default_vault_loot();
        for seed in 1..50 {
            let items = roll_loot(&loot, seed);
            for (item_id, count) in &items {
                let entry = loot.entries.iter().find(|e| e.item_id == *item_id).unwrap();
                assert!(
                    *count >= entry.min && *count <= entry.max,
                    "item {} count {} outside [{}, {}]",
                    item_id,
                    count,
                    entry.min,
                    entry.max,
                );
            }
        }
    }

    #[test]
    fn roll_loot_empty_table() {
        let loot = VaultLoot {
            entries: Vec::new(),
            ominous: false,
        };
        let items = roll_loot(&loot, 42);
        assert!(items.is_empty());
    }

    #[test]
    fn roll_loot_deterministic() {
        let loot = default_vault_loot();
        let a = roll_loot(&loot, 12345);
        let b = roll_loot(&loot, 12345);
        assert_eq!(a, b);
    }

    #[test]
    fn roll_loot_different_seeds_vary() {
        let loot = default_vault_loot();
        let a = roll_loot(&loot, 1);
        let b = roll_loot(&loot, 2);
        // Different seeds should produce different results (extremely likely)
        assert_ne!(a, b);
    }

    #[test]
    fn vault_eject_duration_is_three() {
        assert_eq!(vault_eject_duration(), 3.0);
    }

    #[test]
    fn vault_cooldown_is_24000() {
        assert_eq!(vault_cooldown_ticks(), 24000);
    }

    #[test]
    fn seed_is_deterministic_for_player() {
        let mut state1 = VaultState::new();
        let mut state2 = VaultState::new();
        let seed1 = try_open_vault(&mut state1, 42, ITEM_TRIAL_KEY).unwrap();
        let seed2 = try_open_vault(&mut state2, 42, ITEM_TRIAL_KEY).unwrap();
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn ominous_loot_weights_are_positive() {
        let loot = default_ominous_loot();
        for entry in &loot.entries {
            assert!(entry.weight > 0.0, "entry {} has non-positive weight", entry.item_id);
        }
    }
}
