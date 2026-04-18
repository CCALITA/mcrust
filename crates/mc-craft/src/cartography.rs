// ── Cartography table ────────────────────────────────────────────────────
//
// Implements the three cartography-table operations from vanilla Minecraft:
//   • Clone  – duplicate a map (identical data copy)
//   • Extend – zoom out one level (scale 0..=4, max 4)
//   • Lock   – make the map read-only so it stops updating

/// Maximum map scale level (vanilla: 1:16 block ratio at scale 4).
const MAX_SCALE: u8 = 4;

// ── Data types ──────────────────────────────────────────────────────────

/// Lightweight map representation local to `mc-craft`.
///
/// Kept here rather than depending on `mc-world::MapData` so the craft
/// crate stays self-contained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleMapData {
    /// Zoom level (0 = closest, 4 = furthest).
    pub scale: u8,
    /// Whether the map is locked (no further updates).
    pub locked: bool,
    /// Raw pixel / colour-index data.
    pub data: Vec<u8>,
}

/// The three actions a cartography table can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartographyAction {
    /// Duplicate the map.
    Clone,
    /// Zoom out by one scale level.
    Extend,
    /// Freeze the map so it no longer updates.
    Lock,
}

// ── Operations ──────────────────────────────────────────────────────────

/// Clone a map, producing an identical copy.
#[must_use]
pub fn clone_map(map: &SimpleMapData) -> SimpleMapData {
    map.clone()
}

/// Extend (zoom out) a map by one scale level.
///
/// Returns `None` if the map is already at [`MAX_SCALE`] or is locked.
#[must_use]
pub fn extend_map(map: &SimpleMapData) -> Option<SimpleMapData> {
    if map.locked || map.scale >= MAX_SCALE {
        return None;
    }
    Some(SimpleMapData {
        scale: map.scale + 1,
        locked: map.locked,
        data: map.data.clone(),
    })
}

/// Lock a map in place, preventing further modifications.
pub fn lock_map(map: &mut SimpleMapData) {
    map.locked = true;
}

/// Perform a cartography-table action on the given map.
///
/// Returns `Some(new_map)` for `Clone` and successful `Extend`,
/// `None` when `Extend` is impossible (max scale or locked), and
/// `None` for `Lock` (applied in-place via [`lock_map`]).
#[must_use]
pub fn cartography_table(action: CartographyAction, map: &SimpleMapData) -> Option<SimpleMapData> {
    match action {
        CartographyAction::Clone => Some(clone_map(map)),
        CartographyAction::Extend => extend_map(map),
        CartographyAction::Lock => {
            // Lock mutates the original; caller should use `lock_map` directly.
            // Returning a locked copy here keeps the function pure.
            let mut locked = map.clone();
            lock_map(&mut locked);
            Some(locked)
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map() -> SimpleMapData {
        SimpleMapData {
            scale: 0,
            locked: false,
            data: vec![1, 2, 3, 4],
        }
    }

    #[test]
    fn clone_produces_equal_data() {
        let original = sample_map();
        let cloned = clone_map(&original);
        assert_eq!(original, cloned);
    }

    #[test]
    fn extend_increments_scale() {
        let map = sample_map();
        let extended = extend_map(&map).expect("extend should succeed at scale 0");
        assert_eq!(extended.scale, map.scale + 1);
        assert_eq!(extended.data, map.data);
    }

    #[test]
    fn extend_caps_at_max_scale() {
        let map = SimpleMapData {
            scale: MAX_SCALE,
            locked: false,
            data: vec![10, 20],
        };
        assert!(extend_map(&map).is_none(), "extend beyond MAX_SCALE must return None");
    }

    #[test]
    fn lock_prevents_extend() {
        let mut map = sample_map();
        lock_map(&mut map);
        assert!(map.locked);
        assert!(extend_map(&map).is_none(), "locked map must not be extendable");
    }

    #[test]
    fn cartography_table_clone_action() {
        let map = sample_map();
        let result = cartography_table(CartographyAction::Clone, &map);
        assert_eq!(result, Some(map));
    }

    #[test]
    fn cartography_table_extend_action() {
        let map = sample_map();
        let result = cartography_table(CartographyAction::Extend, &map).unwrap();
        assert_eq!(result.scale, 1);
    }

    #[test]
    fn cartography_table_lock_action() {
        let map = sample_map();
        let result = cartography_table(CartographyAction::Lock, &map).unwrap();
        assert!(result.locked);
    }

    #[test]
    fn extend_all_scales_up_to_max() {
        let mut map = sample_map();
        for expected_scale in 1..=MAX_SCALE {
            map = extend_map(&map).unwrap();
            assert_eq!(map.scale, expected_scale);
        }
        assert!(extend_map(&map).is_none());
    }
}
