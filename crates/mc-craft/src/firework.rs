// ── Item constants (firework-specific) ───────────────────────────────────

/// Paper item ID.
const ITEM_PAPER: u16 = 600;

/// Gunpowder item ID.
const ITEM_GUNPOWDER: u16 = 601;

/// Fire charge item (shape modifier: large ball).
const ITEM_FIRE_CHARGE: u16 = 602;

/// Gold nugget item (shape modifier: star).
const ITEM_GOLD_NUGGET: u16 = 603;

/// Mob head item (shape modifier: creeper).
const ITEM_MOB_HEAD: u16 = 604;

/// Feather item (shape modifier: burst).
const ITEM_FEATHER: u16 = 605;

/// Diamond item (effect: trail).
const ITEM_DIAMOND: u16 = 606;

/// Glowstone dust item (effect: twinkle).
const ITEM_GLOWSTONE_DUST: u16 = 607;

// ── Firework types ──────────────────────────────────────────────────────

/// The explosion shape of a firework star.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireworkShape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

/// A firework star produced by combining dye, gunpowder, and optional
/// shape/effect modifiers on a crafting table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkStar {
    pub shape: FireworkShape,
    pub colors: Vec<u8>,
    pub fade_colors: Vec<u8>,
    pub trail: bool,
    pub twinkle: bool,
}

/// A firework rocket produced by combining paper, gunpowder (1-3), and
/// optional firework stars on a crafting table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkRocket {
    pub stars: Vec<FireworkStar>,
    pub flight_duration: u8,
}

// ── Crafting functions ──────────────────────────────────────────────────

/// Determine the explosion shape from an optional shape-modifier item.
///
/// Returns `SmallBall` when no shape item is provided (the vanilla default).
#[must_use]
fn shape_from_item(shape_item: Option<u16>) -> FireworkShape {
    match shape_item {
        Some(ITEM_FIRE_CHARGE) => FireworkShape::LargeBall,
        Some(ITEM_GOLD_NUGGET) => FireworkShape::Star,
        Some(ITEM_MOB_HEAD) => FireworkShape::Creeper,
        Some(ITEM_FEATHER) => FireworkShape::Burst,
        _ => FireworkShape::SmallBall,
    }
}

/// Craft a firework star from dye colors, an optional shape-modifier item,
/// and a list of effect-modifier items (diamond for trail, glowstone dust
/// for twinkle).
///
/// # Arguments
///
/// * `dye_colors` - Color indices applied on detonation (at least one required).
/// * `shape_item` - Optional item that determines the explosion shape.
/// * `effects`    - Items that add trail / twinkle effects.
///
/// Returns `None` if `dye_colors` is empty.
#[must_use]
pub fn craft_star(
    dye_colors: &[u8],
    shape_item: Option<u16>,
    effects: &[u16],
) -> Option<FireworkStar> {
    if dye_colors.is_empty() {
        return None;
    }

    let shape = shape_from_item(shape_item);
    let trail = effects.contains(&ITEM_DIAMOND);
    let twinkle = effects.contains(&ITEM_GLOWSTONE_DUST);

    Some(FireworkStar {
        shape,
        colors: dye_colors.to_vec(),
        fade_colors: Vec::new(),
        trail,
        twinkle,
    })
}

/// Craft a firework rocket from paper, a gunpowder count (1-3), and
/// optional firework stars.
///
/// # Arguments
///
/// * `paper`           - The paper item ID (must equal `ITEM_PAPER`).
/// * `gunpowder_count` - Number of gunpowder (1-3); determines flight
///                        duration.
/// * `stars`           - Firework stars to embed in the rocket.
///
/// Returns `None` if `paper` is not the correct item or `gunpowder_count`
/// is outside the valid 1..=3 range.
#[must_use]
pub fn craft_rocket(
    paper: u16,
    gunpowder_count: u8,
    stars: &[FireworkStar],
) -> Option<FireworkRocket> {
    if paper != ITEM_PAPER {
        return None;
    }
    if !(1..=3).contains(&gunpowder_count) {
        return None;
    }

    Some(FireworkRocket {
        stars: stars.to_vec(),
        flight_duration: gunpowder_count,
    })
}

