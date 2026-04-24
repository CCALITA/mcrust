//! Key binding remap system.
//!
//! Maps [`GameAction`]s to keyboard key codes (`u32`). Provides defaults,
//! rebinding, lookup, and conflict detection so a settings UI can build a
//! fully customizable control scheme.

use std::collections::HashMap;

/// Every user-facing game action that can be triggered from the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    MoveForward,
    MoveBack,
    MoveLeft,
    MoveRight,
    Jump,
    Sneak,
    Sprint,
    AttackUse,
    BreakBlock,
    PlaceBlock,
    OpenInventory,
    OpenChat,
    Pause,
    ToggleHud,
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    Hotbar6,
    Hotbar7,
    Hotbar8,
    Hotbar9,
    ToggleCamera,
    SwapHands,
    DropItem,
    PickBlock,
    SaveGame,
}

impl GameAction {
    /// Human-readable label for settings UIs.
    pub fn name(&self) -> &'static str {
        match self {
            GameAction::MoveForward => "Move Forward",
            GameAction::MoveBack => "Move Back",
            GameAction::MoveLeft => "Move Left",
            GameAction::MoveRight => "Move Right",
            GameAction::Jump => "Jump",
            GameAction::Sneak => "Sneak",
            GameAction::Sprint => "Sprint",
            GameAction::AttackUse => "Attack / Use",
            GameAction::BreakBlock => "Break Block",
            GameAction::PlaceBlock => "Place Block",
            GameAction::OpenInventory => "Open Inventory",
            GameAction::OpenChat => "Open Chat",
            GameAction::Pause => "Pause",
            GameAction::ToggleHud => "Toggle HUD",
            GameAction::Hotbar1 => "Hotbar 1",
            GameAction::Hotbar2 => "Hotbar 2",
            GameAction::Hotbar3 => "Hotbar 3",
            GameAction::Hotbar4 => "Hotbar 4",
            GameAction::Hotbar5 => "Hotbar 5",
            GameAction::Hotbar6 => "Hotbar 6",
            GameAction::Hotbar7 => "Hotbar 7",
            GameAction::Hotbar8 => "Hotbar 8",
            GameAction::Hotbar9 => "Hotbar 9",
            GameAction::ToggleCamera => "Toggle Camera",
            GameAction::SwapHands => "Swap Hands",
            GameAction::DropItem => "Drop Item",
            GameAction::PickBlock => "Pick Block",
            GameAction::SaveGame => "Save Game",
        }
    }
}

/// Full mapping from [`GameAction`] to key code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBindings {
    pub bindings: HashMap<GameAction, u32>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        // ASCII / common key code values. These match the typical Windows VK /
        // ASCII uppercase mapping and are fine as stable defaults across
        // backends; callers are free to rebind to whatever their input layer
        // uses.
        let pairs: [(GameAction, u32); 28] = [
            (GameAction::MoveForward, 87),   // W
            (GameAction::MoveBack, 83),      // S
            (GameAction::MoveLeft, 65),      // A
            (GameAction::MoveRight, 68),     // D
            (GameAction::Jump, 32),          // Space
            (GameAction::Sneak, 16),         // Shift
            (GameAction::Sprint, 17),        // Ctrl
            (GameAction::AttackUse, 256),    // Left mouse (custom code)
            (GameAction::BreakBlock, 257),   // Right mouse (custom code)
            (GameAction::PlaceBlock, 258),   // Middle mouse (custom code)
            (GameAction::OpenInventory, 69), // E
            (GameAction::OpenChat, 84),      // T
            (GameAction::Pause, 27),         // Esc
            (GameAction::ToggleHud, 112),    // F1
            (GameAction::Hotbar1, 49),       // 1
            (GameAction::Hotbar2, 50),       // 2
            (GameAction::Hotbar3, 51),       // 3
            (GameAction::Hotbar4, 52),       // 4
            (GameAction::Hotbar5, 53),       // 5
            (GameAction::Hotbar6, 54),       // 6
            (GameAction::Hotbar7, 55),       // 7
            (GameAction::Hotbar8, 56),       // 8
            (GameAction::Hotbar9, 57),       // 9
            (GameAction::ToggleCamera, 116), // F5
            (GameAction::SwapHands, 70),     // F
            (GameAction::DropItem, 81),      // Q
            (GameAction::PickBlock, 259),    // Middle click alt (custom)
            (GameAction::SaveGame, 113),     // F2
        ];
        let mut bindings = HashMap::with_capacity(pairs.len());
        for (action, key) in pairs {
            bindings.insert(action, key);
        }
        Self { bindings }
    }
}

