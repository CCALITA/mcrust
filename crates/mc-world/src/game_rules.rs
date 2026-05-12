//! Minecraft game rules that control world behaviour.
//!
//! Each rule is a boolean toggle matching vanilla Minecraft defaults.
//! Rules are accessed by name for command-line `/gamerule` integration.

/// Boolean game rules controlling world behaviour.
///
/// All fields correspond to vanilla Minecraft game rules.
/// Use [`GameRules::default()`] for vanilla defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRules {
    pub keep_inventory: bool,
    pub mob_griefing: bool,
    pub do_daylight_cycle: bool,
    pub do_weather_cycle: bool,
    pub natural_regeneration: bool,
    pub pvp: bool,
    pub fire_tick: bool,
    pub mob_spawning: bool,
    pub do_insomnia: bool,
    pub do_patrol_spawning: bool,
    pub announce_advancements: bool,
    pub do_immediate_respawn: bool,
}

impl Default for GameRules {
    /// Vanilla Minecraft defaults: most rules `true`, except
    /// `keep_inventory` and `do_immediate_respawn` which are `false`.
    fn default() -> Self {
        Self {
            keep_inventory: false,
            mob_griefing: true,
            do_daylight_cycle: true,
            do_weather_cycle: true,
            natural_regeneration: true,
            pvp: true,
            fire_tick: true,
            mob_spawning: true,
            do_insomnia: true,
            do_patrol_spawning: true,
            announce_advancements: true,
            do_immediate_respawn: false,
        }
    }
}

/// Set a game rule by name, returning a new `GameRules` with the updated value.
///
/// Returns `true` if the rule name was recognised, `false` otherwise
/// (in which case the returned `GameRules` is unchanged).
pub fn set_rule(rules: &GameRules, name: &str, value: bool) -> (GameRules, bool) {
    let mut updated = rules.clone();
    let found = match name {
        "keepInventory" => {
            updated.keep_inventory = value;
            true
        }
        "mobGriefing" => {
            updated.mob_griefing = value;
            true
        }
        "doDaylightCycle" => {
            updated.do_daylight_cycle = value;
            true
        }
        "doWeatherCycle" => {
            updated.do_weather_cycle = value;
            true
        }
        "naturalRegeneration" => {
            updated.natural_regeneration = value;
            true
        }
        "pvp" => {
            updated.pvp = value;
            true
        }
        "doFireTick" => {
            updated.fire_tick = value;
            true
        }
        "doMobSpawning" => {
            updated.mob_spawning = value;
            true
        }
        "doInsomnia" => {
            updated.do_insomnia = value;
            true
        }
        "doPatrolSpawning" => {
            updated.do_patrol_spawning = value;
            true
        }
        "announceAdvancements" => {
            updated.announce_advancements = value;
            true
        }
        "doImmediateRespawn" => {
            updated.do_immediate_respawn = value;
            true
        }
        _ => false,
    };
    (updated, found)
}

/// Look up a game rule by its Minecraft camelCase name.
///
/// Returns `None` if the name is not recognised.
pub fn get_rule(rules: &GameRules, name: &str) -> Option<bool> {
    match name {
        "keepInventory" => Some(rules.keep_inventory),
        "mobGriefing" => Some(rules.mob_griefing),
        "doDaylightCycle" => Some(rules.do_daylight_cycle),
        "doWeatherCycle" => Some(rules.do_weather_cycle),
        "naturalRegeneration" => Some(rules.natural_regeneration),
        "pvp" => Some(rules.pvp),
        "doFireTick" => Some(rules.fire_tick),
        "doMobSpawning" => Some(rules.mob_spawning),
        "doInsomnia" => Some(rules.do_insomnia),
        "doPatrolSpawning" => Some(rules.do_patrol_spawning),
        "announceAdvancements" => Some(rules.announce_advancements),
        "doImmediateRespawn" => Some(rules.do_immediate_respawn),
        _ => None,
    }
}

