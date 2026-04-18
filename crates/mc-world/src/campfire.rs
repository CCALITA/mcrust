//! Campfire block entity — cooking, extinguishing, and smoke.

/// Maps a raw food item ID to its cooked variant.
pub fn cooked_item(raw: u16) -> Option<u16> {
    match raw {
        1 => Some(2),   // raw beef -> steak
        3 => Some(4),   // raw chicken -> cooked chicken
        5 => Some(6),   // raw porkchop -> cooked porkchop
        7 => Some(8),   // raw mutton -> cooked mutton
        9 => Some(10),  // raw salmon -> cooked salmon
        11 => Some(12), // raw cod -> cooked cod
        _ => None,
    }
}

/// Damage dealt to entities standing on a campfire.
pub fn campfire_damage(soul: bool) -> f32 {
    if soul {
        2.0
    } else {
        1.0
    }
}

/// Maximum smoke particle height (in blocks) above a campfire.
pub fn smoke_height(hay_below: bool) -> u8 {
    if hay_below {
        24
    } else {
        10
    }
}

const COOK_TICKS: u32 = 600;

/// State of a campfire block entity.
#[derive(Debug, Clone, PartialEq)]
pub struct CampfireState {
    pub lit: bool,
    pub items: [Option<(u16, u32)>; 4],
    pub soul: bool,
}

impl CampfireState {
    /// Create a new campfire (lit by default).
    pub fn new(soul: bool) -> Self {
        Self {
            lit: true,
            items: [None; 4],
            soul,
        }
    }

    /// Place a cookable item on the campfire.
    /// Returns `true` if the item was placed, `false` if all slots are full
    /// or the item has no cooked variant.
    pub fn place_item(&mut self, id: u16) -> bool {
        if cooked_item(id).is_none() {
            return false;
        }
        for slot in &mut self.items {
            if slot.is_none() {
                *slot = Some((id, 0));
                return true;
            }
        }
        false
    }

    /// Advance cooking timers by `dt_ticks`. Items that reach 600 ticks are
    /// removed (considered cooked and popped into the world).
    pub fn tick(&mut self, dt_ticks: u32) {
        if !self.lit {
            return;
        }
        for slot in &mut self.items {
            if let Some((_, ticks)) = slot {
                *ticks = ticks.saturating_add(dt_ticks);
                if *ticks >= COOK_TICKS {
                    *slot = None;
                }
            }
        }
    }

    /// Extinguish the campfire (e.g. water or shovel).
    pub fn extinguish(&mut self) {
        self.lit = false;
    }

    /// Relight the campfire (e.g. flint and steel).
    pub fn relight(&mut self) {
        self.lit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooking_completes_at_600_ticks() {
        let mut cf = CampfireState::new(false);
        assert!(cf.place_item(1)); // raw beef
        cf.tick(599);
        assert!(cf.items[0].is_some(), "item should still be cooking at 599");
        cf.tick(1);
        assert!(cf.items[0].is_none(), "item should be removed at 600");
    }

    #[test]
    fn four_item_limit() {
        let mut cf = CampfireState::new(false);
        assert!(cf.place_item(1));
        assert!(cf.place_item(3));
        assert!(cf.place_item(5));
        assert!(cf.place_item(7));
        assert!(!cf.place_item(9), "fifth item should be rejected");
    }

    #[test]
    fn uncookable_item_rejected() {
        let mut cf = CampfireState::new(false);
        assert!(!cf.place_item(999));
    }

    #[test]
    fn extinguish_stops_cooking() {
        let mut cf = CampfireState::new(false);
        cf.place_item(1);
        cf.extinguish();
        cf.tick(1000);
        assert!(
            cf.items[0].is_some(),
            "item should not cook while extinguished"
        );
        assert_eq!(cf.items[0].unwrap().1, 0, "ticks should not advance");
    }

    #[test]
    fn relight_resumes_cooking() {
        let mut cf = CampfireState::new(false);
        cf.place_item(1);
        cf.extinguish();
        cf.tick(100);
        cf.relight();
        cf.tick(600);
        assert!(cf.items[0].is_none(), "item should finish after relight");
    }

    #[test]
    fn soul_campfire_damage() {
        assert_eq!(campfire_damage(true), 2.0);
        assert_eq!(campfire_damage(false), 1.0);
    }

    #[test]
    fn smoke_height_with_and_without_hay() {
        assert_eq!(smoke_height(true), 24);
        assert_eq!(smoke_height(false), 10);
    }

    #[test]
    fn cooked_item_mappings() {
        assert_eq!(cooked_item(1), Some(2));
        assert_eq!(cooked_item(3), Some(4));
        assert_eq!(cooked_item(5), Some(6));
        assert_eq!(cooked_item(7), Some(8));
        assert_eq!(cooked_item(9), Some(10));
        assert_eq!(cooked_item(11), Some(12));
        assert_eq!(cooked_item(100), None);
    }
}
