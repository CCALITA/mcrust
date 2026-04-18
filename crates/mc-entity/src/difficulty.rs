/// Difficulty levels for the Minecraft game, controlling mob behavior,
/// hunger mechanics, and environmental hazards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    /// Returns the damage multiplier applied to mob attacks.
    /// Peaceful: 0.0, Easy: 0.5, Normal: 1.0, Hard: 1.5
    pub fn mob_damage_multiplier(&self) -> f32 {
        match self {
            Difficulty::Peaceful => 0.0,
            Difficulty::Easy => 0.5,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.5,
        }
    }

    /// Returns the rate at which hunger drains.
    /// Peaceful: 0.0, Easy: 0.5, Normal: 1.0, Hard: 1.5
    pub fn hunger_drain_rate(&self) -> f32 {
        match self {
            Difficulty::Peaceful => 0.0,
            Difficulty::Easy => 0.5,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.5,
        }
    }

    /// Returns the mob spawn rate multiplier.
    /// Peaceful: 0.0, Easy: 0.7, Normal: 1.0, Hard: 1.5
    pub fn mob_spawn_rate(&self) -> f32 {
        match self {
            Difficulty::Peaceful => 0.0,
            Difficulty::Easy => 0.7,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.5,
        }
    }

    /// Returns whether the player can starve to death (only on Hard).
    pub fn can_starve_to_death(&self) -> bool {
        matches!(self, Difficulty::Hard)
    }

    /// Returns whether zombies can break doors (only on Hard).
    pub fn zombies_break_doors(&self) -> bool {
        matches!(self, Difficulty::Hard)
    }

    /// Returns the chance that a mob spawns wearing armor.
    /// Peaceful: 0.0, Easy: 0.0, Normal: 0.15, Hard: 0.9
    pub fn mob_armor_chance(&self) -> f32 {
        match self {
            Difficulty::Peaceful => 0.0,
            Difficulty::Easy => 0.0,
            Difficulty::Normal => 0.15,
            Difficulty::Hard => 0.9,
        }
    }

    /// Returns whether hostile mobs can spawn (everything except Peaceful).
    pub fn hostile_mobs_spawn(&self) -> bool {
        !matches!(self, Difficulty::Peaceful)
    }

    /// Returns whether natural health regeneration is active (always true).
    pub fn natural_regen(&self) -> bool {
        true
    }

    /// Returns whether fire spreads to adjacent blocks (everything except Peaceful).
    pub fn fire_spreads(&self) -> bool {
        !matches!(self, Difficulty::Peaceful)
    }

    /// Parses a difficulty from a case-insensitive string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "peaceful" => Some(Difficulty::Peaceful),
            "easy" => Some(Difficulty::Easy),
            "normal" => Some(Difficulty::Normal),
            "hard" => Some(Difficulty::Hard),
            _ => None,
        }
    }
}

