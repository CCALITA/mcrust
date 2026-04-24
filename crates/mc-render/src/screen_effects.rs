//! Screen shake and motion blur post-processing effects.

/// Camera shake triggered by damage, explosions, or other events.
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub frequency: f32,
}

impl ScreenShake {
    /// Create a new screen shake with the given intensity and duration.
    /// Frequency defaults to 30.0 Hz.
    pub fn new(intensity: f32, duration: f32) -> Self {
        Self {
            intensity,
            duration,
            elapsed: 0.0,
            frequency: 30.0,
        }
    }

    /// Whether the shake is still active (has not yet expired).
    pub fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }

    /// Advance the shake by `dt` seconds and return the `[x, y]` camera offset.
    ///
    /// The offset is based on sin/cos of `time * frequency` and decays
    /// linearly from full intensity to zero over the duration.
    pub fn tick(&mut self, dt: f32, time: f32) -> [f32; 2] {
        self.elapsed += dt;

        if !self.is_active() {
            return [0.0, 0.0];
        }

        let progress = self.elapsed / self.duration;
        let decay = 1.0 - progress;
        let angle = time * self.frequency;

        [
            self.intensity * decay * angle.sin(),
            self.intensity * decay * angle.cos(),
        ]
    }

    /// Create a shake appropriate for taking `damage` hit points.
    ///
    /// Intensity is capped at 1.0 (for damage >= 10), duration is 0.3 s.
    pub fn damage_shake(damage: f32) -> Self {
        let intensity = (damage / 10.0).min(1.0);
        Self {
            intensity,
            duration: 0.3,
            elapsed: 0.0,
            frequency: 30.0,
        }
    }

    /// Create a shake for an explosion at `distance` blocks away with
    /// the given `blast_power`.
    ///
    /// Intensity falls off as `blast_power / (distance + 1)`, duration is 0.5 s.
    pub fn explosion_shake(distance: f32, blast_power: f32) -> Self {
        let intensity = blast_power / (distance + 1.0);
        Self {
            intensity,
            duration: 0.5,
            elapsed: 0.0,
            frequency: 30.0,
        }
    }
}

/// Velocity-based motion blur parameters for the post-process pipeline.
pub struct MotionBlur {
    pub strength: f32,
    pub samples: u8,
}

impl MotionBlur {
    /// Create a new motion blur with zero strength and 4 samples.
    pub fn new() -> Self {
        Self {
            strength: 0.0,
            samples: 4,
        }
    }

    /// Update the blur strength from the player's velocity magnitude.
    ///
    /// Strength is `(velocity / 20).min(1.0)`.
    pub fn set_velocity(&mut self, velocity_magnitude: f32) {
        self.strength = (velocity_magnitude / 20.0).min(1.0);
    }
}

impl Default for MotionBlur {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_decays_over_duration() {
        let mut shake = ScreenShake::new(1.0, 1.0);
        assert!(shake.is_active());

        // At t=0, offset should be non-trivial for a non-zero time value
        let offset_early = shake.tick(0.1, 1.0);
        // Decay factor at elapsed=0.1, duration=1.0 is 0.9, so magnitude > 0
        let mag_early = (offset_early[0].powi(2) + offset_early[1].powi(2)).sqrt();
        assert!(mag_early > 0.0);

        // Advance past the duration
        let offset_late = shake.tick(1.0, 2.0);
        assert!(!shake.is_active());
        assert_eq!(offset_late, [0.0, 0.0]);
    }

    #[test]
    fn damage_shake_intensity_caps_at_one() {
        let shake_low = ScreenShake::damage_shake(5.0);
        assert!((shake_low.intensity - 0.5).abs() < f32::EPSILON);

        let shake_exact = ScreenShake::damage_shake(10.0);
        assert!((shake_exact.intensity - 1.0).abs() < f32::EPSILON);

        let shake_high = ScreenShake::damage_shake(100.0);
        assert!((shake_high.intensity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn explosion_shake_distance_falloff() {
        let close = ScreenShake::explosion_shake(0.0, 4.0);
        let far = ScreenShake::explosion_shake(9.0, 4.0);

        // At distance 0: intensity = 4 / (0+1) = 4.0
        assert!((close.intensity - 4.0).abs() < f32::EPSILON);
        // At distance 9: intensity = 4 / (9+1) = 0.4
        assert!((far.intensity - 0.4).abs() < f32::EPSILON);

        assert!(close.intensity > far.intensity);
    }

    #[test]
    fn motion_blur_velocity_scaling() {
        let mut blur = MotionBlur::new();
        assert!((blur.strength - 0.0).abs() < f32::EPSILON);
        assert_eq!(blur.samples, 4);

        blur.set_velocity(10.0);
        assert!((blur.strength - 0.5).abs() < f32::EPSILON);

        blur.set_velocity(20.0);
        assert!((blur.strength - 1.0).abs() < f32::EPSILON);

        // Capped at 1.0
        blur.set_velocity(100.0);
        assert!((blur.strength - 1.0).abs() < f32::EPSILON);
    }
}
