//! Achievement progress tracking system.
//!
//! Provides [`AchievementProgress`] for individual achievement state and
//! [`AchievementTracker`] for managing a collection of achievements with
//! recent-unlock history.

/// The progress state of a single achievement.
#[derive(Debug, Clone, PartialEq)]
pub struct AchievementProgress {
    /// Unique identifier for this achievement.
    pub id: String,
    /// Current progress count.
    pub current: u32,
    /// Progress count required to unlock.
    pub required: u32,
    /// Whether this achievement has been unlocked.
    pub unlocked: bool,
}

impl AchievementProgress {
    /// Create a new achievement with zero progress.
    pub fn new(id: String, required: u32) -> Self {
        Self {
            id,
            current: 0,
            required,
            unlocked: false,
        }
    }
}

/// A tracker that manages a collection of achievements and recent unlocks.
#[derive(Debug, Clone)]
pub struct AchievementTracker {
    /// All tracked achievements.
    pub progress: Vec<AchievementProgress>,
    /// IDs of recently unlocked achievements (newest last).
    pub recent_unlocks: Vec<String>,
    /// Maximum number of recent unlocks to retain.
    pub max_recent: usize,
}

impl AchievementTracker {
    /// Create an empty tracker with `max_recent` set to `10`.
    pub fn new() -> Self {
        Self {
            progress: Vec::new(),
            recent_unlocks: Vec::new(),
            max_recent: 10,
        }
    }
}

impl Default for AchievementTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Add a new achievement to the tracker.
pub fn add_achievement(tracker: &mut AchievementTracker, id: String, required: u32) {
    tracker.progress.push(AchievementProgress::new(id, required));
}

/// Increment progress on the achievement identified by `id`.
///
/// Returns `Some(id)` if the achievement was newly unlocked by this increment,
/// `None` otherwise (not found, already unlocked, or not yet complete).
pub fn track_progress(tracker: &mut AchievementTracker, id: &str, increment: u32) -> Option<String> {
    let achievement = tracker.progress.iter_mut().find(|a| a.id == id)?;

    if achievement.unlocked {
        return None;
    }

    achievement.current = achievement.current.saturating_add(increment);

    if achievement.current >= achievement.required {
        achievement.unlocked = true;

        let unlocked_id = achievement.id.clone();
        tracker.recent_unlocks.push(unlocked_id.clone());

        // Trim oldest entries when exceeding max_recent.
        while tracker.recent_unlocks.len() > tracker.max_recent {
            tracker.recent_unlocks.remove(0);
        }

        return Some(unlocked_id);
    }

    None
}

/// Return the fraction of progress toward unlocking (0.0 to 1.0).
///
/// Returns `1.0` if the achievement is already unlocked.
pub fn unlock_progress_fraction(p: &AchievementProgress) -> f32 {
    if p.unlocked {
        return 1.0;
    }
    if p.required == 0 {
        return 1.0;
    }
    (p.current as f32) / (p.required as f32)
}

