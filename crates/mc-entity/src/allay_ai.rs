//! Allay mob AI — item collection, delivery, and duplication logic.

/// State for an allay's item-collection AI.
#[derive(Debug, Clone, PartialEq)]
pub struct AllayAiState {
    /// Item ID the allay is searching for, if any.
    pub held_item: Option<u16>,
    /// Position of the note block the allay is bound to.
    pub note_block: Option<(i32, i32, i32)>,
    /// Whether the allay is actively collecting items.
    pub collecting: bool,
}

impl AllayAiState {
    /// Create a new idle allay state.
    pub fn new() -> Self {
        Self {
            held_item: None,
            note_block: None,
            collecting: false,
        }
    }
}

/// Returns the position of the nearest item matching `held` from `nearby`.
///
/// Each entry in `nearby` is `(item_id, [x, y, z])`.
pub fn allay_target_item(held: u16, nearby: &[(u16, [f32; 3])]) -> Option<[f32; 3]> {
    nearby
        .iter()
        .filter(|(id, _)| *id == held)
        .min_by(|(_, a), (_, b)| {
            let dist_a = a[0] * a[0] + a[1] * a[1] + a[2] * a[2];
            let dist_b = b[0] * b[0] + b[1] * b[1] + b[2] * b[2];
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(_, pos)| *pos)
}

/// Returns a normalized direction vector from `pos` toward `noteblock`.
pub fn allay_deliver_direction(pos: [f32; 3], noteblock: (i32, i32, i32)) -> [f32; 3] {
    let dx = noteblock.0 as f32 - pos[0];
    let dy = noteblock.1 as f32 - pos[1];
    let dz = noteblock.2 as f32 - pos[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-6 {
        return [0.0, 0.0, 0.0];
    }
    [dx / len, dy / len, dz / len]
}

/// Maximum range (in blocks) at which an allay searches for items.
pub fn allay_collection_range() -> f32 {
    32.0
}

/// Whether the allay can duplicate. Requires an amethyst shard and dancing state.
pub fn allay_can_duplicate(has_amethyst: bool, dancing: bool) -> bool {
    has_amethyst && dancing
}

/// Allay movement speed in blocks per tick.
pub fn allay_speed() -> f32 {
    0.35
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_idle() {
        let state = AllayAiState::new();
        assert_eq!(state.held_item, None);
        assert_eq!(state.note_block, None);
        assert!(!state.collecting);
    }

    #[test]
    fn target_item_finds_nearest_match() {
        let nearby = vec![
            (10, [5.0, 0.0, 0.0]),
            (10, [2.0, 0.0, 0.0]),
            (20, [1.0, 0.0, 0.0]),
        ];
        let result = allay_target_item(10, &nearby);
        assert_eq!(result, Some([2.0, 0.0, 0.0]));
    }

    #[test]
    fn target_item_returns_none_when_no_match() {
        let nearby = vec![(20, [1.0, 0.0, 0.0])];
        assert_eq!(allay_target_item(10, &nearby), None);
    }

    #[test]
    fn target_item_empty_list() {
        assert_eq!(allay_target_item(10, &[]), None);
    }

    #[test]
    fn deliver_direction_is_normalized() {
        let dir = allay_deliver_direction([0.0, 0.0, 0.0], (3, 4, 0));
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn deliver_direction_zero_distance() {
        let dir = allay_deliver_direction([5.0, 5.0, 5.0], (5, 5, 5));
        assert_eq!(dir, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn collection_range_is_32() {
        assert_eq!(allay_collection_range(), 32.0);
    }

    #[test]
    fn duplicate_requires_both_conditions() {
        assert!(allay_can_duplicate(true, true));
        assert!(!allay_can_duplicate(true, false));
        assert!(!allay_can_duplicate(false, true));
        assert!(!allay_can_duplicate(false, false));
    }

    #[test]
    fn speed_value() {
        assert_eq!(allay_speed(), 0.35);
    }
}
