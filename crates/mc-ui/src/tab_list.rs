//! Tab player list overlay.
//!
//! Provides [`TabList`] and [`TabPlayer`] for rendering the player list shown
//! when the player holds the Tab key, plus helpers for latency colors,
//! gamemode prefixes, and screen layout.

/// A single entry in the tab player list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabPlayer {
    pub name: String,
    pub latency_ms: u32,
    /// 0 = Survival, 1 = Creative, 2 = Adventure, 3 = Spectator.
    pub gamemode: u8,
    pub has_hat: bool,
}

/// Tab overlay containing the active set of players plus server metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabList {
    pub players: Vec<TabPlayer>,
    pub server_name: String,
    pub max_visible: usize,
}

impl TabList {
    /// Create an empty tab list for `server_name` showing up to 20 players.
    pub fn new(server_name: String) -> Self {
        Self {
            players: Vec::new(),
            server_name,
            max_visible: 20,
        }
    }

    /// Add `player` to the list. Duplicates by name are allowed; callers should
    /// remove first if they want to upsert.
    pub fn add_player(&mut self, player: TabPlayer) {
        self.players.push(player);
    }

    /// Remove the entry with the given `name`, if present.
    pub fn remove_player(&mut self, name: &str) {
        self.players.retain(|p| p.name != name);
    }