/// Calculates regional difficulty based on play time, chunk inhabited time, and moon phase.
///
/// Returns a value clamped to \[0.75, 1.5\].
///
/// - `play_time_ticks`: total world time in ticks
/// - `inhabited_time`: how long the local chunk has been inhabited (ticks)
/// - `moon_phase`: current moon phase (0..7)
pub fn regional_difficulty(play_time_ticks: u64, inhabited_time: u64, moon_phase: u8) -> f32 {
    let base = 0.75_f32;

    // Play-time bonus: linearly scales up to 0.25 over 3 in-game days (72 000 ticks)
    let play_bonus = (play_time_ticks as f32 / 72_000.0).min(1.0) * 0.25;

    // Inhabited-time bonus: linearly scales up to 0.25 over 50 hours of real time
    // (50 h * 60 min * 60 s * 20 ticks = 3 600 000 ticks)
    let inhabited_bonus = (inhabited_time as f32 / 3_600_000.0).min(1.0) * 0.25;

    // Moon phase bonus: full moon (phase 0) gives the maximum 0.25 bonus
    let moon_bonus = (1.0 - (moon_phase.min(7) as f32 / 7.0)) * 0.25;

    let total = base + play_bonus + inhabited_bonus + moon_bonus;
    total.clamp(0.75, 1.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── mob_damage_multiplier ───────────────────────────────────────────

    #[test]
    fn peaceful_mob_damage_is_zero() {
        assert!((Difficulty::Peaceful.mob_damage_multiplier() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_mob_damage_is_half() {
        assert!((Difficulty::Easy.mob_damage_multiplier() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_mob_damage_is_one() {
        assert!((Difficulty::Normal.mob_damage_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hard_mob_damage_is_one_point_five() {
        assert!((Difficulty::Hard.mob_damage_multiplier() - 1.5).abs() < f32::EPSILON);
    }

    // ── hunger_drain_rate ───────────────────────────────────────────────

    #[test]
    fn peaceful_hunger_drain_is_zero() {
        assert!((Difficulty::Peaceful.hunger_drain_rate() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_hunger_drain_is_half() {
        assert!((Difficulty::Easy.hunger_drain_rate() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_hunger_drain_is_one() {
        assert!((Difficulty::Normal.hunger_drain_rate() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hard_hunger_drain_is_one_point_five() {
        assert!((Difficulty::Hard.hunger_drain_rate() - 1.5).abs() < f32::EPSILON);
    }

    // ── mob_spawn_rate ──────────────────────────────────────────────────

    #[test]
    fn peaceful_mob_spawn_rate_is_zero() {
        assert!((Difficulty::Peaceful.mob_spawn_rate() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_mob_spawn_rate_is_point_seven() {
        assert!((Difficulty::Easy.mob_spawn_rate() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_mob_spawn_rate_is_one() {
        assert!((Difficulty::Normal.mob_spawn_rate() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hard_mob_spawn_rate_is_one_point_five() {
        assert!((Difficulty::Hard.mob_spawn_rate() - 1.5).abs() < f32::EPSILON);
    }

    // ── can_starve_to_death ─────────────────────────────────────────────

    #[test]
    fn only_hard_can_starve_to_death() {
        assert!(!Difficulty::Peaceful.can_starve_to_death());
        assert!(!Difficulty::Easy.can_starve_to_death());
        assert!(!Difficulty::Normal.can_starve_to_death());
        assert!(Difficulty::Hard.can_starve_to_death());
    }

    // ── zombies_break_doors ─────────────────────────────────────────────

    #[test]
    fn only_hard_zombies_break_doors() {
        assert!(!Difficulty::Peaceful.zombies_break_doors());
        assert!(!Difficulty::Easy.zombies_break_doors());
        assert!(!Difficulty::Normal.zombies_break_doors());
        assert!(Difficulty::Hard.zombies_break_doors());
    }

    // ── mob_armor_chance ────────────────────────────────────────────────

    #[test]
    fn peaceful_mob_armor_chance_is_zero() {
        assert!((Difficulty::Peaceful.mob_armor_chance() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_mob_armor_chance_is_zero() {
        assert!((Difficulty::Easy.mob_armor_chance() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_mob_armor_chance_is_point_fifteen() {
        assert!((Difficulty::Normal.mob_armor_chance() - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn hard_mob_armor_chance_is_point_nine() {
        assert!((Difficulty::Hard.mob_armor_chance() - 0.9).abs() < f32::EPSILON);
    }

    // ── hostile_mobs_spawn ──────────────────────────────────────────────

    #[test]
    fn peaceful_no_hostile_mobs() {
        assert!(!Difficulty::Peaceful.hostile_mobs_spawn());
    }

    #[test]
    fn non_peaceful_hostile_mobs_spawn() {
        assert!(Difficulty::Easy.hostile_mobs_spawn());
        assert!(Difficulty::Normal.hostile_mobs_spawn());
        assert!(Difficulty::Hard.hostile_mobs_spawn());
    }

    // ── natural_regen ───────────────────────────────────────────────────

    #[test]
    fn natural_regen_always_true() {
        assert!(Difficulty::Peaceful.natural_regen());
        assert!(Difficulty::Easy.natural_regen());
        assert!(Difficulty::Normal.natural_regen());
        assert!(Difficulty::Hard.natural_regen());
    }

    // ── fire_spreads ────────────────────────────────────────────────────

    #[test]
    fn peaceful_fire_does_not_spread() {
        assert!(!Difficulty::Peaceful.fire_spreads());
    }

    #[test]
    fn non_peaceful_fire_spreads() {
        assert!(Difficulty::Easy.fire_spreads());
        assert!(Difficulty::Normal.fire_spreads());
        assert!(Difficulty::Hard.fire_spreads());
    }

    // ── regional_difficulty ─────────────────────────────────────────────

    #[test]
    fn regional_difficulty_base_at_zero() {
        // With zero play time, zero inhabited time, and full moon (phase 0),
        // moon bonus is 0.25, so total = 0.75 + 0.0 + 0.0 + 0.25 = 1.0
        let rd = regional_difficulty(0, 0, 0);
        assert!((rd - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn regional_difficulty_minimum_clamp() {
        // Moon phase 7 gives zero moon bonus -> 0.75 + 0 + 0 + 0 = 0.75
        let rd = regional_difficulty(0, 0, 7);
        assert!((rd - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn regional_difficulty_maximum_clamp() {
        // Max play time, max inhabited time, full moon -> clamped to 1.5
        let rd = regional_difficulty(1_000_000, 10_000_000, 0);
        assert!((rd - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn regional_difficulty_always_within_bounds() {
        for play in [0, 1000, 72_000, 500_000] {
            for inhabited in [0, 1_000_000, 3_600_000, 10_000_000] {
                for phase in 0..=7 {
                    let rd = regional_difficulty(play, inhabited, phase);
                    assert!(
                        (0.75..=1.5).contains(&rd),
                        "out of bounds: rd={rd} for play={play}, inhabited={inhabited}, phase={phase}"
                    );
                }
            }
        }
    }

    // ── from_str ────────────────────────────────────────────────────────

    #[test]
    fn from_str_lowercase() {
        assert_eq!(Difficulty::from_str("peaceful"), Some(Difficulty::Peaceful));
        assert_eq!(Difficulty::from_str("easy"), Some(Difficulty::Easy));
        assert_eq!(Difficulty::from_str("normal"), Some(Difficulty::Normal));
        assert_eq!(Difficulty::from_str("hard"), Some(Difficulty::Hard));
    }

    #[test]
    fn from_str_mixed_case() {
        assert_eq!(Difficulty::from_str("Peaceful"), Some(Difficulty::Peaceful));
        assert_eq!(Difficulty::from_str("EASY"), Some(Difficulty::Easy));
        assert_eq!(Difficulty::from_str("Normal"), Some(Difficulty::Normal));
        assert_eq!(Difficulty::from_str("HARD"), Some(Difficulty::Hard));
    }

    #[test]
    fn from_str_invalid() {
        assert_eq!(Difficulty::from_str(""), None);
        assert_eq!(Difficulty::from_str("medium"), None);
        assert_eq!(Difficulty::from_str("hardcore"), None);
    }
}
