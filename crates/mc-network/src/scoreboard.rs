use std::collections::HashMap;

/// A scoreboard objective that tracks a particular criterion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Objective {
    pub name: String,
    pub display_name: String,
    pub criteria: String,
}

/// A single score entry for a player under some objective.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreEntry {
    pub player: String,
    pub score: i32,
}

/// Display slot where an objective can be shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplaySlot {
    BelowName,
    Sidebar,
    PlayerList,
}

/// A team with members, color, and name affixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    pub name: String,
    pub color: u8,
    pub prefix: String,
    pub suffix: String,
    pub members: Vec<String>,
}

/// The full scoreboard state for a game server.
#[derive(Debug, Clone)]
pub struct ScoreboardState {
    pub objectives: Vec<Objective>,
    pub scores: HashMap<String, Vec<ScoreEntry>>,
    pub display: HashMap<DisplaySlot, String>,
    pub teams: Vec<Team>,
}

impl ScoreboardState {
    /// Creates an empty scoreboard state.
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            scores: HashMap::new(),
            display: HashMap::new(),
            teams: Vec::new(),
        }
    }

    /// Adds an objective to the scoreboard.
    ///
    /// Returns `false` if an objective with the same name already exists.
    pub fn add_objective(&mut self, objective: Objective) -> bool {
        if self.objectives.iter().any(|o| o.name == objective.name) {
            return false;
        }
        self.objectives.push(objective);
        true
    }

    /// Sets a player's score for the given objective.
    ///
    /// Creates the objective's score list if it does not exist yet.
    pub fn set_score(&mut self, objective: &str, player: &str, score: i32) {
        let entries = self
            .scores
            .entry(objective.to_string())
            .or_insert_with(Vec::new);

        if let Some(entry) = entries.iter_mut().find(|e| e.player == player) {
            entry.score = score;
        } else {
            entries.push(ScoreEntry {
                player: player.to_string(),
                score,
            });
        }
    }

    /// Removes a player from all objectives' score lists.
    ///
    /// Returns the number of objectives the player was removed from.
    pub fn remove_player(&mut self, player: &str) -> usize {
        let mut removed = 0;
        for entries in self.scores.values_mut() {
            let before = entries.len();
            entries.retain(|e| e.player != player);
            if entries.len() < before {
                removed += 1;
            }
        }
        removed
    }

    /// Returns the objective name assigned to a display slot, if any.
    pub fn get_display(&self, slot: DisplaySlot) -> Option<&str> {
        self.display.get(&slot).map(|s| s.as_str())
    }
}

impl Default for ScoreboardState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_objective(name: &str) -> Objective {
        Objective {
            name: name.to_string(),
            display_name: format!("{name} Display"),
            criteria: "dummy".to_string(),
        }
    }

    #[test]
    fn adds_objective_successfully() {
        let mut state = ScoreboardState::new();
        assert!(state.add_objective(sample_objective("kills")));
        assert_eq!(state.objectives.len(), 1);
        assert_eq!(state.objectives[0].name, "kills");
    }

    #[test]
    fn rejects_duplicate_objective() {
        let mut state = ScoreboardState::new();
        assert!(state.add_objective(sample_objective("kills")));
        assert!(!state.add_objective(sample_objective("kills")));
        assert_eq!(state.objectives.len(), 1);
    }

    #[test]
    fn sets_and_updates_score() {
        let mut state = ScoreboardState::new();
        state.set_score("kills", "Alice", 5);
        state.set_score("kills", "Alice", 10);

        let entries = &state.scores["kills"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].score, 10);
    }

    #[test]
    fn removes_player_from_all_objectives() {
        let mut state = ScoreboardState::new();
        state.set_score("kills", "Alice", 5);
        state.set_score("deaths", "Alice", 2);
        state.set_score("kills", "Bob", 3);

        let removed = state.remove_player("Alice");
        assert_eq!(removed, 2);
        assert!(state.scores["kills"]
            .iter()
            .all(|e| e.player != "Alice"));
        assert!(state.scores["deaths"].is_empty());
    }

    #[test]
    fn gets_display_slot() {
        let mut state = ScoreboardState::new();
        assert!(state.get_display(DisplaySlot::Sidebar).is_none());

        state
            .display
            .insert(DisplaySlot::Sidebar, "kills".to_string());
        assert_eq!(state.get_display(DisplaySlot::Sidebar), Some("kills"));
    }

    #[test]
    fn remove_player_returns_zero_when_absent() {
        let mut state = ScoreboardState::new();
        state.set_score("kills", "Bob", 3);
        assert_eq!(state.remove_player("Alice"), 0);
    }

    #[test]
    fn default_state_is_empty() {
        let state = ScoreboardState::default();
        assert!(state.objectives.is_empty());
        assert!(state.scores.is_empty());
        assert!(state.display.is_empty());
        assert!(state.teams.is_empty());
    }
}