    /// Sort the player list alphabetically by name (case-sensitive).
    pub fn sort_alphabetically(&mut self) {
        self.players.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Color (RGB, 0..=1) for a latency badge in milliseconds.
///
/// - `< 100` ms -> green
/// - `< 300` ms -> yellow
/// - `< 500` ms -> orange
/// - otherwise -> red
pub fn latency_color(ms: u32) -> [f32; 3] {
    if ms < 100 {
        [0.0, 1.0, 0.0]
    } else if ms < 300 {
        [1.0, 1.0, 0.0]
    } else if ms < 500 {
        [1.0, 0.5, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    }
}

/// Short bracketed prefix for a gamemode value.
///
/// 0=Survival "", 1=Creative "[C]", 2=Adventure "[A]", 3=Spectator "[S]".
/// Unknown values return an empty string.
pub fn gamemode_prefix(gamemode: u8) -> &'static str {
    match gamemode {
        1 => "[C]",
        2 => "[A]",
        3 => "[S]",
        _ => "",
    }
}

/// Width of a single tab row in pixels.
pub const TAB_ROW_WIDTH: f32 = 200.0;
/// Height of a single tab row in pixels.
pub const TAB_ROW_HEIGHT: f32 = 16.0;
/// Vertical spacing between rows.
pub const TAB_ROW_GAP: f32 = 2.0;
/// Top margin from the top of the screen.
pub const TAB_TOP_MARGIN: f32 = 20.0;

/// Compute (x, y) screen positions for each row in the tab list.
///
/// Rows are stacked vertically and centered horizontally near the top of the
/// screen. Returned coordinates are the top-left corners of each row.
pub fn tab_layout(players: &[TabPlayer], screen_w: f32, screen_h: f32) -> Vec<(f32, f32)> {
    let _ = screen_h;
    let x = (screen_w - TAB_ROW_WIDTH) * 0.5;
    let row_step = TAB_ROW_HEIGHT + TAB_ROW_GAP;
    (0..players.len())
        .map(|i| (x, TAB_TOP_MARGIN + (i as f32) * row_step))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn player(name: &str, latency: u32, gamemode: u8) -> TabPlayer {
        TabPlayer {
            name: name.to_string(),
            latency_ms: latency,
            gamemode,
            has_hat: false,
        }
    }

    #[test]
    fn new_tab_list_defaults() {
        let list = TabList::new("MyServer".to_string());
        assert_eq!(list.server_name, "MyServer");
        assert_eq!(list.max_visible, 20);
        assert!(list.players.is_empty());
    }

    #[test]
    fn add_player_appends_entry() {
        let mut list = TabList::new("S".to_string());
        list.add_player(player("Alice", 50, 0));
        list.add_player(player("Bob", 80, 1));
        assert_eq!(list.players.len(), 2);
        assert_eq!(list.players[0].name, "Alice");
        assert_eq!(list.players[1].name, "Bob");
    }

    #[test]
    fn remove_player_removes_matching_name() {
        let mut list = TabList::new("S".to_string());
        list.add_player(player("Alice", 50, 0));
        list.add_player(player("Bob", 80, 1));
        list.remove_player("Alice");
        assert_eq!(list.players.len(), 1);
        assert_eq!(list.players[0].name, "Bob");
    }

    #[test]
    fn remove_player_noop_for_missing_name() {
        let mut list = TabList::new("S".to_string());
        list.add_player(player("Alice", 50, 0));
        list.remove_player("Nobody");
        assert_eq!(list.players.len(), 1);
    }

    #[test]
    fn sort_alphabetically_orders_by_name() {
        let mut list = TabList::new("S".to_string());
        list.add_player(player("Charlie", 0, 0));
        list.add_player(player("Alice", 0, 0));
        list.add_player(player("Bob", 0, 0));
        list.sort_alphabetically();
        let names: Vec<&str> = list.players.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
    }

    #[test]
    fn latency_color_green_below_100() {
        assert_eq!(latency_color(0), [0.0, 1.0, 0.0]);
        assert_eq!(latency_color(99), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn latency_color_yellow_at_100_boundary() {
        assert_eq!(latency_color(100), [1.0, 1.0, 0.0]);
        assert_eq!(latency_color(299), [1.0, 1.0, 0.0]);
    }

    #[test]
    fn latency_color_orange_at_300_boundary() {
        assert_eq!(latency_color(300), [1.0, 0.5, 0.0]);
        assert_eq!(latency_color(499), [1.0, 0.5, 0.0]);
    }

    #[test]
    fn latency_color_red_at_500_boundary() {
        assert_eq!(latency_color(500), [1.0, 0.0, 0.0]);
        assert_eq!(latency_color(10_000), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn gamemode_prefix_known_values() {
        assert_eq!(gamemode_prefix(0), "");
        assert_eq!(gamemode_prefix(1), "[C]");
        assert_eq!(gamemode_prefix(2), "[A]");
        assert_eq!(gamemode_prefix(3), "[S]");
    }

    #[test]
    fn gamemode_prefix_unknown_returns_empty() {
        assert_eq!(gamemode_prefix(4), "");
        assert_eq!(gamemode_prefix(255), "");
    }

    #[test]
    fn tab_layout_empty_players_returns_empty() {
        let positions = tab_layout(&[], 800.0, 600.0);
        assert!(positions.is_empty());
    }

    #[test]
    fn tab_layout_centers_horizontally() {
        let players = vec![player("A", 0, 0)];
        let positions = tab_layout(&players, 800.0, 600.0);
        assert_eq!(positions.len(), 1);
        let expected_x = (800.0 - TAB_ROW_WIDTH) * 0.5;
        assert_eq!(positions[0].0, expected_x);
        assert_eq!(positions[0].1, TAB_TOP_MARGIN);
    }

    #[test]
    fn tab_layout_stacks_rows_vertically() {
        let players = vec![player("A", 0, 0), player("B", 0, 0), player("C", 0, 0)];
        let positions = tab_layout(&players, 800.0, 600.0);
        assert_eq!(positions.len(), 3);
        let step = TAB_ROW_HEIGHT + TAB_ROW_GAP;
        assert_eq!(positions[0].1, TAB_TOP_MARGIN);
        assert_eq!(positions[1].1, TAB_TOP_MARGIN + step);
        assert_eq!(positions[2].1, TAB_TOP_MARGIN + 2.0 * step);
        // All rows share the same x coordinate.
        assert_eq!(positions[0].0, positions[1].0);
        assert_eq!(positions[1].0, positions[2].0);
    }

    #[test]
    fn tab_player_fields_round_trip() {
        let p = TabPlayer {
            name: "Steve".to_string(),
            latency_ms: 42,
            gamemode: 1,
            has_hat: true,
        };
        assert_eq!(p.name, "Steve");
        assert_eq!(p.latency_ms, 42);
        assert_eq!(p.gamemode, 1);
        assert!(p.has_hat);
    }
}
