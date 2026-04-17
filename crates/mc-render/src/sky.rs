//! Day/night cycle and sky rendering uniforms.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Duration of a full day/night cycle in seconds (20 minutes).
const CYCLE_DURATION: f32 = 1200.0;

/// GPU-uploadable sky uniform data.
///
/// Layout: 3 floats sky_color + 1 padding, 3 floats sun_dir + 1 ambient.
/// Total: 32 bytes (two `vec4<f32>`).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SkyUniform {
    pub sky_color: [f32; 3],
    pub _pad0: f32,
    pub sun_dir: [f32; 3],
    pub ambient: f32,
}

/// Tracks the time of day and provides sky parameters for rendering.
///
/// `time_of_day` ranges from 0.0 to 1.0:
/// - 0.00 = midnight
/// - 0.25 = sunrise
/// - 0.50 = noon
/// - 0.75 = sunset
#[derive(Debug, Clone)]
pub struct DayNightCycle {
    /// Current time of day in the range `[0.0, 1.0)`.
    pub time_of_day: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 0.5, // Start at noon
        }
    }
}

impl DayNightCycle {
    /// Create a new cycle at the given time of day (clamped to `[0.0, 1.0)`).
    pub fn new(time_of_day: f32) -> Self {
        Self {
            time_of_day: time_of_day.rem_euclid(1.0),
        }
    }

    /// Advance the cycle by `dt` seconds. One full cycle = 1200 seconds (20 minutes).
    pub fn advance(&mut self, dt: f32) {
        self.time_of_day = (self.time_of_day + dt / CYCLE_DURATION).rem_euclid(1.0);
    }

    /// Sun direction as a unit vector.
    ///
    /// The sun orbits in the XY plane:
    /// - At sunrise (0.25) it is on the horizon at +X.
    /// - At noon (0.50) it is directly overhead.
    /// - At sunset (0.75) it is on the horizon at -X.
    /// - At midnight (0.00) it is directly below.
    pub fn sun_direction(&self) -> Vec3 {
        let angle = self.time_of_day * std::f32::consts::TAU;
        // sin(angle) maps: midnight(0)=0, sunrise(0.25)=1, noon(0.5)=0, sunset(0.75)=-1
        // For Y (up) we want: midnight=-1, sunrise=0, noon=1, sunset=0
        // That is -cos(angle).
        let x = angle.sin();
        let y = -angle.cos();
        Vec3::new(x, y, 0.3).normalize()
    }

    /// Interpolated sky color based on time of day.
    ///
    /// Returns an RGB triple in linear space.
    pub fn sky_color(&self) -> [f32; 3] {
        // Key colors
        const NIGHT: [f32; 3] = [0.02, 0.02, 0.08]; // dark blue
        const SUNRISE: [f32; 3] = [0.85, 0.45, 0.20]; // orange
        const DAY: [f32; 3] = [0.53, 0.81, 0.92]; // light blue
        const SUNSET: [f32; 3] = [0.85, 0.35, 0.15]; // orange-red

        let t = self.time_of_day;

        // Keyframe positions: midnight=0.0, sunrise=0.25, noon=0.5, sunset=0.75
        // Interpolate between adjacent keyframes.
        if t < 0.20 {
            // midnight -> pre-sunrise (night)
            NIGHT
        } else if t < 0.30 {
            // sunrise transition
            let f = (t - 0.20) / 0.10;
            lerp_color(NIGHT, SUNRISE, f)
        } else if t < 0.35 {
            // sunrise -> day
            let f = (t - 0.30) / 0.05;
            lerp_color(SUNRISE, DAY, f)
        } else if t < 0.65 {
            // daytime
            DAY
        } else if t < 0.70 {
            // day -> sunset
            let f = (t - 0.65) / 0.05;
            lerp_color(DAY, SUNSET, f)
        } else if t < 0.80 {
            // sunset -> night
            let f = (t - 0.70) / 0.10;
            lerp_color(SUNSET, NIGHT, f)
        } else {
            // night
            NIGHT
        }
    }

