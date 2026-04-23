//! Scoreboard display formatting for the HUD.
//!
//! Provides [`ScoreboardDisplay`] for tracking objective scores and
//! [`format_sidebar_lines`] / [`sidebar_width_chars`] for rendering the
//! sidebar overlay.

/// Maximum number of entries shown in the sidebar.
pub const MAX_SIDEBAR_ENTRIES: usize = 15;

/// Where a scoreboard objective is displayed on-screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplaySlot {
    /// Right-hand sidebar overlay.
    Sidebar,
    /// Tab-list (player list) header column.
    PlayerList,
    /// Rendered below the player's nametag.
    BelowName,
}

/// A single (name, score) pair in a scoreboard objective.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreEntry {
    pub name: String,
    pub score: i32,
}

/// Tracks scores for one objective and the slot where it should be displayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardDisplay {
    pub objective: String,
    pub entries: Vec<ScoreEntry>,
    pub display_slot: DisplaySlot,
}

impl ScoreboardDisplay {
    /// Create an empty scoreboard for `objective` shown in `slot`.
    pub fn new(objective: String, slot: DisplaySlot) -> Self {
        Self {
            objective,
            entries: Vec::new(),
            display_slot: slot,
        }
    }

    /// Insert or update the score for `name`.
    pub fn set_score(&mut self, name: String, score: i32) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.name == name) {
            entry.score = score;
        } else {
            self.entries.push(ScoreEntry { name, score });
        }
    }

    /// Remove the entry with the given `name`, if present.
    pub fn remove_entry(&mut self, name: &str) {
        self.entries.retain(|e| e.name != name);
    }

    /// Return references to entries sorted by score descending (stable order for ties).
    pub fn sorted_entries(&self) -> Vec<&ScoreEntry> {
        let mut sorted: Vec<&ScoreEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.score.cmp(&a.score));
        sorted
    }
}

/// Format the sidebar as `(name, score)` pairs, sorted descending, capped at
/// [`MAX_SIDEBAR_ENTRIES`].
pub fn format_sidebar_lines(display: &ScoreboardDisplay) -> Vec<(String, i32)> {
    display
        .sorted_entries()
        .into_iter()
        .take(MAX_SIDEBAR_ENTRIES)
        .map(|e| (e.name.clone(), e.score))
        .collect()
}

