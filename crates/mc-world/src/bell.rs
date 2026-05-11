//! Bell ringing mechanics.

/// State of a bell block.
#[derive(Debug, Clone, PartialEq)]
pub struct BellState {
    pub ringing: bool,
    pub ring_timer: f32,
    pub direction: u8,
}

impl BellState {
    pub fn new() -> Self {
        Self {
            ringing: false,
            ring_timer: 0.0,
            direction: 0,
        }
    }
}

/// Duration of a bell ring in seconds.
pub fn bell_ring_duration() -> f32 {
    1.5
}

/// Range in blocks at which a bell alerts entities.
pub fn bell_alert_range() -> f32 {
    32.0
}

/// Whether a ringing bell highlights raids.
pub fn bell_highlights_raids(ringing: bool) -> bool {
    ringing
}

/// Start ringing the bell from the given hit direction.
pub fn ring_bell(state: &mut BellState, hit_direction: u8) {
    state.ringing = true;
    state.ring_timer = 0.0;
    state.direction = hit_direction;
}

/// Tick the bell state, stopping after the ring duration elapses.
pub fn tick_bell(state: &mut BellState, dt: f32) {
    if !state.ringing {
        return;
    }
    state.ring_timer += dt;
    if state.ring_timer >= bell_ring_duration() {
        state.ringing = false;
        state.ring_timer = bell_ring_duration();
    }
}

/// Compute the swing angle of the bell based on elapsed timer.
/// Returns a decaying sinusoidal angle in radians.
pub fn bell_swing_angle(timer: f32) -> f32 {
    let duration = bell_ring_duration();
    if timer >= duration {
        return 0.0;
    }
    let progress = timer / duration;
    let decay = 1.0 - progress;
    let frequency = std::f32::consts::PI * 8.0;
    decay * (timer * frequency).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = BellState::new();
        assert!(!state.ringing);
        assert_eq!(state.ring_timer, 0.0);
        assert_eq!(state.direction, 0);
    }

    #[test]
    fn test_ring_bell() {
        let mut state = BellState::new();
        ring_bell(&mut state, 3);
        assert!(state.ringing);
        assert_eq!(state.ring_timer, 0.0);
        assert_eq!(state.direction, 3);
    }

    #[test]
    fn test_tick_bell_stops_after_duration() {
        let mut state = BellState::new();
        ring_bell(&mut state, 1);
        tick_bell(&mut state, 1.0);
        assert!(state.ringing);
        tick_bell(&mut state, 0.6);
        assert!(!state.ringing);
    }

    #[test]
    fn test_tick_bell_noop_when_not_ringing() {
        let mut state = BellState::new();
        tick_bell(&mut state, 1.0);
        assert!(!state.ringing);
        assert_eq!(state.ring_timer, 0.0);
    }

    #[test]
    fn test_bell_ring_duration() {
        assert_eq!(bell_ring_duration(), 1.5);
    }

    #[test]
    fn test_bell_alert_range() {
        assert_eq!(bell_alert_range(), 32.0);
    }

    #[test]
    fn test_bell_highlights_raids() {
        assert!(bell_highlights_raids(true));
        assert!(!bell_highlights_raids(false));
    }

    #[test]
    fn test_bell_swing_angle_at_zero() {
        // At timer=0, sin(0)=0 so angle is 0
        assert_eq!(bell_swing_angle(0.0), 0.0);
    }

    #[test]
    fn test_bell_swing_angle_at_end() {
        assert_eq!(bell_swing_angle(1.5), 0.0);
        assert_eq!(bell_swing_angle(2.0), 0.0);
    }

    #[test]
    fn test_bell_swing_angle_decays() {
        let early = bell_swing_angle(0.1).abs();
        let late = bell_swing_angle(1.3).abs();
        // Early swing should generally have larger amplitude than late
        assert!(early > late || late < 0.1);
    }
}
