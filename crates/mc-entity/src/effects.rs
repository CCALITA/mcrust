/// Status effects system for entities (potions, beacons, etc.)

/// All possible status effects matching vanilla Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    Speed,
    Slowness,
    Haste,
    MiningFatigue,
    Strength,
    InstantHealth,
    InstantDamage,
    JumpBoost,
    Nausea,
    Regeneration,
    Resistance,
    FireResistance,
    WaterBreathing,
    Invisibility,
    Blindness,
    NightVision,
    Hunger,
    Weakness,
    Poison,
    Wither,
    Absorption,
    Glowing,
    Levitation,
    SlowFalling,
}

/// An active effect applied to an entity with a remaining duration and potency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveEffect {
    pub effect: StatusEffect,
    pub duration_ticks: u32,
    pub amplifier: u8,
}

/// Manages the collection of active status effects on an entity.
#[derive(Debug, Clone)]
pub struct EffectManager {
    effects: Vec<ActiveEffect>,
}

impl EffectManager {
    /// Creates an empty effect manager.
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Adds an effect. If the entity already has the same effect type, the new
    /// effect replaces the old one only when its amplifier is strictly higher.
    pub fn add_effect(&mut self, effect: ActiveEffect) {
        if let Some(existing) = self
            .effects
            .iter_mut()
            .find(|e| e.effect == effect.effect)
        {
            if effect.amplifier > existing.amplifier {
                *existing = effect;
            }
        } else {
            self.effects.push(effect);
        }
    }

    /// Removes the effect of the given type, if present.
    pub fn remove_effect(&mut self, effect_type: StatusEffect) {
        self.effects.retain(|e| e.effect != effect_type);
    }

    /// Advances all effects by one tick: decrements durations and removes any
    /// that have expired (reached zero).
    pub fn tick(&mut self) {
        for effect in &mut self.effects {
            effect.duration_ticks = effect.duration_ticks.saturating_sub(1);
        }
        self.effects.retain(|e| e.duration_ticks > 0);
    }

    /// Returns `true` if the entity currently has the given effect.
    pub fn has_effect(&self, effect_type: StatusEffect) -> bool {
        self.effects.iter().any(|e| e.effect == effect_type)
    }

    /// Returns the amplifier of the given effect, or `None` if absent.
    pub fn get_amplifier(&self, effect_type: StatusEffect) -> Option<u8> {
        self.effects
            .iter()
            .find(|e| e.effect == effect_type)
            .map(|e| e.amplifier)
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Modifier helpers
// ---------------------------------------------------------------------------

/// Applies the Speed effect modifier: `base * (1.0 + 0.2 * (amplifier + 1))`.
pub fn apply_speed_modifier(base: f32, amplifier: u8) -> f32 {
    base * (1.0 + 0.2 * (amplifier as f32 + 1.0))
}

/// Applies the Slowness effect modifier: `base * (1.0 - 0.15 * (amplifier + 1))`.
pub fn apply_slowness_modifier(base: f32, amplifier: u8) -> f32 {
    base * (1.0 - 0.15 * (amplifier as f32 + 1.0))
}

/// Applies the Strength effect modifier: `base + 3.0 * (amplifier + 1)`.
pub fn apply_strength_modifier(base: f32, amplifier: u8) -> f32 {
    base + 3.0 * (amplifier as f32 + 1.0)
}

/// Applies the Jump Boost effect modifier: `base + 0.1 * (amplifier + 1)`.
pub fn apply_jump_modifier(base: f32, amplifier: u8) -> f32 {
    base + 0.1 * (amplifier as f32 + 1.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove_effect() {
        let mut mgr = EffectManager::new();
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Speed,
            duration_ticks: 100,
            amplifier: 0,
        });

        assert!(mgr.has_effect(StatusEffect::Speed));
        assert_eq!(mgr.get_amplifier(StatusEffect::Speed), Some(0));

        mgr.remove_effect(StatusEffect::Speed);
        assert!(!mgr.has_effect(StatusEffect::Speed));
        assert_eq!(mgr.get_amplifier(StatusEffect::Speed), None);
    }

    #[test]
    fn stronger_replaces_weaker() {
        let mut mgr = EffectManager::new();
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Strength,
            duration_ticks: 200,
            amplifier: 0,
        });
        // Stronger amplifier should replace.
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Strength,
            duration_ticks: 100,
            amplifier: 2,
        });
        assert_eq!(mgr.get_amplifier(StatusEffect::Strength), Some(2));
    }

    #[test]
    fn weaker_does_not_replace_stronger() {
        let mut mgr = EffectManager::new();
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Strength,
            duration_ticks: 200,
            amplifier: 2,
        });
        // Weaker amplifier should NOT replace.
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Strength,
            duration_ticks: 300,
            amplifier: 1,
        });
        assert_eq!(mgr.get_amplifier(StatusEffect::Strength), Some(2));
    }

    #[test]
    fn tick_decrements_duration() {
        let mut mgr = EffectManager::new();
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Regeneration,
            duration_ticks: 3,
            amplifier: 0,
        });

        mgr.tick();
        assert!(mgr.has_effect(StatusEffect::Regeneration));

        mgr.tick();
        assert!(mgr.has_effect(StatusEffect::Regeneration));

        // Third tick brings duration to 0 -> removed.
        mgr.tick();
        assert!(!mgr.has_effect(StatusEffect::Regeneration));
    }

    #[test]
    fn tick_removes_expired_effects_only() {
        let mut mgr = EffectManager::new();
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Speed,
            duration_ticks: 1,
            amplifier: 0,
        });
        mgr.add_effect(ActiveEffect {
            effect: StatusEffect::Invisibility,
            duration_ticks: 5,
            amplifier: 1,
        });

        mgr.tick();
        assert!(!mgr.has_effect(StatusEffect::Speed));
        assert!(mgr.has_effect(StatusEffect::Invisibility));
    }

    // -- modifier tests ------------------------------------------------

    #[test]
    fn speed_modifier_amplifier_0() {
        let result = apply_speed_modifier(1.0, 0);
        assert!((result - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_modifier_amplifier_1() {
        let result = apply_speed_modifier(1.0, 1);
        assert!((result - 1.4).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_modifier_non_unit_base() {
        let result = apply_speed_modifier(5.0, 0);
        assert!((result - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn slowness_modifier_amplifier_0() {
        let result = apply_slowness_modifier(1.0, 0);
        assert!((result - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn slowness_modifier_amplifier_2() {
        let result = apply_slowness_modifier(1.0, 2);
        assert!((result - 0.55).abs() < f32::EPSILON);
    }

    #[test]
    fn strength_modifier_amplifier_0() {
        let result = apply_strength_modifier(1.0, 0);
        assert!((result - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn strength_modifier_amplifier_1() {
        let result = apply_strength_modifier(1.0, 1);
        assert!((result - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn jump_modifier_amplifier_0() {
        let result = apply_jump_modifier(0.42, 0);
        assert!((result - 0.52).abs() < 1e-6);
    }

    #[test]
    fn jump_modifier_amplifier_3() {
        let result = apply_jump_modifier(0.42, 3);
        assert!((result - 0.82).abs() < 1e-6);
    }
}
