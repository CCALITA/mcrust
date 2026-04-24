//! Specialized furnaces — Smoker (food only) and Blast Furnace (metals/ores only).
//!
//! Both cook at twice the speed of a regular [`crate::furnace::Furnace`] but accept
//! only a restricted subset of inputs. See [`SpecializedFurnace`].

/// Type of specialized furnace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnaceType {
    /// Cooks food items at 2x speed.
    Smoker,
    /// Smelts metals and ores at 2x speed.
    BlastFurnace,
}

/// State for a specialized furnace block.
#[derive(Debug, Clone)]
pub struct SpecializedFurnace {
    pub furnace_type: FurnaceType,
    pub input_slot: Option<u16>,
    pub fuel_slot: Option<u16>,
    pub output_slot: Option<u16>,
    /// Time elapsed cooking the current input, in seconds.
    pub cook_time: f32,
    /// Total time required to finish the current cook, in seconds.
    pub total_time: f32,
    /// Fuel remaining (in seconds of burn time).
    pub fuel_remaining: f32,
}

impl SpecializedFurnace {
    /// Construct an empty specialized furnace of the given type.
    pub fn new(furnace_type: FurnaceType) -> Self {
        Self {
            furnace_type,
            input_slot: None,
            fuel_slot: None,
            output_slot: None,
            cook_time: 0.0,
            total_time: cook_duration(),
            fuel_remaining: 0.0,
        }
    }
}

// ── Item-class predicates ──────────────────────────────────────────────────

/// Food item ID range from batch 17 (3000-3017 inclusive).
pub fn is_food(item_id: u16) -> bool {
    (3000..=3017).contains(&item_id)
}

/// Iron, gold, and copper ores/raw items used in blast-furnace smelting.
///
/// Ranges chosen to cover both the legacy ore IDs (300, 301) and new
/// raw-metal/copper IDs (340-349) without overlapping the food range.
pub fn is_smeltable_metal(item_id: u16) -> bool {
    matches!(item_id, 300 | 301) || (340..=349).contains(&item_id)
}

/// Whether `item` is a valid input for a furnace of `furnace_type`.
pub fn can_cook(furnace_type: FurnaceType, item: u16) -> bool {
    match furnace_type {
        FurnaceType::Smoker => is_food(item),
        FurnaceType::BlastFurnace => is_smeltable_metal(item),
    }
}

// ── Speed and timing ───────────────────────────────────────────────────────

/// Speed multiplier vs. a regular furnace (specialized furnaces are 2x faster).
pub fn cook_speed_multiplier() -> f32 {
    2.0
}

/// Time required to finish one cook cycle, in seconds.
pub fn cook_duration() -> f32 {
    5.0
}

// ── Experience ─────────────────────────────────────────────────────────────

/// Experience awarded per completed cook for `item`.
pub fn experience_per_cook(item: u16) -> f32 {
    match item {
        // Iron/gold/copper smelting yields more XP than basic food.
        300 => 0.7, // iron ore
        301 => 1.0, // gold ore
        340..=349 => 0.7,
        // Food items
        3000..=3017 => 0.35,
        _ => 0.1,
    }
}

// ── Tick ───────────────────────────────────────────────────────────────────

/// Advance the furnace by `dt` seconds. Returns the produced output item id
/// when a cook completes on this tick (and resets `cook_time`).
pub fn tick(state: &mut SpecializedFurnace, dt: f32) -> Option<u16> {
    let input = state.input_slot?;
    if !can_cook(state.furnace_type, input) {
        state.cook_time = 0.0;
        return None;
    }
    if state.fuel_remaining <= 0.0 {
        // Try to consume one unit of fuel from the fuel slot.
        if state.fuel_slot.is_some() {
            state.fuel_remaining = cook_duration();
            state.fuel_slot = None;
        } else {
            state.cook_time = 0.0;
            return None;
        }
    }
    let burn = dt.min(state.fuel_remaining);
    state.fuel_remaining -= burn;
    state.cook_time += burn;
    if state.cook_time >= state.total_time {
        state.cook_time = 0.0;
        let output = smelt_output(input);
        state.input_slot = None;
        state.output_slot = Some(output);
        Some(output)
    } else {
        None
    }
}

