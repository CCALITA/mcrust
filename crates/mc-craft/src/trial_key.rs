//! Trial key items for unlocking trial vaults.

/// A trial key that can be normal or ominous.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialKey {
    pub ominous: bool,
}

impl TrialKey {
    /// Creates a normal trial key.
    pub fn normal() -> Self {
        Self { ominous: false }
    }

    /// Creates an ominous trial key.
    pub fn ominous() -> Self {
        Self { ominous: true }
    }
}

/// Returns the item ID for a trial key (9000 for normal, 9001 for ominous).
pub fn trial_key_item_id(ominous: bool) -> u16 {
    if ominous { 9001 } else { 9000 }
}

/// Returns whether the key can open the vault (ominous must match).
pub fn can_open_vault(key: &TrialKey, vault_ominous: bool) -> bool {
    key.ominous == vault_ominous
}

/// Creates a trial key from a spawner's ominous state.
pub fn trial_key_from_spawner(spawner_ominous: bool) -> TrialKey {
    TrialKey { ominous: spawner_ominous }
}

/// Returns the display name for a trial key.
pub fn trial_key_name(ominous: bool) -> &'static str {
    if ominous { "Ominous Trial Key" } else { "Trial Key" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_key_is_not_ominous() {
        let key = TrialKey::normal();
        assert!(!key.ominous);
    }

    #[test]
    fn ominous_key_is_ominous() {
        let key = TrialKey::ominous();
        assert!(key.ominous);
    }

    #[test]
    fn item_ids_are_correct() {
        assert_eq!(trial_key_item_id(false), 9000);
        assert_eq!(trial_key_item_id(true), 9001);
    }

    #[test]
    fn normal_key_opens_normal_vault() {
        let key = TrialKey::normal();
        assert!(can_open_vault(&key, false));
    }

    #[test]
    fn normal_key_cannot_open_ominous_vault() {
        let key = TrialKey::normal();
        assert!(!can_open_vault(&key, true));
    }

    #[test]
    fn ominous_key_opens_ominous_vault() {
        let key = TrialKey::ominous();
        assert!(can_open_vault(&key, true));
    }

    #[test]
    fn ominous_key_cannot_open_normal_vault() {
        let key = TrialKey::ominous();
        assert!(!can_open_vault(&key, false));
    }

    #[test]
    fn spawner_creates_matching_key() {
        let normal = trial_key_from_spawner(false);
        assert!(!normal.ominous);
        let ominous = trial_key_from_spawner(true);
        assert!(ominous.ominous);
    }

    #[test]
    fn names_are_correct() {
        assert_eq!(trial_key_name(false), "Trial Key");
        assert_eq!(trial_key_name(true), "Ominous Trial Key");
    }
}
