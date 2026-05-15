//! Sky gradient computation: color, sun/moon positions, and star visibility.

use std::f32::consts::TAU;

/// Key points for the sky color cycle.
/// time 0.0 = dawn, 0.25 = noon, 0.5 = dusk, 0.75 = midnight.
const SKY_KEY_POINTS: [(f32, [f32; 3]); 4] = [
    (0.0, [1.0, 0.5, 0.2]),      // dawn
    (0.25, [0.5, 0.7, 1.0]),     // noon
    (0.5, [1.0, 0.3, 0.1]),      // dusk
    (0.75, [0.05, 0.05, 0.15]),  // midnight
];

/// Horizon key points — warmer/more saturated near the horizon.
const HORIZON_KEY_POINTS: [(f32, [f32; 3]); 4] = [
    (0.0, [1.0, 0.6, 0.3]),      // dawn horizon
    (0.25, [0.6, 0.8, 1.0]),     // noon horizon
    (0.5, [1.0, 0.4, 0.15]),     // dusk horizon
    (0.75, [0.03, 0.03, 0.1]),   // midnight horizon
];

/// Smoothly interpolate between cyclic key points.
fn interpolate_cyclic(time: f32, key_points: &[(f32, [f32; 3])]) -> [f32; 3] {
    let t = time.rem_euclid(1.0);
    let n = key_points.len();

    // Find the two surrounding key points.
    let mut idx = 0;
    for i in 0..n {
        if key_points[(i + 1) % n].0.rem_euclid(1.0) > t || i == n - 1 {
            idx = i;
            break;
        }
        if key_points[i].0 <= t && t < key_points[(i + 1) % n].0 {
            idx = i;
            break;
        }
    }

    let (t0, c0) = key_points[idx];
    let (t1, c1) = key_points[(idx + 1) % n];

    let segment_len = if t1 > t0 { t1 - t0 } else { 1.0 - t0 + t1 };
    let offset = if t >= t0 { t - t0 } else { 1.0 - t0 + t };
    let frac = if segment_len > 0.0 { offset / segment_len } else { 0.0 };

    // Smoothstep for smooth transitions.
    let s = frac * frac * (3.0 - 2.0 * frac);

    [
        c0[0] + (c1[0] - c0[0]) * s,
        c0[1] + (c1[1] - c0[1]) * s,
        c0[2] + (c1[2] - c0[2]) * s,
    ]
}

/// Returns the sky color at a given time of day.
///
/// `time` ranges from 0.0 to 1.0 (cyclic):
/// - 0.0 = dawn (orange-red)
/// - 0.25 = noon (blue)
/// - 0.5 = dusk (deep orange)
/// - 0.75 = midnight (dark blue)
pub fn sky_color_at_time(time: f32) -> [f32; 3] {
    interpolate_cyclic(time, &SKY_KEY_POINTS)
}

/// Returns the sun position at a given time of day.
///
/// The sun traces a circle in the XY plane.
/// At time 0.0 (dawn) the sun is on the horizon (east).
pub fn sun_position_at_time(time: f32) -> [f32; 3] {
    let angle = time * TAU;
    [angle.cos(), angle.sin(), 0.0]
}

/// Returns the moon position at a given time of day.
///
/// The moon is always opposite the sun.
pub fn moon_position_at_time(time: f32) -> [f32; 3] {
    let sun = sun_position_at_time(time);
    [-sun[0], -sun[1], -sun[2]]
}

/// Returns star visibility at a given time of day.
///
/// Returns 0.0 during the day and 1.0 at night, with smooth transitions
/// around dawn and dusk.
pub fn star_visibility(time: f32) -> f32 {
    let t = time.rem_euclid(1.0);
    // Night is centered around 0.75 (midnight).
    // Stars fully visible from ~0.6 to ~0.9, fading at dawn (0.0) and dusk (0.5).
    // Use distance from noon (0.25) mapped through smoothstep.
    let dist_from_noon = {
        let d = (t - 0.25).abs();
        if d > 0.5 { 1.0 - d } else { d }
    };
    // dist_from_noon: 0.0 at noon, 0.5 at midnight
    // Map to visibility: 0 when dist < 0.2, 1 when dist > 0.4
    let normalized = ((dist_from_noon - 0.2) / 0.2).clamp(0.0, 1.0);
    normalized * normalized * (3.0 - 2.0 * normalized)
}

