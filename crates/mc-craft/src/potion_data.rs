// ── Potion types and effects ────────────────────────────────────────────────

/// All potion types available in the brewing system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PotionType {
    Speed,
    Slowness,
    JumpBoost,
    Strength,
    Weakness,
    Healing,
    Harming,
    Regeneration,
    Poison,
    FireResistance,
    NightVision,
    Invisibility,
    WaterBreathing,
    SlowFalling,
    Luck,
}

/// A single potion effect with type, remaining duration, and amplifier level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionEffect {
    pub potion_type: PotionType,
    pub duration_ticks: u32,
    pub amplifier: u8,
}

// ── Status effect manager ──────────────────────────────────────────────────

/// Manages active potion effects on an entity (typically a player).
///
/// Handles effect stacking (replacing weaker effects of the same type),
/// tick-based expiry, and derived stat multipliers.
#[derive(Debug, Clone, Default)]
pub struct StatusEffectManager {
    active_effects: Vec<PotionEffect>,
}

impl StatusEffectManager {
    /// Create a manager with no active effects.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_effects: Vec::new(),
        }
    }

    /// Add a potion effect. If an effect of the same type already exists,
    /// it is replaced only when the new effect is strictly stronger
    /// (higher amplifier, or equal amplifier with longer duration).
    pub fn add_effect(&mut self, effect: PotionEffect) {
        if let Some(existing) = self
            .active_effects
            .iter_mut()
            .find(|e| e.potion_type == effect.potion_type)
        {
            let dominated = effect.amplifier > existing.amplifier
                || (effect.amplifier == existing.amplifier
                    && effect.duration_ticks > existing.duration_ticks);
            if dominated {
                *existing = effect;
            }
        } else {
            self.active_effects.push(effect);
        }
    }

    /// Advance all active effects by one tick.
    /// Effects whose duration reaches zero are removed.
    pub fn tick(&mut self) {
        for effect in &mut self.active_effects {
            effect.duration_ticks = effect.duration_ticks.saturating_sub(1);
        }
        self.active_effects.retain(|e| e.duration_ticks > 0);
    }

    /// Look up an active effect by type.
    #[must_use]
    pub fn has_effect(&self, t: PotionType) -> Option<&PotionEffect> {
        self.active_effects.iter().find(|e| e.potion_type == t)
    }

    /// Compute the speed multiplier from active Speed / Slowness effects.
    ///
    /// Base is `1.0`. Each Speed amplifier level adds `+0.2`,
    /// each Slowness amplifier level subtracts `0.15`.
    /// Amplifier `0` counts as level 1.
    #[must_use]
    pub fn get_speed_multiplier(&self) -> f32 {
        let mut multiplier: f32 = 1.0;

        if let Some(speed) = self.has_effect(PotionType::Speed) {
            let level = f32::from(speed.amplifier) + 1.0;
            multiplier += 0.2 * level;
        }
        if let Some(slow) = self.has_effect(PotionType::Slowness) {
            let level = f32::from(slow.amplifier) + 1.0;
            multiplier -= 0.15 * level;
        }

        multiplier
    }

    /// Compute the damage multiplier from active Strength / Weakness effects.
    ///
    /// Each Strength amplifier level adds `+3.0` damage,
    /// each Weakness amplifier level subtracts `4.0` damage.
    /// Amplifier `0` counts as level 1.
    #[must_use]
    pub fn get_damage_multiplier(&self) -> f32 {
        let mut multiplier: f32 = 0.0;

        if let Some(str_eff) = self.has_effect(PotionType::Strength) {
            let level = f32::from(str_eff.amplifier) + 1.0;
            multiplier += 3.0 * level;
        }
        if let Some(weak_eff) = self.has_effect(PotionType::Weakness) {
            let level = f32::from(weak_eff.amplifier) + 1.0;
            multiplier -= 4.0 * level;
        }

        multiplier
    }

    /// Whether the entity is currently invisible.
    #[must_use]
    pub fn is_invisible(&self) -> bool {
        self.has_effect(PotionType::Invisibility).is_some()
    }

    /// Whether the entity is currently fire-resistant.
    #[must_use]
    pub fn is_fire_resistant(&self) -> bool {
        self.has_effect(PotionType::FireResistance).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Status effect stacking / expiry ────────────────────────────────

    #[test]
    fn add_effect_stores_new_type() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 0,
        });
        assert!(mgr.has_effect(PotionType::Speed).is_some());
        assert!(mgr.has_effect(PotionType::Strength).is_none());
    }

    #[test]
    fn stronger_effect_replaces_weaker() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 200,
            amplifier: 0,
        });
        // Stronger amplifier should replace.
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 1,
        });
        let eff = mgr.has_effect(PotionType::Speed).expect("should exist");
        assert_eq!(eff.amplifier, 1);
        assert_eq!(eff.duration_ticks, 100);
    }

    #[test]
    fn weaker_effect_does_not_replace_stronger() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 200,
            amplifier: 1,
        });
        // Lower amplifier should NOT replace.
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 300,
            amplifier: 0,
        });
        let eff = mgr.has_effect(PotionType::Speed).expect("should exist");
        assert_eq!(eff.amplifier, 1);
        assert_eq!(eff.duration_ticks, 200);
    }

    #[test]
    fn equal_amplifier_longer_duration_replaces() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Regeneration,
            duration_ticks: 100,
            amplifier: 0,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Regeneration,
            duration_ticks: 200,
            amplifier: 0,
        });
        let eff = mgr
            .has_effect(PotionType::Regeneration)
            .expect("should exist");
        assert_eq!(eff.duration_ticks, 200);
    }

    #[test]
    fn tick_decrements_duration() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 5,
            amplifier: 0,
        });
        mgr.tick();
        let eff = mgr.has_effect(PotionType::Speed).expect("should exist");
        assert_eq!(eff.duration_ticks, 4);
    }

    #[test]
    fn expired_effect_removed_after_tick() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 1,
            amplifier: 0,
        });
        mgr.tick();
        assert!(mgr.has_effect(PotionType::Speed).is_none());
    }

    // ── Speed / damage multipliers ─────────────────────────────────────

    #[test]
    fn speed_multiplier_no_effects() {
        let mgr = StatusEffectManager::new();
        assert!((mgr.get_speed_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_with_speed_1() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 1.0 + 0.2 * 1 = 1.2
        assert!((mgr.get_speed_multiplier() - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_with_slowness() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Slowness,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 1.0 - 0.15 * 1 = 0.85
        assert!((mgr.get_speed_multiplier() - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_combined() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 1, // level 2
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Slowness,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 1.0 + 0.2*2 - 0.15*1 = 1.25
        assert!((mgr.get_speed_multiplier() - 1.25).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_no_effects() {
        let mgr = StatusEffectManager::new();
        assert!((mgr.get_damage_multiplier() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_with_strength() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Strength,
            duration_ticks: 100,
            amplifier: 0,
        });
        // 3.0 * 1 = 3.0
        assert!((mgr.get_damage_multiplier() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_with_weakness() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Weakness,
            duration_ticks: 100,
            amplifier: 0,
        });
        // -4.0 * 1 = -4.0
        assert!((mgr.get_damage_multiplier() - (-4.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_combined() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Strength,
            duration_ticks: 100,
            amplifier: 1, // level 2
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Weakness,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 3.0*2 - 4.0*1 = 2.0
        assert!((mgr.get_damage_multiplier() - 2.0).abs() < f32::EPSILON);
    }

    // ── Boolean helpers ────────────────────────────────────────────────

    #[test]
    fn is_invisible_when_effect_active() {
        let mut mgr = StatusEffectManager::new();
        assert!(!mgr.is_invisible());
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Invisibility,
            duration_ticks: 100,
            amplifier: 0,
        });
        assert!(mgr.is_invisible());
    }

    #[test]
    fn is_fire_resistant_when_effect_active() {
        let mut mgr = StatusEffectManager::new();
        assert!(!mgr.is_fire_resistant());
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::FireResistance,
            duration_ticks: 100,
            amplifier: 0,
        });
        assert!(mgr.is_fire_resistant());
    }

    // ── Effect replacement edge cases ──────────────────────────────────

    #[test]
    fn equal_amplifier_shorter_duration_does_not_replace() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Poison,
            duration_ticks: 200,
            amplifier: 0,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Poison,
            duration_ticks: 100,
            amplifier: 0,
        });
        let eff = mgr.has_effect(PotionType::Poison).expect("should exist");
        assert_eq!(eff.duration_ticks, 200);
    }

    #[test]
    fn multiple_different_effects_coexist() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 0,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Strength,
            duration_ticks: 200,
            amplifier: 1,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Invisibility,
            duration_ticks: 50,
            amplifier: 0,
        });
        assert!(mgr.has_effect(PotionType::Speed).is_some());
        assert!(mgr.has_effect(PotionType::Strength).is_some());
        assert!(mgr.has_effect(PotionType::Invisibility).is_some());
    }
}
