//! Respawn anchor mechanics for Nether respawn points.

/// Maximum number of glowstone charges an anchor can hold.
pub const MAX_CHARGES: u8 = 4;

/// A respawn anchor block that stores charges and dimension info.
#[derive(Debug, Clone, PartialEq)]
pub struct RespawnAnchor {
    pub charges: u8,
    pub dimension: u8,
}

impl RespawnAnchor {
    /// Create a new respawn anchor in the given dimension with zero charges.
    pub fn new(dim: u8) -> Self {
        Self {
            charges: 0,
            dimension: dim,
        }
    }
}

/// Add a charge to the anchor. Returns true if successful, false if already full.
pub fn charge_anchor(anchor: &mut RespawnAnchor) -> bool {
    if anchor.charges >= MAX_CHARGES {
        return false;
    }
    anchor.charges += 1;
    true
}

/// Use the anchor to set a respawn point. Only works in the Nether.
pub fn use_anchor(anchor: &mut RespawnAnchor, in_nether: bool) -> Result<(), &'static str> {
    if !in_nether {
        return Err("Respawn anchor explodes outside the Nether");
    }
    if anchor.charges == 0 {
        return Err("Respawn anchor has no charges");
    }
    anchor.charges -= 1;
    Ok(())
}

/// Calculate the light level emitted by an anchor based on its charges.
pub fn anchor_light_level(charges: u8) -> u8 {
    3 * charges
}

/// Whether a respawn anchor explodes in the Overworld.
pub fn anchor_explodes_in_overworld() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_anchor_has_zero_charges() {
        let anchor = RespawnAnchor::new(1);
        assert_eq!(anchor.charges, 0);
        assert_eq!(anchor.dimension, 1);
    }

    #[test]
    fn charge_anchor_increments() {
        let mut anchor = RespawnAnchor::new(1);
        assert!(charge_anchor(&mut anchor));
        assert_eq!(anchor.charges, 1);
    }

    #[test]
    fn charge_anchor_fails_when_full() {
        let mut anchor = RespawnAnchor::new(1);
        for _ in 0..MAX_CHARGES {
            charge_anchor(&mut anchor);
        }
        assert!(!charge_anchor(&mut anchor));
        assert_eq!(anchor.charges, MAX_CHARGES);
    }

    #[test]
    fn use_anchor_in_nether_decrements() {
        let mut anchor = RespawnAnchor::new(1);
        charge_anchor(&mut anchor);
        charge_anchor(&mut anchor);
        assert!(use_anchor(&mut anchor, true).is_ok());
        assert_eq!(anchor.charges, 1);
    }

    #[test]
    fn use_anchor_fails_outside_nether() {
        let mut anchor = RespawnAnchor::new(0);
        charge_anchor(&mut anchor);
        let result = use_anchor(&mut anchor, false);
        assert_eq!(result, Err("Respawn anchor explodes outside the Nether"));
    }

    #[test]
    fn use_anchor_fails_with_no_charges() {
        let mut anchor = RespawnAnchor::new(1);
        let result = use_anchor(&mut anchor, true);
        assert_eq!(result, Err("Respawn anchor has no charges"));
    }

    #[test]
    fn light_level_calculation() {
        assert_eq!(anchor_light_level(0), 0);
        assert_eq!(anchor_light_level(1), 3);
        assert_eq!(anchor_light_level(4), 12);
    }

    #[test]
    fn explodes_in_overworld() {
        assert!(anchor_explodes_in_overworld());
    }
}
