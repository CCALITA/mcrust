// ---------------------------------------------------------------------------
// Armor slot
// ---------------------------------------------------------------------------

/// The four equipment slots that can hold armor pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmorSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}

impl ArmorSlot {
    /// Index into the fixed-size `[Option<ArmorPiece>; 4]` array.
    fn index(self) -> usize {
        match self {
            ArmorSlot::Helmet => 0,
            ArmorSlot::Chestplate => 1,
            ArmorSlot::Leggings => 2,
            ArmorSlot::Boots => 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Armor material
// ---------------------------------------------------------------------------

/// Material an armor piece is made from.
///
/// Each material defines per-slot defense values, toughness, and base
/// durability following Minecraft conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmorMaterial {
    Leather,
    Chain,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

impl ArmorMaterial {
    /// Defense points granted by this material in the given slot.
    ///
    /// Values follow the Minecraft wiki order: Helmet / Chestplate / Leggings /
    /// Boots.
    pub fn defense_for_slot(self, slot: ArmorSlot) -> u32 {
        match (self, slot) {
            // Leather: 1 / 3 / 2 / 1
            (ArmorMaterial::Leather, ArmorSlot::Helmet) => 1,
            (ArmorMaterial::Leather, ArmorSlot::Chestplate) => 3,
            (ArmorMaterial::Leather, ArmorSlot::Leggings) => 2,
            (ArmorMaterial::Leather, ArmorSlot::Boots) => 1,

            // Chain: 2 / 5 / 4 / 1
            (ArmorMaterial::Chain, ArmorSlot::Helmet) => 2,
            (ArmorMaterial::Chain, ArmorSlot::Chestplate) => 5,
            (ArmorMaterial::Chain, ArmorSlot::Leggings) => 4,
            (ArmorMaterial::Chain, ArmorSlot::Boots) => 1,

            // Iron: 2 / 6 / 5 / 2
            (ArmorMaterial::Iron, ArmorSlot::Helmet) => 2,
            (ArmorMaterial::Iron, ArmorSlot::Chestplate) => 6,
            (ArmorMaterial::Iron, ArmorSlot::Leggings) => 5,
            (ArmorMaterial::Iron, ArmorSlot::Boots) => 2,

            // Gold: 2 / 5 / 3 / 1
            (ArmorMaterial::Gold, ArmorSlot::Helmet) => 2,
            (ArmorMaterial::Gold, ArmorSlot::Chestplate) => 5,
            (ArmorMaterial::Gold, ArmorSlot::Leggings) => 3,
            (ArmorMaterial::Gold, ArmorSlot::Boots) => 1,

            // Diamond: 3 / 8 / 6 / 3
            (ArmorMaterial::Diamond, ArmorSlot::Helmet) => 3,
            (ArmorMaterial::Diamond, ArmorSlot::Chestplate) => 8,
            (ArmorMaterial::Diamond, ArmorSlot::Leggings) => 6,
            (ArmorMaterial::Diamond, ArmorSlot::Boots) => 3,

            // Netherite: 3 / 8 / 6 / 3
            (ArmorMaterial::Netherite, ArmorSlot::Helmet) => 3,
            (ArmorMaterial::Netherite, ArmorSlot::Chestplate) => 8,
            (ArmorMaterial::Netherite, ArmorSlot::Leggings) => 6,
            (ArmorMaterial::Netherite, ArmorSlot::Boots) => 3,
        }
    }

    /// Armor toughness granted by this material.
    ///
    /// Only Diamond and Netherite provide toughness; all other materials
    /// return 0.
    pub fn toughness(self) -> f32 {
        match self {
            ArmorMaterial::Diamond => 2.0,
            ArmorMaterial::Netherite => 3.0,
            _ => 0.0,
        }
    }

    /// Base durability multiplier per material. The actual durability equals
    /// `base_durability(slot) * material_multiplier`.
    fn material_multiplier(self) -> u32 {
        match self {
            ArmorMaterial::Leather => 5,
            ArmorMaterial::Chain => 15,
            ArmorMaterial::Iron => 15,
            ArmorMaterial::Gold => 7,
            ArmorMaterial::Diamond => 33,
            ArmorMaterial::Netherite => 37,
        }
    }

    /// Maximum durability for this material in the given slot.
    ///
    /// Follows the Minecraft formula: `slot_base * material_multiplier`.
    /// Slot bases: Helmet 11, Chestplate 16, Leggings 15, Boots 13.
    pub fn durability_for_slot(self, slot: ArmorSlot) -> u32 {
        let slot_base = match slot {
            ArmorSlot::Helmet => 11,
            ArmorSlot::Chestplate => 16,
            ArmorSlot::Leggings => 15,
            ArmorSlot::Boots => 13,
        };
        slot_base * self.material_multiplier()
    }
}

// ---------------------------------------------------------------------------
// Armor piece
// ---------------------------------------------------------------------------

/// A single piece of armor occupying one equipment slot.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmorPiece {
    pub material: ArmorMaterial,
    pub slot: ArmorSlot,
    pub durability: u32,
    pub max_durability: u32,
}

impl ArmorPiece {
    /// Create a new armor piece at full durability.
    pub fn new(material: ArmorMaterial, slot: ArmorSlot) -> Self {
        let max_durability = material.durability_for_slot(slot);
        Self {
            material,
            slot,
            durability: max_durability,
            max_durability,
        }
    }

    /// Returns `true` when the piece has no remaining durability.
    pub fn is_broken(&self) -> bool {
        self.durability == 0
    }
}

// ---------------------------------------------------------------------------
// Armor set
// ---------------------------------------------------------------------------

/// The four armor slots a player can wear simultaneously.
///
/// Slot layout: `[Helmet, Chestplate, Leggings, Boots]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmorSet {
    pub slots: [Option<ArmorPiece>; 4],
}

impl ArmorSet {
    /// Create an empty armor set (no pieces equipped).
    pub fn new() -> Self {
        Self {
            slots: [None, None, None, None],
        }
    }

