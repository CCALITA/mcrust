//! Sound subtitle system: queues short labels for recent sounds with fade
//! animation and direction-relative bearing for HUD display.

/// A single subtitle entry shown for a recent sound event.
#[derive(Debug, Clone, PartialEq)]
pub struct Subtitle {
    pub text: String,
    pub source_yaw: f32,
    pub distance: f32,
    pub timer: f32,
    pub max_lifetime: f32,
}

impl Subtitle {
    /// Create a new subtitle with the default 3-second lifetime.
    pub fn new(text: String, source_yaw: f32, distance: f32) -> Self {
        Self {
            text,
            source_yaw,
            distance,
            timer: 0.0,
            max_lifetime: 3.0,
        }
    }
}

/// Bounded queue of currently visible subtitles.
#[derive(Debug, Clone)]
pub struct SubtitleQueue {
    pub active: Vec<Subtitle>,
    pub max_visible: usize,
}

impl SubtitleQueue {
    /// Create a new queue with capacity for 3 visible subtitles.
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            max_visible: 3,
        }
    }

    /// Push a new subtitle, popping the oldest entry when at capacity.
    pub fn push(&mut self, subtitle: Subtitle) {
        if self.active.len() >= self.max_visible {
            self.active.remove(0);
        }
        self.active.push(subtitle);
    }

    /// Advance all subtitle timers by `dt` and remove any that have expired.
    pub fn tick(&mut self, dt: f32) {
        for s in self.active.iter_mut() {
            s.timer += dt;
        }
        self.active.retain(|s| s.timer < s.max_lifetime);
    }
}

impl Default for SubtitleQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a sound id to its display subtitle text.
pub fn subtitle_text_for_sound(sound_id: u16) -> &'static str {
    match sound_id {
        0 => "Footsteps",
        1 => "Block placed",
        2 => "Block broken",
        3 => "Door creaks",
        4 => "Creeper hisses",
        5 => "Zombie groans",
        6 => "Skeleton rattles",
        7 => "Spider hisses",
        8 => "Wind blows",
        9 => "Water flows",
        _ => "Sound",
    }
}

/// Compute the alpha (0.0–1.0) for a subtitle based on its current timer.
///
/// Fades in over the first 0.3s, holds at full opacity, then fades out over
/// the final 0.5s before `max`.
pub fn subtitle_alpha(timer: f32, max: f32) -> f32 {
    if timer <= 0.0 {
        return 0.0;
    }
    if timer >= max {
        return 0.0;
    }
    let fade_in = 0.3_f32;
    let fade_out = 0.5_f32;
    if timer < fade_in {
        return (timer / fade_in).clamp(0.0, 1.0);
    }
    let fade_out_start = max - fade_out;
    if timer >= fade_out_start {
        let remaining = max - timer;
        return (remaining / fade_out).clamp(0.0, 1.0);
    }
    1.0
}

/// Compute the angle (in radians) from the listener's facing direction to the
/// sound source on the horizontal plane. Result is in `(-PI, PI]`.
///
/// `listener_yaw` is in radians, where 0 points along +Z (forward) using the
/// convention that yaw increases clockwise when viewed from above.
pub fn relative_yaw(
    source_pos: [f32; 3],
    listener_pos: [f32; 3],
    listener_yaw: f32,
) -> f32 {
    let dx = source_pos[0] - listener_pos[0];
    let dz = source_pos[2] - listener_pos[2];
    let source_angle = dx.atan2(dz);
    let mut diff = source_angle - listener_yaw;
    let pi = std::f32::consts::PI;
    let two_pi = 2.0 * pi;
    while diff > pi {
        diff -= two_pi;
    }
    while diff <= -pi {
        diff += two_pi;
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_new_sets_default_lifetime() {
        let s = Subtitle::new("Test".to_string(), 0.0, 5.0);
        assert_eq!(s.max_lifetime, 3.0);
        assert_eq!(s.timer, 0.0);
        assert_eq!(s.text, "Test");
    }

    #[test]
    fn queue_push_respects_max_visible() {
        let mut q = SubtitleQueue::new();
        q.push(Subtitle::new("a".into(), 0.0, 1.0));
        q.push(Subtitle::new("b".into(), 0.0, 1.0));
        q.push(Subtitle::new("c".into(), 0.0, 1.0));
        q.push(Subtitle::new("d".into(), 0.0, 1.0));
        assert_eq!(q.active.len(), 3);
        assert_eq!(q.active[0].text, "b");
        assert_eq!(q.active[2].text, "d");
    }

    #[test]
    fn queue_tick_removes_expired() {
        let mut q = SubtitleQueue::new();
        q.push(Subtitle::new("a".into(), 0.0, 1.0));
        q.push(Subtitle::new("b".into(), 0.0, 1.0));
        q.tick(1.0);
        assert_eq!(q.active.len(), 2);
        assert!((q.active[0].timer - 1.0).abs() < 1e-6);
        q.tick(2.5);
        assert_eq!(q.active.len(), 0);
    }

    #[test]
    fn alpha_curve_fades_in_holds_and_fades_out() {
        let max = 3.0;
        assert!((subtitle_alpha(0.0, max) - 0.0).abs() < 1e-6);
        assert!((subtitle_alpha(0.15, max) - 0.5).abs() < 1e-6);
        assert!((subtitle_alpha(0.3, max) - 1.0).abs() < 1e-6);
        assert!((subtitle_alpha(1.5, max) - 1.0).abs() < 1e-6);
        // fade out begins at max - 0.5 = 2.5
        assert!((subtitle_alpha(2.5, max) - 1.0).abs() < 1e-6);
        assert!((subtitle_alpha(2.75, max) - 0.5).abs() < 1e-6);
        assert!((subtitle_alpha(3.0, max) - 0.0).abs() < 1e-6);
        assert!((subtitle_alpha(3.5, max) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn text_lookup_known_and_default() {
        assert_eq!(subtitle_text_for_sound(0), "Footsteps");
        assert_eq!(subtitle_text_for_sound(4), "Creeper hisses");
        assert_eq!(subtitle_text_for_sound(9), "Water flows");
        assert_eq!(subtitle_text_for_sound(999), "Sound");
    }

    #[test]
    fn relative_yaw_directly_ahead_is_zero() {
        let yaw = relative_yaw([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 0.0);
        assert!(yaw.abs() < 1e-6);
    }

    #[test]
    fn relative_yaw_to_the_right() {
        let yaw = relative_yaw([5.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.0);
        assert!((yaw - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn relative_yaw_behind_is_pi() {
        let yaw = relative_yaw([0.0, 0.0, -5.0], [0.0, 0.0, 0.0], 0.0);
        assert!((yaw.abs() - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn relative_yaw_accounts_for_listener_facing() {
        // Source ahead of world but listener facing right -> source is to the left
        let yaw = relative_yaw(
            [0.0, 0.0, 5.0],
            [0.0, 0.0, 0.0],
            std::f32::consts::FRAC_PI_2,
        );
        assert!((yaw + std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn relative_yaw_wraps_to_pi_range() {
        let yaw = relative_yaw([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], -3.0 * std::f32::consts::PI);
        assert!(yaw > -std::f32::consts::PI && yaw <= std::f32::consts::PI);
    }
}
