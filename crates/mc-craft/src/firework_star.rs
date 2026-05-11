//! Firework star crafting recipes and firework rocket assembly.
//!
//! Provides [`FireworkStarRecipe`] for representing crafted firework stars,
//! [`craft_firework_star`] for crafting them from components, and
//! [`firework_rocket_recipe`] for assembling rockets from stars and gunpowder.

// ── Shape constants ─────────────────────────────────────────────────────

/// Small ball (default when no shape item is provided).
const SHAPE_SMALL_BALL: u8 = 0;
/// Large ball (fire charge).
const SHAPE_LARGE_BALL: u8 = 1;
/// Star-shaped (gold nugget).
const SHAPE_STAR: u8 = 2;
/// Creeper face (mob head).
const SHAPE_CREEPER: u8 = 3;
/// Burst (feather).
const SHAPE_BURST: u8 = 4;

// ── Item IDs for shape modifiers ────────────────────────────────────────

const ITEM_FIRE_CHARGE: u16 = 602;
const ITEM_GOLD_NUGGET: u16 = 603;
const ITEM_MOB_HEAD: u16 = 604;
const ITEM_FEATHER: u16 = 605;

// ── Types ───────────────────────────────────────────────────────────────

/// A crafted firework star with explosion properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkStarRecipe {
    /// Explosion shape (0=small ball, 1=large ball, 2=star, 3=creeper, 4=burst).
    pub shape: u8,
    /// Primary color indices applied on detonation.
    pub colors: Vec<u8>,
    /// Fade color indices (applied after initial explosion fades).
    pub fade_colors: Vec<u8>,
    /// Whether the explosion leaves a trail.
    pub trail: bool,
    /// Whether the explosion has a twinkle effect.
    pub twinkle: bool,
}

// ── Public API ──────────────────────────────────────────────────────────

/// Determine the explosion shape from an optional shape-modifier item ID.
///
/// Returns `SHAPE_SMALL_BALL` (0) when no recognised shape item is given.
#[must_use]
pub fn shape_from_item(item: u16) -> u8 {
    match item {
        ITEM_FIRE_CHARGE => SHAPE_LARGE_BALL,
        ITEM_GOLD_NUGGET => SHAPE_STAR,
        ITEM_MOB_HEAD => SHAPE_CREEPER,
        ITEM_FEATHER => SHAPE_BURST,
        _ => SHAPE_SMALL_BALL,
    }
}

/// Craft a firework star from raw ingredients.
///
/// # Arguments
///
/// * `gunpowder`  - Whether gunpowder is present (required).
/// * `dyes`       - Colour indices for the explosion (at least one required).
/// * `shape_item` - Optional item that determines the explosion shape.
/// * `trail`      - Whether diamond (trail modifier) is included.
/// * `twinkle`    - Whether glowstone dust (twinkle modifier) is included.
///
/// Returns `None` when gunpowder is missing or no dyes are provided.
#[must_use]
pub fn craft_firework_star(
    gunpowder: bool,
    dyes: &[u8],
    shape_item: Option<u16>,
    trail: bool,
    twinkle: bool,
) -> Option<FireworkStarRecipe> {
    if !gunpowder || dyes.is_empty() {
        return None;
    }

    let shape = match shape_item {
        Some(id) => shape_from_item(id),
        None => SHAPE_SMALL_BALL,
    };

    Some(FireworkStarRecipe {
        shape,
        colors: dyes.to_vec(),
        fade_colors: Vec::new(),
        trail,
        twinkle,
    })
}

/// Assemble a firework rocket from paper, gunpowder, and optional stars.
///
/// # Arguments
///
/// * `paper`      - Whether paper is present (required).
/// * `gunpowder`  - Number of gunpowder items (1-3); determines flight duration.
/// * `stars`      - Firework stars to embed in the rocket.
///
/// Returns the flight duration (1-3) on success, or `None` if paper is missing
/// or gunpowder count is outside the valid 1..=3 range.
#[must_use]
pub fn firework_rocket_recipe(
    paper: bool,
    gunpowder: u8,
    stars: &[FireworkStarRecipe],
) -> Option<u8> {
    if !paper || !(1..=3).contains(&gunpowder) {
        return None;
    }
    // Stars are valid (0-7 in vanilla); we accept any slice.
    let _ = stars;
    Some(gunpowder)
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── shape_from_item ─────────────────────────────────────────────────

    #[test]
    fn shape_fire_charge_returns_large_ball() {
        assert_eq!(shape_from_item(ITEM_FIRE_CHARGE), SHAPE_LARGE_BALL);
    }

    #[test]
    fn shape_gold_nugget_returns_star() {
        assert_eq!(shape_from_item(ITEM_GOLD_NUGGET), SHAPE_STAR);
    }

    #[test]
    fn shape_mob_head_returns_creeper() {
        assert_eq!(shape_from_item(ITEM_MOB_HEAD), SHAPE_CREEPER);
    }

    #[test]
    fn shape_feather_returns_burst() {
        assert_eq!(shape_from_item(ITEM_FEATHER), SHAPE_BURST);
    }

    #[test]
    fn shape_unknown_item_returns_small_ball() {
        assert_eq!(shape_from_item(9999), SHAPE_SMALL_BALL);
    }

    // ── craft_firework_star ─────────────────────────────────────────────

    #[test]
    fn craft_basic_star() {
        let star = craft_firework_star(true, &[1, 2], None, false, false).unwrap();
        assert_eq!(star.shape, SHAPE_SMALL_BALL);
        assert_eq!(star.colors, vec![1, 2]);
        assert!(star.fade_colors.is_empty());
        assert!(!star.trail);
        assert!(!star.twinkle);
    }

    #[test]
    fn craft_star_with_shape() {
        let star = craft_firework_star(true, &[3], Some(ITEM_FIRE_CHARGE), false, false).unwrap();
        assert_eq!(star.shape, SHAPE_LARGE_BALL);
    }

    #[test]
    fn craft_star_with_trail_and_twinkle() {
        let star = craft_firework_star(true, &[1], None, true, true).unwrap();
        assert!(star.trail);
        assert!(star.twinkle);
    }

    #[test]
    fn craft_star_requires_gunpowder() {
        assert!(craft_firework_star(false, &[1], None, false, false).is_none());
    }

    #[test]
    fn craft_star_requires_at_least_one_dye() {
        assert!(craft_firework_star(true, &[], None, false, false).is_none());
    }

    #[test]
    fn craft_star_no_gunpowder_no_dyes() {
        assert!(craft_firework_star(false, &[], None, false, false).is_none());
    }

    // ── firework_rocket_recipe ──────────────────────────────────────────

    #[test]
    fn rocket_basic() {
        assert_eq!(firework_rocket_recipe(true, 1, &[]), Some(1));
    }

    #[test]
    fn rocket_max_gunpowder() {
        assert_eq!(firework_rocket_recipe(true, 3, &[]), Some(3));
    }

    #[test]
    fn rocket_with_stars() {
        let star = craft_firework_star(true, &[1], None, false, false).unwrap();
        assert_eq!(firework_rocket_recipe(true, 2, &[star]), Some(2));
    }

    #[test]
    fn rocket_requires_paper() {
        assert!(firework_rocket_recipe(false, 1, &[]).is_none());
    }

    #[test]
    fn rocket_rejects_zero_gunpowder() {
        assert!(firework_rocket_recipe(true, 0, &[]).is_none());
    }

    #[test]
    fn rocket_rejects_excess_gunpowder() {
        assert!(firework_rocket_recipe(true, 4, &[]).is_none());
    }
}
