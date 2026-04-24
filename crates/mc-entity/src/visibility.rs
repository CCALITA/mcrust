//! Player visibility, sneak fade, and invisibility-effect rendering helpers.
//!
//! Provides per-player visibility state and pure helper functions used by the
//! rendering and HUD layers to compute alpha values, nameplate visibility,
//! and armor see-through behavior under the invisibility status effect.

/// Per-player visibility state tracked across ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerVisibility {
    /// Current render alpha for the player body (0.0..=1.0).
    pub alpha: f32,
    /// Total time (in seconds) the player has been continuously sneaking.
    pub sneak_time: f32,
    /// Remaining duration (in seconds) of the invisibility status effect.
    pub invisibility_remaining: f32,
}

impl PlayerVisibility {
    /// Construct a fully-visible player with no sneak history.
    pub fn new() -> Self {
        Self {
            alpha: 1.0,
            sneak_time: 0.0,
            invisibility_remaining: 0.0,
        }
    }
}

impl Default for PlayerVisibility {
    fn default() -> Self {
        Self::new()
    }
}

/// Advance the visibility state by `dt` seconds.
///
/// Accumulates sneak time while sneaking and resets it when not sneaking.
/// While the invisibility effect is active the body alpha drops to 0,
/// otherwise it returns to fully visible.
pub fn tick_visibility(
    state: &mut PlayerVisibility,
    sneaking: bool,
    has_invisibility: bool,
    dt: f32,
) {
    if sneaking {
        state.sneak_time += dt.max(0.0);
    } else {
        state.sneak_time = 0.0;
    }

    if has_invisibility {
        state.alpha = 0.0;
    } else {
        state.alpha = 1.0;
    }
}

/// Whether the player nameplate should be hidden.
///
/// Hidden when the player is sneaking and farther than 4 blocks away,
/// or whenever invisibility is active.
pub fn should_hide_nameplate(distance: f32, sneaking: bool, has_invisibility: bool) -> bool {
    has_invisibility || (sneaking && distance > 4.0)
}

/// Compute the render alpha for an entity body, combining fog distance fade
/// with the invisibility effect (which forces alpha to 0).
pub fn entity_render_alpha(distance: f32, fog_end: f32, has_invisibility: bool) -> f32 {
    if has_invisibility {
        return 0.0;
    }
    if fog_end <= 0.0 {
        return 1.0;
    }
    let fog_start = fog_end * 0.5;
    if distance <= fog_start {
        1.0
    } else if distance >= fog_end {
        0.0
    } else {
        let span = fog_end - fog_start;
        1.0 - ((distance - fog_start) / span)
    }
}

/// Armor remains partially visible even when the wearer is invisible
/// (vanilla behavior — armor + held items still render).
pub fn armor_visibility_with_invisibility(invisibility: bool) -> f32 {
    if invisibility { 0.6 } else { 1.0 }
}

/// Linear nameplate fade: fully opaque within 32 blocks, faded to 0 by 64.
pub fn nameplate_distance_alpha(distance: f32) -> f32 {
    if distance <= 32.0 {
        1.0
    } else if distance >= 64.0 {
        0.0
    } else {
        1.0 - ((distance - 32.0) / 32.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_is_fully_visible() {
        let v = PlayerVisibility::new();
        assert_eq!(v.alpha, 1.0);
        assert_eq!(v.sneak_time, 0.0);
        assert_eq!(v.invisibility_remaining, 0.0);
    }

    #[test]
    fn invisibility_drops_alpha_to_zero() {
        let mut v = PlayerVisibility::new();
        tick_visibility(&mut v, false, true, 0.05);
        assert_eq!(v.alpha, 0.0);
    }

    #[test]
    fn alpha_restores_after_invisibility_ends() {
        let mut v = PlayerVisibility::new();
        tick_visibility(&mut v, false, true, 0.05);
        assert_eq!(v.alpha, 0.0);
        tick_visibility(&mut v, false, false, 0.05);
        assert_eq!(v.alpha, 1.0);
    }

    #[test]
    fn sneak_time_accumulates_while_sneaking() {
        let mut v = PlayerVisibility::new();
        tick_visibility(&mut v, true, false, 0.1);
        tick_visibility(&mut v, true, false, 0.2);
        assert!((v.sneak_time - 0.3).abs() < 1e-6);
    }

    #[test]
    fn sneak_time_resets_when_not_sneaking() {
        let mut v = PlayerVisibility::new();
        tick_visibility(&mut v, true, false, 0.5);
        tick_visibility(&mut v, false, false, 0.1);
        assert_eq!(v.sneak_time, 0.0);
    }

    #[test]
    fn nameplate_hidden_when_sneaking_far_away() {
        assert!(should_hide_nameplate(5.0, true, false));
        assert!(!should_hide_nameplate(3.0, true, false));
    }

    #[test]
    fn nameplate_always_hidden_with_invisibility() {
        assert!(should_hide_nameplate(1.0, false, true));
        assert!(should_hide_nameplate(100.0, false, true));
    }

    #[test]
    fn nameplate_visible_when_close_and_not_sneaking() {
        assert!(!should_hide_nameplate(2.0, false, false));
        assert!(!should_hide_nameplate(50.0, false, false));
    }

    #[test]
    fn entity_alpha_zero_when_invisible() {
        assert_eq!(entity_render_alpha(10.0, 100.0, true), 0.0);
    }

    #[test]
    fn entity_alpha_full_within_fog_start() {
        assert_eq!(entity_render_alpha(10.0, 100.0, false), 1.0);
    }

    #[test]
    fn entity_alpha_zero_beyond_fog_end() {
        assert_eq!(entity_render_alpha(150.0, 100.0, false), 0.0);
    }

    #[test]
    fn entity_alpha_fades_within_fog_range() {
        let a = entity_render_alpha(75.0, 100.0, false);
        assert!(a > 0.0 && a < 1.0);
        assert!((a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn armor_stays_partially_visible_when_invisible() {
        assert_eq!(armor_visibility_with_invisibility(true), 0.6);
        assert_eq!(armor_visibility_with_invisibility(false), 1.0);
    }

    #[test]
    fn nameplate_alpha_full_under_32() {
        assert_eq!(nameplate_distance_alpha(10.0), 1.0);
        assert_eq!(nameplate_distance_alpha(32.0), 1.0);
    }

    #[test]
    fn nameplate_alpha_zero_above_64() {
        assert_eq!(nameplate_distance_alpha(64.0), 0.0);
        assert_eq!(nameplate_distance_alpha(100.0), 0.0);
    }

    #[test]
    fn nameplate_alpha_fades_between_32_and_64() {
        let a = nameplate_distance_alpha(48.0);
        assert!((a - 0.5).abs() < 1e-6);
    }
}