/// Returns the horizon color at a given time of day.
///
/// The horizon is typically warmer and more saturated than the zenith sky color.
pub fn horizon_color(time: f32) -> [f32; 3] {
    interpolate_cyclic(time, &HORIZON_KEY_POINTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: [f32; 3], b: [f32; 3], epsilon: f32) -> bool {
        (a[0] - b[0]).abs() < epsilon
            && (a[1] - b[1]).abs() < epsilon
            && (a[2] - b[2]).abs() < epsilon
    }

    #[test]
    fn sky_color_at_dawn() {
        let c = sky_color_at_time(0.0);
        assert!(approx_eq(c, [1.0, 0.5, 0.2], 0.01));
    }

    #[test]
    fn sky_color_at_noon() {
        let c = sky_color_at_time(0.25);
        assert!(approx_eq(c, [0.5, 0.7, 1.0], 0.01));
    }

    #[test]
    fn sky_color_at_dusk() {
        let c = sky_color_at_time(0.5);
        assert!(approx_eq(c, [1.0, 0.3, 0.1], 0.01));
    }

    #[test]
    fn sky_color_at_midnight() {
        let c = sky_color_at_time(0.75);
        assert!(approx_eq(c, [0.05, 0.05, 0.15], 0.01));
    }

    #[test]
    fn sky_color_wraps_around() {
        let c0 = sky_color_at_time(0.0);
        let c1 = sky_color_at_time(1.0);
        assert!(approx_eq(c0, c1, 0.01));
    }

    #[test]
    fn sky_color_interpolates_smoothly() {
        let c = sky_color_at_time(0.125);
        // Should be between dawn and noon
        assert!(c[0] > 0.4 && c[0] < 1.1);
        assert!(c[1] > 0.4 && c[1] < 0.8);
    }

    #[test]
    fn sun_position_at_dawn() {
        let p = sun_position_at_time(0.0);
        assert!((p[0] - 1.0).abs() < 0.01);
        assert!(p[1].abs() < 0.01);
        assert!(p[2].abs() < 0.01);
    }

    #[test]
    fn sun_position_quarter_cycle() {
        let p = sun_position_at_time(0.25);
        assert!(p[0].abs() < 0.01);
        assert!((p[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn moon_opposite_sun() {
        for t in [0.0, 0.1, 0.25, 0.5, 0.75] {
            let sun = sun_position_at_time(t);
            let moon = moon_position_at_time(t);
            assert!((sun[0] + moon[0]).abs() < 0.01);
            assert!((sun[1] + moon[1]).abs() < 0.01);
            assert!((sun[2] + moon[2]).abs() < 0.01);
        }
    }

    #[test]
    fn stars_invisible_at_noon() {
        let v = star_visibility(0.25);
        assert!(v < 0.01, "stars should be invisible at noon, got {v}");
    }

    #[test]
    fn stars_visible_at_midnight() {
        let v = star_visibility(0.75);
        assert!(v > 0.99, "stars should be fully visible at midnight, got {v}");
    }

    #[test]
    fn star_visibility_bounded() {
        for i in 0..100 {
            let t = i as f32 / 100.0;
            let v = star_visibility(t);
            assert!((0.0..=1.0).contains(&v), "visibility out of range at t={t}: {v}");
        }
    }

    #[test]
    fn horizon_color_at_dawn() {
        let c = horizon_color(0.0);
        assert!(approx_eq(c, [1.0, 0.6, 0.3], 0.01));
    }

    #[test]
    fn horizon_color_at_noon() {
        let c = horizon_color(0.25);
        assert!(approx_eq(c, [0.6, 0.8, 1.0], 0.01));
    }

    #[test]
    fn horizon_warmer_than_sky_at_dawn() {
        let sky = sky_color_at_time(0.0);
        let hor = horizon_color(0.0);
        // Horizon should have more green/blue warmth
        assert!(hor[1] > sky[1], "horizon should be warmer at dawn");
    }
}