/// List all game rules with their current values.
///
/// Returns pairs of `(name, value)` in alphabetical order by name.
pub fn list_rules(rules: &GameRules) -> Vec<(&str, bool)> {
    vec![
        ("announceAdvancements", rules.announce_advancements),
        ("doDaylightCycle", rules.do_daylight_cycle),
        ("doFireTick", rules.fire_tick),
        ("doImmediateRespawn", rules.do_immediate_respawn),
        ("doInsomnia", rules.do_insomnia),
        ("doMobSpawning", rules.mob_spawning),
        ("doPatrolSpawning", rules.do_patrol_spawning),
        ("doWeatherCycle", rules.do_weather_cycle),
        ("keepInventory", rules.keep_inventory),
        ("mobGriefing", rules.mob_griefing),
        ("naturalRegeneration", rules.natural_regeneration),
        ("pvp", rules.pvp),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_vanilla_values() {
        let rules = GameRules::default();
        assert!(!rules.keep_inventory);
        assert!(rules.mob_griefing);
        assert!(rules.do_daylight_cycle);
        assert!(rules.do_weather_cycle);
        assert!(rules.natural_regeneration);
        assert!(rules.pvp);
        assert!(rules.fire_tick);
        assert!(rules.mob_spawning);
        assert!(rules.do_insomnia);
        assert!(rules.do_patrol_spawning);
        assert!(rules.announce_advancements);
        assert!(!rules.do_immediate_respawn);
    }

    #[test]
    fn set_rule_updates_known_rule() {
        let rules = GameRules::default();
        let (updated, found) = set_rule(&rules, "keepInventory", true);
        assert!(found);
        assert!(updated.keep_inventory);
        // Original unchanged (immutable pattern).
        assert!(!rules.keep_inventory);
    }

    #[test]
    fn set_rule_returns_false_for_unknown() {
        let rules = GameRules::default();
        let (updated, found) = set_rule(&rules, "notARealRule", true);
        assert!(!found);
        assert_eq!(updated, rules);
    }

    #[test]
    fn get_rule_returns_value_for_known_rules() {
        let rules = GameRules::default();
        assert_eq!(get_rule(&rules, "keepInventory"), Some(false));
        assert_eq!(get_rule(&rules, "mobGriefing"), Some(true));
        assert_eq!(get_rule(&rules, "doDaylightCycle"), Some(true));
        assert_eq!(get_rule(&rules, "doWeatherCycle"), Some(true));
        assert_eq!(get_rule(&rules, "naturalRegeneration"), Some(true));
        assert_eq!(get_rule(&rules, "pvp"), Some(true));
        assert_eq!(get_rule(&rules, "doFireTick"), Some(true));
        assert_eq!(get_rule(&rules, "doMobSpawning"), Some(true));
        assert_eq!(get_rule(&rules, "doInsomnia"), Some(true));
        assert_eq!(get_rule(&rules, "doPatrolSpawning"), Some(true));
        assert_eq!(get_rule(&rules, "announceAdvancements"), Some(true));
        assert_eq!(get_rule(&rules, "doImmediateRespawn"), Some(false));
    }

    #[test]
    fn get_rule_returns_none_for_unknown() {
        let rules = GameRules::default();
        assert_eq!(get_rule(&rules, "notARealRule"), None);
    }

    #[test]
    fn get_rule_reflects_set_rule_changes() {
        let rules = GameRules::default();
        let (updated, _) = set_rule(&rules, "mobGriefing", false);
        assert_eq!(get_rule(&updated, "mobGriefing"), Some(false));
    }

    #[test]
    fn list_rules_returns_all_rules_sorted() {
        let rules = GameRules::default();
        let list = list_rules(&rules);
        assert_eq!(list.len(), 12);

        // Verify alphabetical order.
        let names: Vec<&str> = list.iter().map(|(n, _)| *n).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn list_rules_reflects_current_values() {
        let rules = GameRules::default();
        let (updated, _) = set_rule(&rules, "pvp", false);
        let list = list_rules(&updated);
        let pvp_entry = list.iter().find(|(n, _)| *n == "pvp");
        assert_eq!(pvp_entry, Some(&("pvp", false)));
    }

    #[test]
    fn set_rule_covers_all_rules() {
        let rules = GameRules::default();
        let rule_names = [
            "keepInventory",
            "mobGriefing",
            "doDaylightCycle",
            "doWeatherCycle",
            "naturalRegeneration",
            "pvp",
            "doFireTick",
            "doMobSpawning",
            "doInsomnia",
            "doPatrolSpawning",
            "announceAdvancements",
            "doImmediateRespawn",
        ];
        for name in &rule_names {
            let (_, found) = set_rule(&rules, name, true);
            assert!(found, "set_rule should recognise {name}");
        }
    }

    #[test]
    fn set_rule_does_not_mutate_other_fields() {
        let rules = GameRules::default();
        let (updated, _) = set_rule(&rules, "keepInventory", true);
        // Only keep_inventory should differ.
        assert!(updated.keep_inventory);
        assert_eq!(updated.mob_griefing, rules.mob_griefing);
        assert_eq!(updated.do_daylight_cycle, rules.do_daylight_cycle);
        assert_eq!(updated.do_weather_cycle, rules.do_weather_cycle);
        assert_eq!(updated.natural_regeneration, rules.natural_regeneration);
        assert_eq!(updated.pvp, rules.pvp);
        assert_eq!(updated.fire_tick, rules.fire_tick);
        assert_eq!(updated.mob_spawning, rules.mob_spawning);
        assert_eq!(updated.do_insomnia, rules.do_insomnia);
        assert_eq!(updated.do_patrol_spawning, rules.do_patrol_spawning);
        assert_eq!(updated.announce_advancements, rules.announce_advancements);
        assert_eq!(updated.do_immediate_respawn, rules.do_immediate_respawn);
    }
}