/// Return a slice of the most recent `n` unlocked achievement IDs.
///
/// If fewer than `n` are available, the entire recent-unlock list is returned.
pub fn recent_unlocks(tracker: &AchievementTracker, n: usize) -> &[String] {
    let len = tracker.recent_unlocks.len();
    if n >= len {
        &tracker.recent_unlocks
    } else {
        &tracker.recent_unlocks[len - n..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AchievementProgress construction ────────────────────────────

    #[test]
    fn new_achievement_has_zero_progress() {
        let a = AchievementProgress::new("mine_diamond".into(), 10);
        assert_eq!(a.id, "mine_diamond");
        assert_eq!(a.current, 0);
        assert_eq!(a.required, 10);
        assert!(!a.unlocked);
    }

    // ── AchievementTracker construction ─────────────────────────────

    #[test]
    fn new_tracker_is_empty_with_default_max_recent() {
        let t = AchievementTracker::new();
        assert!(t.progress.is_empty());
        assert!(t.recent_unlocks.is_empty());
        assert_eq!(t.max_recent, 10);
    }

    #[test]
    fn default_tracker_matches_new() {
        let a = AchievementTracker::new();
        let b = AchievementTracker::default();
        assert_eq!(a.max_recent, b.max_recent);
        assert_eq!(a.progress.len(), b.progress.len());
    }

    // ── add_achievement ─────────────────────────────────────────────

    #[test]
    fn add_achievement_appends_to_progress() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "kill_dragon".into(), 1);
        assert_eq!(t.progress.len(), 1);
        assert_eq!(t.progress[0].id, "kill_dragon");
        assert_eq!(t.progress[0].required, 1);
    }

    #[test]
    fn add_multiple_achievements() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "a".into(), 5);
        add_achievement(&mut t, "b".into(), 10);
        assert_eq!(t.progress.len(), 2);
    }

    // ── track_progress ──────────────────────────────────────────────

    #[test]
    fn track_progress_increments_current() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "mine".into(), 10);
        track_progress(&mut t, "mine", 3);
        assert_eq!(t.progress[0].current, 3);
    }

    #[test]
    fn track_progress_returns_none_when_not_yet_complete() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "mine".into(), 10);
        let result = track_progress(&mut t, "mine", 3);
        assert!(result.is_none());
    }

    #[test]
    fn track_progress_unlocks_when_reaching_required() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "mine".into(), 5);
        let result = track_progress(&mut t, "mine", 5);
        assert_eq!(result, Some("mine".into()));
        assert!(t.progress[0].unlocked);
    }

    #[test]
    fn track_progress_unlocks_when_exceeding_required() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "mine".into(), 5);
        let result = track_progress(&mut t, "mine", 10);
        assert_eq!(result, Some("mine".into()));
        assert!(t.progress[0].unlocked);
    }

    #[test]
    fn track_progress_returns_none_if_already_unlocked() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "mine".into(), 1);
        track_progress(&mut t, "mine", 1);
        let result = track_progress(&mut t, "mine", 1);
        assert!(result.is_none());
    }

    #[test]
    fn track_progress_returns_none_for_unknown_id() {
        let mut t = AchievementTracker::new();
        let result = track_progress(&mut t, "nonexistent", 1);
        assert!(result.is_none());
    }

    #[test]
    fn track_progress_pushes_to_recent_unlocks() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "mine".into(), 1);
        track_progress(&mut t, "mine", 1);
        assert_eq!(t.recent_unlocks, vec!["mine"]);
    }

    #[test]
    fn recent_unlocks_trims_to_max_recent() {
        let mut t = AchievementTracker::new();
        t.max_recent = 3;
        for i in 0..5 {
            let id = format!("a{i}");
            add_achievement(&mut t, id.clone(), 1);
            track_progress(&mut t, &id, 1);
        }
        assert_eq!(t.recent_unlocks.len(), 3);
        assert_eq!(t.recent_unlocks, vec!["a2", "a3", "a4"]);
    }

    #[test]
    fn track_progress_saturates_instead_of_overflow() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "big".into(), u32::MAX);
        track_progress(&mut t, "big", u32::MAX - 1);
        track_progress(&mut t, "big", u32::MAX - 1);
        // Should not panic; current saturates at u32::MAX.
        assert!(t.progress[0].current >= u32::MAX - 1);
    }

    // ── unlock_progress_fraction ────────────────────────────────────

    #[test]
    fn fraction_zero_when_no_progress() {
        let a = AchievementProgress::new("x".into(), 10);
        assert!((unlock_progress_fraction(&a)).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_half_when_halfway() {
        let mut a = AchievementProgress::new("x".into(), 10);
        a.current = 5;
        assert!((unlock_progress_fraction(&a) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_one_when_unlocked() {
        let mut a = AchievementProgress::new("x".into(), 10);
        a.unlocked = true;
        assert!((unlock_progress_fraction(&a) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_one_when_required_is_zero() {
        let a = AchievementProgress::new("x".into(), 0);
        assert!((unlock_progress_fraction(&a) - 1.0).abs() < f32::EPSILON);
    }

    // ── recent_unlocks ──────────────────────────────────────────────

    #[test]
    fn recent_unlocks_returns_all_when_n_is_large() {
        let mut t = AchievementTracker::new();
        add_achievement(&mut t, "a".into(), 1);
        add_achievement(&mut t, "b".into(), 1);
        track_progress(&mut t, "a", 1);
        track_progress(&mut t, "b", 1);
        let result = recent_unlocks(&t, 100);
        assert_eq!(result, &["a", "b"]);
    }

    #[test]
    fn recent_unlocks_returns_last_n() {
        let mut t = AchievementTracker::new();
        for i in 0..5 {
            let id = format!("a{i}");
            add_achievement(&mut t, id.clone(), 1);
            track_progress(&mut t, &id, 1);
        }
        let result = recent_unlocks(&t, 2);
        assert_eq!(result, &["a3", "a4"]);
    }

    #[test]
    fn recent_unlocks_returns_empty_when_none_unlocked() {
        let t = AchievementTracker::new();
        let result = recent_unlocks(&t, 5);
        assert!(result.is_empty());
    }
}