/// Compute the character width needed to render the sidebar.
///
/// Each line occupies `name.len() + digit_count(score) + 2` characters
/// (the 2 accounts for the space and a separator). Returns the maximum
/// across all entries, or 0 if the slice is empty.
pub fn sidebar_width_chars(entries: &[(String, i32)]) -> usize {
    entries
        .iter()
        .map(|(name, score)| {
            let digits = if *score == 0 {
                1
            } else {
                let abs = (*score as i64).unsigned_abs();
                let digit_count = (abs as f64).log10().floor() as usize + 1;
                if *score < 0 {
                    digit_count + 1 // minus sign
                } else {
                    digit_count
                }
            };
            name.len() + digits + 2
        })
        .max()
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scoreboard_is_empty() {
        let sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        assert_eq!(sb.objective, "kills");
        assert_eq!(sb.display_slot, DisplaySlot::Sidebar);
        assert!(sb.entries.is_empty());
    }

    #[test]
    fn set_score_inserts_new_entry() {
        let mut sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        sb.set_score("Alice".to_string(), 10);
        assert_eq!(sb.entries.len(), 1);
        assert_eq!(sb.entries[0].name, "Alice");
        assert_eq!(sb.entries[0].score, 10);
    }

    #[test]
    fn set_score_updates_existing_entry() {
        let mut sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        sb.set_score("Alice".to_string(), 10);
        sb.set_score("Alice".to_string(), 25);
        assert_eq!(sb.entries.len(), 1);
        assert_eq!(sb.entries[0].score, 25);
    }

    #[test]
    fn remove_entry_removes_matching_name() {
        let mut sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        sb.set_score("Alice".to_string(), 10);
        sb.set_score("Bob".to_string(), 5);
        sb.remove_entry("Alice");
        assert_eq!(sb.entries.len(), 1);
        assert_eq!(sb.entries[0].name, "Bob");
    }

    #[test]
    fn remove_entry_noop_for_missing_name() {
        let mut sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        sb.set_score("Alice".to_string(), 10);
        sb.remove_entry("Charlie");
        assert_eq!(sb.entries.len(), 1);
    }

    #[test]
    fn sorted_entries_returns_descending_order() {
        let mut sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        sb.set_score("Alice".to_string(), 5);
        sb.set_score("Bob".to_string(), 20);
        sb.set_score("Charlie".to_string(), 12);

        let sorted = sb.sorted_entries();
        assert_eq!(sorted[0].name, "Bob");
        assert_eq!(sorted[1].name, "Charlie");
        assert_eq!(sorted[2].name, "Alice");
    }

    #[test]
    fn sorted_entries_empty_scoreboard() {
        let sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        assert!(sb.sorted_entries().is_empty());
    }

    #[test]
    fn format_sidebar_lines_sorted_descending() {
        let mut sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        sb.set_score("Alice".to_string(), 3);
        sb.set_score("Bob".to_string(), 10);

        let lines = format_sidebar_lines(&sb);
        assert_eq!(lines, vec![("Bob".to_string(), 10), ("Alice".to_string(), 3)]);
    }

    #[test]
    fn format_sidebar_lines_caps_at_max_entries() {
        let mut sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        for i in 0..20 {
            sb.set_score(format!("Player{i}"), i);
        }
        let lines = format_sidebar_lines(&sb);
        assert_eq!(lines.len(), MAX_SIDEBAR_ENTRIES);
        // First entry should be the highest score (19).
        assert_eq!(lines[0].1, 19);
    }

    #[test]
    fn format_sidebar_lines_empty_scoreboard() {
        let sb = ScoreboardDisplay::new("kills".to_string(), DisplaySlot::Sidebar);
        assert!(format_sidebar_lines(&sb).is_empty());
    }

    #[test]
    fn sidebar_width_chars_basic() {
        // "Alice" (5) + score 10 (2 digits) + 2 = 9
        // "Bob" (3) + score 5 (1 digit) + 2 = 6
        let entries = vec![("Alice".to_string(), 10), ("Bob".to_string(), 5)];
        assert_eq!(sidebar_width_chars(&entries), 9);
    }

    #[test]
    fn sidebar_width_chars_negative_score() {
        // "X" (1) + score -42 (3 chars: minus + 2 digits) + 2 = 6
        let entries = vec![("X".to_string(), -42)];
        assert_eq!(sidebar_width_chars(&entries), 6);
    }

    #[test]
    fn sidebar_width_chars_zero_score() {
        // "Test" (4) + score 0 (1 digit) + 2 = 7
        let entries = vec![("Test".to_string(), 0)];
        assert_eq!(sidebar_width_chars(&entries), 7);
    }

    #[test]
    fn sidebar_width_chars_empty() {
        let entries: Vec<(String, i32)> = vec![];
        assert_eq!(sidebar_width_chars(&entries), 0);
    }

    #[test]
    fn display_slot_variants_are_distinct() {
        assert_ne!(DisplaySlot::Sidebar, DisplaySlot::PlayerList);
        assert_ne!(DisplaySlot::PlayerList, DisplaySlot::BelowName);
        assert_ne!(DisplaySlot::Sidebar, DisplaySlot::BelowName);
    }

    #[test]
    fn set_score_multiple_entries_and_sort() {
        let mut sb = ScoreboardDisplay::new("obj".to_string(), DisplaySlot::PlayerList);
        sb.set_score("Z".to_string(), 1);
        sb.set_score("A".to_string(), 100);
        sb.set_score("M".to_string(), 50);

        let sorted = sb.sorted_entries();
        let scores: Vec<i32> = sorted.iter().map(|e| e.score).collect();
        assert_eq!(scores, vec![100, 50, 1]);
    }

    #[test]
    fn sidebar_width_chars_large_score() {
        // "P" (1) + score 1000000 (7 digits) + 2 = 10
        let entries = vec![("P".to_string(), 1_000_000)];
        assert_eq!(sidebar_width_chars(&entries), 10);
    }
}