    /// Ambient light intensity: 0.15 at night, 0.5 at sunrise/sunset, 1.0 at noon.
    pub fn ambient_light(&self) -> f32 {
        let t = self.time_of_day;

        if t < 0.20 {
            // night
            0.15
        } else if t < 0.30 {
            // night -> sunrise
            let f = (t - 0.20) / 0.10;
            lerp_f32(0.15, 0.5, f)
        } else if t < 0.40 {
            // sunrise -> noon
            let f = (t - 0.30) / 0.10;
            lerp_f32(0.5, 1.0, f)
        } else if t < 0.60 {
            // around noon
            1.0
        } else if t < 0.70 {
            // noon -> sunset
            let f = (t - 0.60) / 0.10;
            lerp_f32(1.0, 0.5, f)
        } else if t < 0.80 {
            // sunset -> night
            let f = (t - 0.70) / 0.10;
            lerp_f32(0.5, 0.15, f)
        } else {
            // night
            0.15
        }
    }

    /// Build the GPU uniform from the current cycle state.
    pub fn uniform(&self) -> SkyUniform {
        let dir = self.sun_direction();
        SkyUniform {
            sky_color: self.sky_color(),
            _pad0: 0.0,
            sun_dir: [dir.x, dir.y, dir.z],
            ambient: self.ambient_light(),
        }
    }
}

/// Linearly interpolate between two colors.
fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Linearly interpolate between two floats.
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_starts_at_noon() {
        let cycle = DayNightCycle::default();
        assert!((cycle.time_of_day - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn advance_wraps_around() {
        let mut cycle = DayNightCycle::new(0.9);
        // Advance by 20% of a cycle (240 seconds)
        cycle.advance(240.0);
        assert!(cycle.time_of_day >= 0.0 && cycle.time_of_day < 1.0);
        assert!((cycle.time_of_day - 0.1).abs() < 1e-5);
    }

    #[test]
    fn advance_does_not_exceed_bounds() {
        let mut cycle = DayNightCycle::new(0.0);
        for _ in 0..10_000 {
            cycle.advance(1.0);
        }
        assert!(cycle.time_of_day >= 0.0 && cycle.time_of_day < 1.0);
    }

    #[test]
    fn sun_direction_is_normalized() {
        for i in 0..100 {
            let cycle = DayNightCycle::new(i as f32 / 100.0);
            let dir = cycle.sun_direction();
            assert!(
                (dir.length() - 1.0).abs() < 1e-5,
                "dir not normalized at t={}",
                cycle.time_of_day
            );
        }
    }

    #[test]
    fn noon_sun_is_roughly_overhead() {
        let cycle = DayNightCycle::new(0.5);
        let dir = cycle.sun_direction();
        // At noon, sun should be roughly in +Y direction
        assert!(dir.y > 0.9, "noon sun y={} should be > 0.9", dir.y);
    }

    #[test]
    fn midnight_sun_is_below_horizon() {
        let cycle = DayNightCycle::new(0.0);
        let dir = cycle.sun_direction();
        // At midnight, sun should be below
        assert!(dir.y < -0.9, "midnight sun y={} should be < -0.9", dir.y);
    }

    #[test]
    fn ambient_light_at_noon_is_full() {
        let cycle = DayNightCycle::new(0.5);
        assert!((cycle.ambient_light() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ambient_light_at_midnight_is_minimal() {
        let cycle = DayNightCycle::new(0.0);
        assert!((cycle.ambient_light() - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn sky_color_at_noon_is_blue() {
        let cycle = DayNightCycle::new(0.5);
        let color = cycle.sky_color();
        // Day blue: [0.53, 0.81, 0.92]
        assert!((color[0] - 0.53).abs() < 1e-5);
        assert!((color[1] - 0.81).abs() < 1e-5);
        assert!((color[2] - 0.92).abs() < 1e-5);
    }

    #[test]
    fn sky_color_at_midnight_is_dark() {
        let cycle = DayNightCycle::new(0.0);
        let color = cycle.sky_color();
        // Night: [0.02, 0.02, 0.08]
        assert!(color[0] < 0.1);
        assert!(color[1] < 0.1);
        assert!(color[2] < 0.15);
    }

    #[test]
    fn uniform_has_correct_layout() {
        let cycle = DayNightCycle::new(0.5);
        let u = cycle.uniform();
        assert_eq!(u.sky_color, cycle.sky_color());
        assert!((u.ambient - cycle.ambient_light()).abs() < f32::EPSILON);
        assert_eq!(u._pad0, 0.0);
    }

    #[test]
    fn sky_uniform_is_pod() {
        // Ensure SkyUniform is 32 bytes (2 x vec4<f32>)
        assert_eq!(std::mem::size_of::<SkyUniform>(), 32);
    }
}