/// Map an input item to the smelted output. Currently identity for unknown
/// items; specific outputs encoded for known iron/gold/food pairs.
fn smelt_output(input: u16) -> u16 {
    match input {
        300 => 330, // iron ore -> iron ingot (placeholder id)
        301 => 331, // gold ore -> gold ingot (placeholder id)
        // Raw metal -> ingot (offset by 10 within the 340-349 range).
        340..=349 => input.saturating_add(10),
        // Food: cooked variant lives in the 3100-3117 band by convention.
        3000..=3017 => input + 100,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoker_accepts_food_but_not_metals() {
        assert!(can_cook(FurnaceType::Smoker, 3005));
        assert!(!can_cook(FurnaceType::Smoker, 300));
        assert!(!can_cook(FurnaceType::Smoker, 345));
    }

    #[test]
    fn blast_furnace_accepts_metals_but_not_food() {
        assert!(can_cook(FurnaceType::BlastFurnace, 300));
        assert!(can_cook(FurnaceType::BlastFurnace, 301));
        assert!(can_cook(FurnaceType::BlastFurnace, 345));
        assert!(!can_cook(FurnaceType::BlastFurnace, 3005));
    }

    #[test]
    fn cook_speed_is_2x() {
        assert_eq!(cook_speed_multiplier(), 2.0);
    }

    #[test]
    fn cook_duration_is_5_seconds() {
        assert_eq!(cook_duration(), 5.0);
    }

    #[test]
    fn experience_values_vary_by_item() {
        assert_eq!(experience_per_cook(300), 0.7);
        assert_eq!(experience_per_cook(301), 1.0);
        assert_eq!(experience_per_cook(3005), 0.35);
        assert_eq!(experience_per_cook(9999), 0.1);
    }

    #[test]
    fn new_initializes_empty_with_default_total_time() {
        let f = SpecializedFurnace::new(FurnaceType::Smoker);
        assert_eq!(f.furnace_type, FurnaceType::Smoker);
        assert!(f.input_slot.is_none());
        assert!(f.output_slot.is_none());
        assert_eq!(f.total_time, cook_duration());
    }

    #[test]
    fn tick_with_no_input_returns_none() {
        let mut f = SpecializedFurnace::new(FurnaceType::Smoker);
        assert_eq!(tick(&mut f, 1.0), None);
    }

    #[test]
    fn tick_completes_cook_after_full_duration() {
        let mut f = SpecializedFurnace::new(FurnaceType::Smoker);
        f.input_slot = Some(3005);
        f.fuel_slot = Some(1);
        // Below the threshold first.
        assert_eq!(tick(&mut f, cook_duration() - 0.1), None);
        // Crossing the threshold finishes the cook.
        let out = tick(&mut f, 0.2);
        assert_eq!(out, Some(3105));
        assert!(f.input_slot.is_none());
        assert_eq!(f.output_slot, Some(3105));
    }

    #[test]
    fn tick_rejects_invalid_input_for_furnace_type() {
        let mut f = SpecializedFurnace::new(FurnaceType::BlastFurnace);
        f.input_slot = Some(3005); // food in blast furnace
        f.fuel_slot = Some(1);
        assert_eq!(tick(&mut f, 10.0), None);
    }

    #[test]
    fn tick_without_fuel_does_nothing() {
        let mut f = SpecializedFurnace::new(FurnaceType::Smoker);
        f.input_slot = Some(3005);
        assert_eq!(tick(&mut f, 1.0), None);
        assert_eq!(f.cook_time, 0.0);
    }

    #[test]
    fn is_food_range_boundaries() {
        assert!(is_food(3000));
        assert!(is_food(3017));
        assert!(!is_food(2999));
        assert!(!is_food(3018));
    }

    #[test]
    fn is_smeltable_metal_covers_known_ids() {
        assert!(is_smeltable_metal(300));
        assert!(is_smeltable_metal(301));
        assert!(is_smeltable_metal(340));
        assert!(is_smeltable_metal(349));
        assert!(!is_smeltable_metal(339));
        assert!(!is_smeltable_metal(350));
    }
}
