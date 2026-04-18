// ── Enchantment ID ─────────────────────────────────────────────────────────

/// All enchantment types available in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnchantmentId {
    // Armor
    Protection,
    FireProtection,
    BlastProtection,
    ProjectileProtection,
    Thorns,
    Respiration,
    AquaAffinity,
    DepthStrider,
    FrostWalker,
    // Weapon
    Sharpness,
    Smite,
    BaneOfArthropods,
    Knockback,
    FireAspect,
    Looting,
    SweepingEdge,
    // Tool
    Efficiency,
    SilkTouch,
    Fortune,
    Unbreaking,
    // Bow
    Power,
    Punch,
    Flame,
    Infinity,
    // General
    Mending,
    CurseOfVanishing,
}

impl EnchantmentId {
    /// All enchantment variants, in declaration order.
    pub const ALL: [EnchantmentId; 26] = [
        EnchantmentId::Protection,
        EnchantmentId::FireProtection,
        EnchantmentId::BlastProtection,
        EnchantmentId::ProjectileProtection,
        EnchantmentId::Thorns,
        EnchantmentId::Respiration,
        EnchantmentId::AquaAffinity,
        EnchantmentId::DepthStrider,
        EnchantmentId::FrostWalker,
        EnchantmentId::Sharpness,
        EnchantmentId::Smite,
        EnchantmentId::BaneOfArthropods,
        EnchantmentId::Knockback,
        EnchantmentId::FireAspect,
        EnchantmentId::Looting,
        EnchantmentId::SweepingEdge,
        EnchantmentId::Efficiency,
        EnchantmentId::SilkTouch,
        EnchantmentId::Fortune,
        EnchantmentId::Unbreaking,
        EnchantmentId::Power,
        EnchantmentId::Punch,
        EnchantmentId::Flame,
        EnchantmentId::Infinity,
        EnchantmentId::Mending,
        EnchantmentId::CurseOfVanishing,
    ];

    /// Get the static properties for this enchantment.
    #[must_use]
    pub fn properties(self) -> &'static EnchantmentProperties {
        &ENCHANTMENT_REGISTRY[self as usize]
    }
}

// ── Enchantment Category ───────────────────────────────────────────────────

/// Broad category used to determine which items an enchantment can apply to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnchantmentCategory {
    Armor,
    Weapon,
    Tool,
    Bow,
    General,
}

// ── Enchantment Properties ─────────────────────────────────────────────────

/// Static data describing a single enchantment kind.
#[derive(Debug, Clone)]
pub struct EnchantmentProperties {
    pub name: &'static str,
    pub max_level: u8,
    pub category: EnchantmentCategory,
    pub incompatible_with: &'static [EnchantmentId],
}

// ── Static registry ────────────────────────────────────────────────────────

pub static ENCHANTMENT_REGISTRY: [EnchantmentProperties; 26] = [
    // Armor
    EnchantmentProperties {
        name: "Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::FireProtection,
            EnchantmentId::BlastProtection,
            EnchantmentId::ProjectileProtection,
        ],
    },
    EnchantmentProperties {
        name: "Fire Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::Protection,
            EnchantmentId::BlastProtection,
            EnchantmentId::ProjectileProtection,
        ],
    },
    EnchantmentProperties {
        name: "Blast Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::Protection,
            EnchantmentId::FireProtection,
            EnchantmentId::ProjectileProtection,
        ],
    },
    EnchantmentProperties {
        name: "Projectile Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::Protection,
            EnchantmentId::FireProtection,
            EnchantmentId::BlastProtection,
        ],
    },
    EnchantmentProperties {
        name: "Thorns",
        max_level: 3,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Respiration",
        max_level: 3,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Aqua Affinity",
        max_level: 1,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Depth Strider",
        max_level: 3,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[EnchantmentId::FrostWalker],
    },
    EnchantmentProperties {
        name: "Frost Walker",
        max_level: 2,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[EnchantmentId::DepthStrider],
    },
    // Weapon
    EnchantmentProperties {
        name: "Sharpness",
        max_level: 5,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[EnchantmentId::Smite, EnchantmentId::BaneOfArthropods],
    },
    EnchantmentProperties {
        name: "Smite",
        max_level: 5,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[EnchantmentId::Sharpness, EnchantmentId::BaneOfArthropods],
    },
    EnchantmentProperties {
        name: "Bane of Arthropods",
        max_level: 5,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[EnchantmentId::Sharpness, EnchantmentId::Smite],
    },
    EnchantmentProperties {
        name: "Knockback",
        max_level: 2,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Fire Aspect",
        max_level: 2,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Looting",
        max_level: 3,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Sweeping Edge",
        max_level: 3,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    // Tool
    EnchantmentProperties {
        name: "Efficiency",
        max_level: 5,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Silk Touch",
        max_level: 1,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[EnchantmentId::Fortune],
    },
    EnchantmentProperties {
        name: "Fortune",
        max_level: 3,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[EnchantmentId::SilkTouch],
    },
    EnchantmentProperties {
        name: "Unbreaking",
        max_level: 3,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[],
    },
    // Bow
    EnchantmentProperties {
        name: "Power",
        max_level: 5,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Punch",
        max_level: 2,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Flame",
        max_level: 1,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Infinity",
        max_level: 1,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[EnchantmentId::Mending],
    },
    // General
    EnchantmentProperties {
        name: "Mending",
        max_level: 1,
        category: EnchantmentCategory::General,
        incompatible_with: &[EnchantmentId::Infinity],
    },
    EnchantmentProperties {
        name: "Curse of Vanishing",
        max_level: 1,
        category: EnchantmentCategory::General,
        incompatible_with: &[],
    },
];

// ── Tests ──────────────────────────────────────────────────────────────────

// ── Enchantment (instance) ─────────────────────────────────────────────────

/// A specific enchantment applied at a given level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enchantment {
    pub id: EnchantmentId,
    pub level: u8,
}

impl Enchantment {
    /// Create an enchantment, clamping the level to the enchantment's max.
    #[must_use]
    pub fn new(id: EnchantmentId, level: u8) -> Self {
        let max = id.properties().max_level;
        Self {
            id,
            level: level.clamp(1, max),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_26_variants() {
        assert_eq!(EnchantmentId::ALL.len(), 26);
    }

    #[test]
    fn properties_lookup_returns_correct_name() {
        assert_eq!(EnchantmentId::Sharpness.properties().name, "Sharpness");
        assert_eq!(
            EnchantmentId::CurseOfVanishing.properties().name,
            "Curse of Vanishing"
        );
    }

    #[test]
    fn max_levels_are_within_1_to_5() {
        for id in &EnchantmentId::ALL {
            let max = id.properties().max_level;
            assert!(
                (1..=5).contains(&max),
                "{:?} has max_level {} outside 1..=5",
                id,
                max
            );
        }
    }

    // ── Enchantment level clamping ─────────────────────────────────────

    #[test]
    fn enchantment_new_clamps_level_to_max() {
        let e = Enchantment::new(EnchantmentId::AquaAffinity, 5);
        assert_eq!(e.level, 1); // max_level is 1
    }

    #[test]
    fn enchantment_new_clamps_level_to_minimum_1() {
        let e = Enchantment::new(EnchantmentId::Sharpness, 0);
        assert_eq!(e.level, 1);
    }

    #[test]
    fn enchantment_new_preserves_valid_level() {
        let e = Enchantment::new(EnchantmentId::Sharpness, 3);
        assert_eq!(e.level, 3);
    }
}