/// Calculate the elytra boost power produced by a firework rocket.
///
/// In vanilla Minecraft the boost is proportional to the flight duration
/// (gunpowder count 1-3).  We model this as a linear scale where each
/// unit of flight duration contributes 1.5 units of boost power.
///
/// Returns 0.0 for a flight duration of 0.
#[must_use]
pub fn elytra_boost_power(flight_duration: u8) -> f32 {
    f32::from(flight_duration) * 1.5
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── craft_star ──────────────────────────────────────────────────────

    #[test]
    fn craft_star_default_shape_small_ball() {
        let star = craft_star(&[1, 2], None, &[]).unwrap();
        assert_eq!(star.shape, FireworkShape::SmallBall);
        assert_eq!(star.colors, vec![1, 2]);
        assert!(star.fade_colors.is_empty());
        assert!(!star.trail);
        assert!(!star.twinkle);
    }

    #[test]
    fn craft_star_with_large_ball_shape() {
        let star = craft_star(&[3], Some(ITEM_FIRE_CHARGE), &[]).unwrap();
        assert_eq!(star.shape, FireworkShape::LargeBall);
    }

    #[test]
    fn craft_star_with_star_shape() {
        let star = craft_star(&[4], Some(ITEM_GOLD_NUGGET), &[]).unwrap();
        assert_eq!(star.shape, FireworkShape::Star);
    }

    #[test]
    fn craft_star_with_creeper_shape() {
        let star = craft_star(&[5], Some(ITEM_MOB_HEAD), &[]).unwrap();
        assert_eq!(star.shape, FireworkShape::Creeper);
    }

    #[test]
    fn craft_star_with_burst_shape() {
        let star = craft_star(&[6], Some(ITEM_FEATHER), &[]).unwrap();
        assert_eq!(star.shape, FireworkShape::Burst);
    }

    #[test]
    fn craft_star_with_trail_effect() {
        let star = craft_star(&[1], None, &[ITEM_DIAMOND]).unwrap();
        assert!(star.trail);
        assert!(!star.twinkle);
    }

    #[test]
    fn craft_star_with_twinkle_effect() {
        let star = craft_star(&[1], None, &[ITEM_GLOWSTONE_DUST]).unwrap();
        assert!(!star.trail);
        assert!(star.twinkle);
    }

    #[test]
    fn craft_star_with_both_effects() {
        let star = craft_star(&[1], None, &[ITEM_DIAMOND, ITEM_GLOWSTONE_DUST]).unwrap();
        assert!(star.trail);
        assert!(star.twinkle);
    }

    #[test]
    fn craft_star_returns_none_with_no_dye() {
        let result = craft_star(&[], None, &[]);
        assert!(result.is_none());
    }

    // ── craft_rocket ────────────────────────────────────────────────────

    #[test]
    fn craft_rocket_with_no_stars() {
        let rocket = craft_rocket(ITEM_PAPER, 1, &[]).unwrap();
        assert!(rocket.stars.is_empty());
        assert_eq!(rocket.flight_duration, 1);
    }

    #[test]
    fn craft_rocket_with_stars() {
        let star = craft_star(&[1, 2], None, &[]).unwrap();
        let rocket = craft_rocket(ITEM_PAPER, 2, &[star.clone()]).unwrap();
        assert_eq!(rocket.stars.len(), 1);
        assert_eq!(rocket.stars[0], star);
        assert_eq!(rocket.flight_duration, 2);
    }

    #[test]
    fn craft_rocket_max_gunpowder() {
        let rocket = craft_rocket(ITEM_PAPER, 3, &[]).unwrap();
        assert_eq!(rocket.flight_duration, 3);
    }

    #[test]
    fn craft_rocket_rejects_wrong_paper_item() {
        let result = craft_rocket(9999, 1, &[]);
        assert!(result.is_none());
    }

    #[test]
    fn craft_rocket_rejects_zero_gunpowder() {
        let result = craft_rocket(ITEM_PAPER, 0, &[]);
        assert!(result.is_none());
    }

    #[test]
    fn craft_rocket_rejects_excess_gunpowder() {
        let result = craft_rocket(ITEM_PAPER, 4, &[]);
        assert!(result.is_none());
    }

    // ── elytra_boost_power ──────────────────────────────────────────────

    #[test]
    fn elytra_boost_zero_duration() {
        let power = elytra_boost_power(0);
        assert!((power - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn elytra_boost_scales_with_duration() {
        assert!((elytra_boost_power(1) - 1.5).abs() < f32::EPSILON);
        assert!((elytra_boost_power(2) - 3.0).abs() < f32::EPSILON);
        assert!((elytra_boost_power(3) - 4.5).abs() < f32::EPSILON);
    }
}