    /// Equip a piece in its matching slot, returning the previously equipped
    /// piece (if any).
    pub fn equip(&mut self, piece: ArmorPiece) -> Option<ArmorPiece> {
        let idx = piece.slot.index();
        let previous = self.slots[idx].take();
        self.slots[idx] = Some(piece);
        previous
    }

    /// Remove and return the piece in the given slot.
    pub fn unequip(&mut self, slot: ArmorSlot) -> Option<ArmorPiece> {
        self.slots[slot.index()].take()
    }

    /// Sum of defense points across all equipped, non-broken pieces.
    pub fn total_defense(&self) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|p| !p.is_broken())
            .map(|p| p.material.defense_for_slot(p.slot))
            .sum()
    }

    /// Sum of toughness across all equipped, non-broken pieces.
    ///
    /// Each piece contributes its material's per-piece toughness.
    pub fn total_toughness(&self) -> f32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|p| !p.is_broken())
            .map(|p| p.material.toughness())
            .sum()
    }
}

impl Default for ArmorSet {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Damage reduction (Minecraft formula)
// ---------------------------------------------------------------------------

/// Calculate the fraction of damage that gets through armor.
///
/// Uses the Minecraft damage reduction formula:
///
/// ```text
/// effective_defense = max(defense / 5, defense - 4 * damage / (toughness + 8))
/// capped_defense    = min(20, effective_defense)
/// damage_taken      = damage * (1 - capped_defense / 25)
/// ```
///
/// Returns the **post-armor damage** (always >= 0).
pub fn calculate_damage_reduction(defense: u32, toughness: f32, damage: f32) -> f32 {
    if damage <= 0.0 {
        return 0.0;
    }
    let def = defense as f32;
    let effective = (def / 5.0).max(def - 4.0 * damage / (toughness + 8.0));
    let capped = effective.min(20.0);
    let taken = damage * (1.0 - capped / 25.0);
    taken.max(0.0)
}

// ---------------------------------------------------------------------------
// Apply armor durability damage
// ---------------------------------------------------------------------------

/// Reduce durability on every equipped piece by 1 when the wearer takes
/// damage. Broken pieces (durability == 0) are left in place but ignored by
/// defense / toughness calculations.
pub fn apply_armor_damage(set: &mut ArmorSet, damage: f32) {
    if damage <= 0.0 {
        return;
    }
    for slot in &mut set.slots {
        if let Some(piece) = slot.as_mut() {
            if !piece.is_broken() {
                piece.durability = piece.durability.saturating_sub(1);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ArmorSlot index mapping -------------------------------------------

    #[test]
    fn slot_indices_are_unique_and_sequential() {
        assert_eq!(ArmorSlot::Helmet.index(), 0);
        assert_eq!(ArmorSlot::Chestplate.index(), 1);
        assert_eq!(ArmorSlot::Leggings.index(), 2);
        assert_eq!(ArmorSlot::Boots.index(), 3);
    }

    // -- ArmorMaterial defense values --------------------------------------

    #[test]
    fn leather_defense_values() {
        assert_eq!(
            ArmorMaterial::Leather.defense_for_slot(ArmorSlot::Helmet),
            1
        );
        assert_eq!(
            ArmorMaterial::Leather.defense_for_slot(ArmorSlot::Chestplate),
            3
        );
        assert_eq!(
            ArmorMaterial::Leather.defense_for_slot(ArmorSlot::Leggings),
            2
        );
        assert_eq!(ArmorMaterial::Leather.defense_for_slot(ArmorSlot::Boots), 1);
    }

    #[test]
    fn iron_defense_values() {
        assert_eq!(ArmorMaterial::Iron.defense_for_slot(ArmorSlot::Helmet), 2);
        assert_eq!(
            ArmorMaterial::Iron.defense_for_slot(ArmorSlot::Chestplate),
            6
        );
        assert_eq!(ArmorMaterial::Iron.defense_for_slot(ArmorSlot::Leggings), 5);
        assert_eq!(ArmorMaterial::Iron.defense_for_slot(ArmorSlot::Boots), 2);
    }

    #[test]
    fn diamond_defense_values() {
        assert_eq!(
            ArmorMaterial::Diamond.defense_for_slot(ArmorSlot::Helmet),
            3
        );
        assert_eq!(
            ArmorMaterial::Diamond.defense_for_slot(ArmorSlot::Chestplate),
            8
        );
        assert_eq!(
            ArmorMaterial::Diamond.defense_for_slot(ArmorSlot::Leggings),
            6
        );
        assert_eq!(ArmorMaterial::Diamond.defense_for_slot(ArmorSlot::Boots), 3);
    }

    #[test]
    fn netherite_defense_matches_diamond() {
        for slot in [
            ArmorSlot::Helmet,
            ArmorSlot::Chestplate,
            ArmorSlot::Leggings,
            ArmorSlot::Boots,
        ] {
            assert_eq!(
                ArmorMaterial::Netherite.defense_for_slot(slot),
                ArmorMaterial::Diamond.defense_for_slot(slot),
                "netherite and diamond should match for {slot:?}"
            );
        }
    }

    #[test]
    fn gold_defense_values() {
        assert_eq!(ArmorMaterial::Gold.defense_for_slot(ArmorSlot::Helmet), 2);
        assert_eq!(
            ArmorMaterial::Gold.defense_for_slot(ArmorSlot::Chestplate),
            5
        );
        assert_eq!(ArmorMaterial::Gold.defense_for_slot(ArmorSlot::Leggings), 3);
        assert_eq!(ArmorMaterial::Gold.defense_for_slot(ArmorSlot::Boots), 1);
    }

    #[test]
    fn chain_defense_values() {
        assert_eq!(ArmorMaterial::Chain.defense_for_slot(ArmorSlot::Helmet), 2);
        assert_eq!(
            ArmorMaterial::Chain.defense_for_slot(ArmorSlot::Chestplate),
            5
        );
        assert_eq!(
            ArmorMaterial::Chain.defense_for_slot(ArmorSlot::Leggings),
            4
        );
        assert_eq!(ArmorMaterial::Chain.defense_for_slot(ArmorSlot::Boots), 1);
    }

    // -- Toughness ---------------------------------------------------------

    #[test]
    fn toughness_only_for_diamond_and_netherite() {
        assert!((ArmorMaterial::Leather.toughness()).abs() < f32::EPSILON);
        assert!((ArmorMaterial::Chain.toughness()).abs() < f32::EPSILON);
        assert!((ArmorMaterial::Iron.toughness()).abs() < f32::EPSILON);
        assert!((ArmorMaterial::Gold.toughness()).abs() < f32::EPSILON);
        assert!((ArmorMaterial::Diamond.toughness() - 2.0).abs() < f32::EPSILON);
        assert!((ArmorMaterial::Netherite.toughness() - 3.0).abs() < f32::EPSILON);
    }

    // -- Durability --------------------------------------------------------

    #[test]
    fn durability_scales_with_material_and_slot() {
        // Leather helmet: 11 * 5 = 55
        assert_eq!(
            ArmorMaterial::Leather.durability_for_slot(ArmorSlot::Helmet),
            55
        );
        // Iron chestplate: 16 * 15 = 240
        assert_eq!(
            ArmorMaterial::Iron.durability_for_slot(ArmorSlot::Chestplate),
            240
        );
        // Diamond leggings: 15 * 33 = 495
        assert_eq!(
            ArmorMaterial::Diamond.durability_for_slot(ArmorSlot::Leggings),
            495
        );
        // Netherite boots: 13 * 37 = 481
        assert_eq!(
            ArmorMaterial::Netherite.durability_for_slot(ArmorSlot::Boots),
            481
        );
        // Gold helmet: 11 * 7 = 77
        assert_eq!(
            ArmorMaterial::Gold.durability_for_slot(ArmorSlot::Helmet),
            77
        );
    }

    // -- ArmorPiece --------------------------------------------------------

    #[test]
    fn new_piece_starts_at_full_durability() {
        let piece = ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Chestplate);
        assert_eq!(piece.durability, 240);
        assert_eq!(piece.max_durability, 240);
        assert!(!piece.is_broken());
    }

    #[test]
    fn piece_is_broken_at_zero_durability() {
        let piece = ArmorPiece {
            material: ArmorMaterial::Leather,
            slot: ArmorSlot::Boots,
            durability: 0,
            max_durability: 65,
        };
        assert!(piece.is_broken());
    }

    // -- ArmorSet equip / unequip ------------------------------------------

    #[test]
    fn equip_places_piece_in_correct_slot() {
        let mut set = ArmorSet::new();
        let helmet = ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet);
        let prev = set.equip(helmet.clone());
        assert!(prev.is_none());
        assert_eq!(set.slots[0].as_ref().unwrap().material, ArmorMaterial::Iron);
    }

    #[test]
    fn equip_returns_previous_piece() {
        let mut set = ArmorSet::new();
        let iron = ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet);
        let diamond = ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Helmet);

        set.equip(iron);
        let prev = set.equip(diamond);
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().material, ArmorMaterial::Iron);
        assert_eq!(
            set.slots[0].as_ref().unwrap().material,
            ArmorMaterial::Diamond
        );
    }

    #[test]
    fn unequip_removes_and_returns_piece() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Gold, ArmorSlot::Boots));
        let removed = set.unequip(ArmorSlot::Boots);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().material, ArmorMaterial::Gold);
        assert!(set.slots[ArmorSlot::Boots.index()].is_none());
    }

    #[test]
    fn unequip_empty_slot_returns_none() {
        let mut set = ArmorSet::new();
        assert!(set.unequip(ArmorSlot::Leggings).is_none());
    }

    // -- Total defense / toughness -----------------------------------------

    #[test]
    fn total_defense_full_iron_set() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet));
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Chestplate));
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Leggings));
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Boots));
        // 2 + 6 + 5 + 2 = 15
        assert_eq!(set.total_defense(), 15);
    }

    #[test]
    fn total_defense_full_diamond_set() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Helmet));
        set.equip(ArmorPiece::new(
            ArmorMaterial::Diamond,
            ArmorSlot::Chestplate,
        ));
        set.equip(ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Leggings));
        set.equip(ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Boots));
        // 3 + 8 + 6 + 3 = 20
        assert_eq!(set.total_defense(), 20);
    }

    #[test]
    fn total_defense_partial_set() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(
            ArmorMaterial::Diamond,
            ArmorSlot::Chestplate,
        ));
        set.equip(ArmorPiece::new(ArmorMaterial::Leather, ArmorSlot::Boots));
        // 8 + 1 = 9
        assert_eq!(set.total_defense(), 9);
    }

    #[test]
    fn total_defense_excludes_broken_pieces() {
        let mut set = ArmorSet::new();
        let mut broken = ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet);
        broken.durability = 0;
        set.equip(broken);
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Chestplate));
        // Only chestplate counts: 6
        assert_eq!(set.total_defense(), 6);
    }

    #[test]
    fn total_toughness_full_diamond_set() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Helmet));
        set.equip(ArmorPiece::new(
            ArmorMaterial::Diamond,
            ArmorSlot::Chestplate,
        ));
        set.equip(ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Leggings));
        set.equip(ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Boots));
        // 4 pieces * 2.0 = 8.0
        assert!((set.total_toughness() - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn total_toughness_iron_is_zero() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet));
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Chestplate));
        assert!((set.total_toughness()).abs() < f32::EPSILON);
    }

    #[test]
    fn total_toughness_excludes_broken_pieces() {
        let mut set = ArmorSet::new();
        let mut broken = ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Helmet);
        broken.durability = 0;
        set.equip(broken);
        set.equip(ArmorPiece::new(ArmorMaterial::Diamond, ArmorSlot::Boots));
        // Only boots: 2.0
        assert!((set.total_toughness() - 2.0).abs() < f32::EPSILON);
    }

    // -- Damage reduction formula ------------------------------------------

    #[test]
    fn no_armor_means_full_damage() {
        let taken = calculate_damage_reduction(0, 0.0, 10.0);
        assert!((taken - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn full_diamond_reduces_damage() {
        // defense = 20, toughness = 8.0, damage = 10.0
        // effective = max(20/5, 20 - 4*10/(8+8)) = max(4, 20-2.5) = 17.5
        // capped = min(20, 17.5) = 17.5
        // taken = 10.0 * (1 - 17.5/25) = 10.0 * 0.3 = 3.0
        let taken = calculate_damage_reduction(20, 8.0, 10.0);
        assert!((taken - 3.0).abs() < 0.01);
    }

    #[test]
    fn full_iron_reduces_damage() {
        // defense = 15, toughness = 0.0, damage = 10.0
        // effective = max(15/5, 15 - 4*10/(0+8)) = max(3, 15-5) = 10
        // capped = min(20, 10) = 10
        // taken = 10.0 * (1 - 10/25) = 10.0 * 0.6 = 6.0
        let taken = calculate_damage_reduction(15, 0.0, 10.0);
        assert!((taken - 6.0).abs() < 0.01);
    }

    #[test]
    fn toughness_helps_against_high_damage() {
        // Without toughness: defense=20, toughness=0, damage=50
        // effective = max(4, 20-25) = 4
        // taken = 50 * (1 - 4/25) = 50 * 0.84 = 42.0
        let without = calculate_damage_reduction(20, 0.0, 50.0);

        // With toughness: defense=20, toughness=8, damage=50
        // effective = max(4, 20 - 200/16) = max(4, 7.5) = 7.5
        // taken = 50 * (1 - 7.5/25) = 50 * 0.7 = 35.0
        let with = calculate_damage_reduction(20, 8.0, 50.0);

        assert!(with < without, "toughness should reduce more damage");
    }

    #[test]
    fn zero_damage_returns_zero() {
        assert!((calculate_damage_reduction(20, 8.0, 0.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn negative_damage_returns_zero() {
        assert!((calculate_damage_reduction(20, 8.0, -5.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_never_goes_negative() {
        // Even with max defense, damage should not be negative
        let taken = calculate_damage_reduction(100, 100.0, 1.0);
        assert!(taken >= 0.0);
    }

    // -- apply_armor_damage ------------------------------------------------

    #[test]
    fn armor_damage_reduces_durability_by_one() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet));
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Chestplate));

        let helmet_before = set.slots[0].as_ref().unwrap().durability;
        let chest_before = set.slots[1].as_ref().unwrap().durability;

        apply_armor_damage(&mut set, 5.0);

        assert_eq!(set.slots[0].as_ref().unwrap().durability, helmet_before - 1);
        assert_eq!(set.slots[1].as_ref().unwrap().durability, chest_before - 1);
    }

    #[test]
    fn armor_damage_skips_empty_slots() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Boots));
        apply_armor_damage(&mut set, 5.0);
        // Only boots slot occupied; should not panic.
        assert!(set.slots[0].is_none());
        assert!(set.slots[1].is_none());
        assert!(set.slots[2].is_none());
    }

    #[test]
    fn armor_damage_skips_broken_pieces() {
        let mut set = ArmorSet::new();
        let mut broken = ArmorPiece::new(ArmorMaterial::Leather, ArmorSlot::Helmet);
        broken.durability = 0;
        set.equip(broken);

        apply_armor_damage(&mut set, 5.0);
        // Should remain at 0, not underflow
        assert_eq!(set.slots[0].as_ref().unwrap().durability, 0);
    }

    #[test]
    fn armor_damage_zero_damage_does_nothing() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet));
        let before = set.slots[0].as_ref().unwrap().durability;
        apply_armor_damage(&mut set, 0.0);
        assert_eq!(set.slots[0].as_ref().unwrap().durability, before);
    }

    #[test]
    fn armor_damage_negative_damage_does_nothing() {
        let mut set = ArmorSet::new();
        set.equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet));
        let before = set.slots[0].as_ref().unwrap().durability;
        apply_armor_damage(&mut set, -3.0);
        assert_eq!(set.slots[0].as_ref().unwrap().durability, before);
    }

    // -- Default trait -----------------------------------------------------

    #[test]
    fn default_armor_set_is_empty() {
        let set = ArmorSet::default();
        for slot in &set.slots {
            assert!(slot.is_none());
        }
    }
}