/// Rebind `action` to `key`. Overwrites any existing mapping for the action.
pub fn rebind(b: &mut KeyBindings, action: GameAction, key: u32) {
    b.bindings.insert(action, key);
}

/// Look up the key currently bound to `action`.
pub fn get_key(b: &KeyBindings, action: GameAction) -> Option<u32> {
    b.bindings.get(&action).copied()
}

/// Find every unordered pair of distinct actions that share the same key.
///
/// Each conflicting pair is returned exactly once. Ordering within the pair
/// and across the returned vector is not guaranteed, but the output is stable
/// for a given input with respect to content.
pub fn find_conflicts(b: &KeyBindings) -> Vec<(GameAction, GameAction)> {
    // Group actions by key code.
    let mut by_key: HashMap<u32, Vec<GameAction>> = HashMap::new();
    for (&action, &key) in &b.bindings {
        by_key.entry(key).or_default().push(action);
    }

    let mut conflicts = Vec::new();
    for (_, mut actions) in by_key {
        if actions.len() < 2 {
            continue;
        }
        // Sort for deterministic pair ordering within a single key group.
        actions.sort_by_key(|a| format!("{a:?}"));
        for i in 0..actions.len() {
            for j in (i + 1)..actions.len() {
                conflicts.push((actions[i], actions[j]));
            }
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_all_28_actions_bound() {
        let b = KeyBindings::default();
        assert_eq!(b.bindings.len(), 28);
        // Spot check a few canonical defaults.
        assert_eq!(get_key(&b, GameAction::MoveForward), Some(87));
        assert_eq!(get_key(&b, GameAction::MoveBack), Some(83));
        assert_eq!(get_key(&b, GameAction::MoveLeft), Some(65));
        assert_eq!(get_key(&b, GameAction::MoveRight), Some(68));
        assert_eq!(get_key(&b, GameAction::Jump), Some(32));
        assert_eq!(get_key(&b, GameAction::Hotbar9), Some(57));
    }

    #[test]
    fn name_returns_human_readable_labels() {
        assert_eq!(GameAction::MoveForward.name(), "Move Forward");
        assert_eq!(GameAction::OpenInventory.name(), "Open Inventory");
        assert_eq!(GameAction::Hotbar1.name(), "Hotbar 1");
        assert_eq!(GameAction::SaveGame.name(), "Save Game");
    }

    #[test]
    fn rebind_overwrites_existing_binding() {
        let mut b = KeyBindings::default();
        rebind(&mut b, GameAction::Jump, 999);
        assert_eq!(get_key(&b, GameAction::Jump), Some(999));
    }

    #[test]
    fn get_key_returns_none_for_unbound_action_on_empty_map() {
        let b = KeyBindings {
            bindings: HashMap::new(),
        };
        assert_eq!(get_key(&b, GameAction::Jump), None);
    }

    #[test]
    fn find_conflicts_detects_shared_key() {
        let mut b = KeyBindings::default();
        // Bind Sprint onto the same key as MoveForward (W = 87).
        rebind(&mut b, GameAction::Sprint, 87);
        let conflicts = find_conflicts(&b);
        assert_eq!(conflicts.len(), 1);
        let (a, c) = conflicts[0];
        let pair = [a, c];
        assert!(pair.contains(&GameAction::MoveForward));
        assert!(pair.contains(&GameAction::Sprint));
    }

    #[test]
    fn find_conflicts_empty_when_all_unique() {
        let b = KeyBindings::default();
        assert!(find_conflicts(&b).is_empty());
    }

    #[test]
    fn find_conflicts_reports_every_pair_in_three_way_collision() {
        let mut b = KeyBindings {
            bindings: HashMap::new(),
        };
        rebind(&mut b, GameAction::Jump, 42);
        rebind(&mut b, GameAction::Sneak, 42);
        rebind(&mut b, GameAction::Sprint, 42);
        let conflicts = find_conflicts(&b);
        // 3 actions sharing a key => C(3,2) = 3 pairs.
        assert_eq!(conflicts.len(), 3);
    }

    #[test]
    fn bindings_clone_is_independent() {
        let b = KeyBindings::default();
        let mut c = b.clone();
        rebind(&mut c, GameAction::Jump, 1);
        assert_ne!(get_key(&b, GameAction::Jump), get_key(&c, GameAction::Jump));
    }
}
