//! Smithing template combinations.
//!
//! Models the 17 smithing templates introduced across the Trails & Tales,
//! Trail Chambers, and Tricky Trials updates plus the Netherite Upgrade
//! template, including their duplication recipes and apply costs.

/// All known smithing template variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SmithingTemplateType {
    NetheriteUpgrade,
    Sentry,
    Dune,
    Coast,
    Wild,
    Ward,
    Eye,
    Vex,
    Tide,
    Snout,
    Rib,
    Silence,
    Wayfinder,
    Raiser,
    Shaper,
    Host,
    Flow,
}

impl SmithingTemplateType {
    /// Display name for the template.
    pub fn name(&self) -> &'static str {
        match self {
            SmithingTemplateType::NetheriteUpgrade => "Netherite Upgrade",
            SmithingTemplateType::Sentry => "Sentry",
            SmithingTemplateType::Dune => "Dune",
            SmithingTemplateType::Coast => "Coast",
            SmithingTemplateType::Wild => "Wild",
            SmithingTemplateType::Ward => "Ward",
            SmithingTemplateType::Eye => "Eye",
            SmithingTemplateType::Vex => "Vex",
            SmithingTemplateType::Tide => "Tide",
            SmithingTemplateType::Snout => "Snout",
            SmithingTemplateType::Rib => "Rib",
            SmithingTemplateType::Silence => "Silence",
            SmithingTemplateType::Wayfinder => "Wayfinder",
            SmithingTemplateType::Raiser => "Raiser",
            SmithingTemplateType::Shaper => "Shaper",
            SmithingTemplateType::Host => "Host",
            SmithingTemplateType::Flow => "Flow",
        }
    }
}

// ── Item ID constants used for duplication recipes ─────────────────────────
/// Diamond item ID per crafting spec.
pub const ITEM_DIAMOND: u16 = 2002;
/// Netherite scrap item ID per crafting spec.
pub const ITEM_NETHERITE_SCRAP: u16 = 2003;

/// Returns the unique item ID assigned to a smithing template (range 6000-6016).
pub fn template_id(t: SmithingTemplateType) -> u16 {
    match t {
        SmithingTemplateType::NetheriteUpgrade => 6000,
        SmithingTemplateType::Sentry => 6001,
        SmithingTemplateType::Dune => 6002,
        SmithingTemplateType::Coast => 6003,
        SmithingTemplateType::Wild => 6004,
        SmithingTemplateType::Ward => 6005,
        SmithingTemplateType::Eye => 6006,
        SmithingTemplateType::Vex => 6007,
        SmithingTemplateType::Tide => 6008,
        SmithingTemplateType::Snout => 6009,
        SmithingTemplateType::Rib => 6010,
        SmithingTemplateType::Silence => 6011,
        SmithingTemplateType::Wayfinder => 6012,
        SmithingTemplateType::Raiser => 6013,
        SmithingTemplateType::Shaper => 6014,
        SmithingTemplateType::Host => 6015,
        SmithingTemplateType::Flow => 6016,
    }
}

/// Returns the duplication recipe for a smithing template.
///
/// The tuple is `(template_to_duplicate_id, secondary_material_id, count)`.
/// `NetheriteUpgrade` is duplicated with netherite scrap; all trim templates
/// use diamonds plus the appropriate cobblestone-variant structure block.
/// Result yields `count` copies (Vanilla: 7 — original returned plus 6 new).
pub fn template_duplication_recipe(t: SmithingTemplateType) -> (u16, u16, u8) {
    let id = template_id(t);
    let secondary = match t {
        SmithingTemplateType::NetheriteUpgrade => ITEM_NETHERITE_SCRAP,
        _ => ITEM_DIAMOND,
    };
    (id, secondary, 7)
}

/// Returns the apply cost at the smithing table.
///
/// Tuple is `(templates, ingots, base_items)` — applying any template costs
/// 1 template, 4 upgrade ingots, and operates on 1 base item.
pub fn template_apply_cost() -> (u8, u8, u8) {
    (1, 4, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [SmithingTemplateType; 17] = [
        SmithingTemplateType::NetheriteUpgrade,
        SmithingTemplateType::Sentry,
        SmithingTemplateType::Dune,
        SmithingTemplateType::Coast,
        SmithingTemplateType::Wild,
        SmithingTemplateType::Ward,
        SmithingTemplateType::Eye,
        SmithingTemplateType::Vex,
        SmithingTemplateType::Tide,
        SmithingTemplateType::Snout,
        SmithingTemplateType::Rib,
        SmithingTemplateType::Silence,
        SmithingTemplateType::Wayfinder,
        SmithingTemplateType::Raiser,
        SmithingTemplateType::Shaper,
        SmithingTemplateType::Host,
        SmithingTemplateType::Flow,
    ];

    #[test]
    fn names_are_non_empty_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for t in ALL {
            let name = t.name();
            assert!(!name.is_empty(), "{:?} has empty name", t);
            assert!(seen.insert(name), "duplicate name: {}", name);
        }
    }

    #[test]
    fn template_ids_cover_6000_through_6016() {
        let ids: Vec<u16> = ALL.iter().copied().map(template_id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        let expected: Vec<u16> = (6000..=6016).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn template_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for t in ALL {
            assert!(seen.insert(template_id(t)), "duplicate id for {:?}", t);
        }
    }

    #[test]
    fn netherite_upgrade_duplicates_with_scrap() {
        let (id, secondary, count) = template_duplication_recipe(SmithingTemplateType::NetheriteUpgrade);
        assert_eq!(id, 6000);
        assert_eq!(secondary, ITEM_NETHERITE_SCRAP);
        assert_eq!(count, 7);
    }

    #[test]
    fn trim_templates_duplicate_with_diamond() {
        for t in ALL {
            if matches!(t, SmithingTemplateType::NetheriteUpgrade) {
                continue;
            }
            let (id, secondary, count) = template_duplication_recipe(t);
            assert_eq!(id, template_id(t));
            assert_eq!(secondary, ITEM_DIAMOND, "{:?} should use diamond", t);
            assert_eq!(secondary, 2002, "diamond id must be 2002");
            assert_eq!(count, 7);
        }
    }

    #[test]
    fn apply_cost_is_one_four_one() {
        assert_eq!(template_apply_cost(), (1, 4, 1));
    }

    #[test]
    fn all_seventeen_variants_present() {
        assert_eq!(ALL.len(), 17);
    }
}
