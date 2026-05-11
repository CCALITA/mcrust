//! Camel dash mechanics, sitting behavior, and passenger management.

/// State for a camel entity.
#[derive(Debug, Clone, PartialEq)]
pub struct CamelState {
    pub sitting: bool,
    pub dash_cooldown: f32,
    pub dash_active: bool,
    pub passengers: u8,
}

impl CamelState {
    pub fn new() -> Self {
        Self {
            sitting: false,
            dash_cooldown: 0.0,
            dash_active: false,
            passengers: 0,
        }
    }
}

/// Maximum number of passengers a camel can carry.
pub fn camel_max_passengers() -> u8 {
    2
}

/// Speed multiplier applied during a dash.
pub fn camel_dash_boost() -> f32 {
    1.5
}

/// Cooldown duration in seconds between dashes.
pub fn camel_dash_cooldown() -> f32 {
    2.75
}

/// Block height a camel can step up without jumping.
pub fn camel_step_height() -> f32 {
    1.5
}

/// Duration in seconds a camel remains sitting before standing.
pub fn camel_sit_duration() -> f32 {
    5.0
}

/// Tick the camel state and return a velocity delta `[dx, dy, dz]`.
///
/// `dt` is the time step in seconds. `dash_input` is true when the player
/// requests a dash this tick.
pub fn tick_camel(state: &mut CamelState, dt: f32, dash_input: bool) -> [f32; 3] {
    // Reduce cooldown
    if state.dash_cooldown > 0.0 {
        state.dash_cooldown = (state.dash_cooldown - dt).max(0.0);
    }

    // Sitting camels cannot dash
    if state.sitting {
        state.dash_active = false;
        return [0.0, 0.0, 0.0];
    }

    // Start dash if input and cooldown elapsed
    if dash_input && state.dash_cooldown <= 0.0 && !state.dash_active {
        state.dash_active = true;
    }

    if state.dash_active {
        state.dash_active = false;
        state.dash_cooldown = camel_dash_cooldown();
        [0.0, 0.0, camel_dash_boost()]
    } else {
        [0.0, 0.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_defaults() {
        let s = CamelState::new();
        assert!(!s.sitting);
        assert_eq!(s.dash_cooldown, 0.0);
        assert!(!s.dash_active);
        assert_eq!(s.passengers, 0);
    }

    #[test]
    fn constants() {
        assert_eq!(camel_max_passengers(), 2);
        assert_eq!(camel_dash_boost(), 1.5);
        assert_eq!(camel_dash_cooldown(), 2.75);
        assert_eq!(camel_step_height(), 1.5);
        assert_eq!(camel_sit_duration(), 5.0);
    }

    #[test]
    fn dash_produces_boost() {
        let mut s = CamelState::new();
        let vel = tick_camel(&mut s, 0.05, true);
        assert_eq!(vel, [0.0, 0.0, 1.5]);
        assert!(!s.dash_active);
        assert_eq!(s.dash_cooldown, 2.75);
    }

    #[test]
    fn dash_cooldown_prevents_immediate_redash() {
        let mut s = CamelState::new();
        tick_camel(&mut s, 0.05, true);
        let vel = tick_camel(&mut s, 0.05, true);
        assert_eq!(vel, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn cooldown_decreases_over_time() {
        let mut s = CamelState::new();
        tick_camel(&mut s, 0.05, true);
        tick_camel(&mut s, 1.0, false);
        assert!((s.dash_cooldown - 1.75).abs() < 0.01);
    }

    #[test]
    fn sitting_prevents_dash() {
        let mut s = CamelState::new();
        s.sitting = true;
        let vel = tick_camel(&mut s, 0.05, true);
        assert_eq!(vel, [0.0, 0.0, 0.0]);
        assert!(!s.dash_active);
    }

    #[test]
    fn no_input_no_dash() {
        let mut s = CamelState::new();
        let vel = tick_camel(&mut s, 0.05, false);
        assert_eq!(vel, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn cooldown_expires_allows_redash() {
        let mut s = CamelState::new();
        tick_camel(&mut s, 0.05, true);
        tick_camel(&mut s, 3.0, false);
        assert_eq!(s.dash_cooldown, 0.0);
        let vel = tick_camel(&mut s, 0.05, true);
        assert_eq!(vel, [0.0, 0.0, 1.5]);
    }
}
