/// World border system for constraining player movement within a square boundary.
///
/// The border is a square centered at (`center_x`, `center_z`) with the given
/// `radius` (half-width). It deals damage to entities outside and provides
/// warning indicators when players approach the edge.

/// World border configuration and state.
#[derive(Debug, Clone)]
pub struct WorldBorder {
    pub center_x: f64,
    pub center_z: f64,
    pub radius: f64,
    pub warning_distance: f64,
    pub warning_time_ticks: u32,
}

impl WorldBorder {
    /// Create a new world border with vanilla defaults.
    ///
    /// Center at (0, 0), radius = 29_999_984.0 (vanilla max),
    /// warning distance = 5.0 blocks, warning time = 15 ticks.
    pub fn new() -> Self {
        Self {
            center_x: 0.0,
            center_z: 0.0,
            radius: 29_999_984.0,
            warning_distance: 5.0,
            warning_time_ticks: 15,
        }
    }
}

impl Default for WorldBorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns `true` if the position (x, z) is outside the border.
///
/// The border is a square: `|x - cx| > radius || |z - cz| > radius`.
pub fn is_outside_border(x: f64, z: f64, border: &WorldBorder) -> bool {
    (x - border.center_x).abs() > border.radius
        || (z - border.center_z).abs() > border.radius
}

/// Distance from position (x, z) to the nearest edge of the square border.
///
/// Returns a positive value when inside (distance to the closest wall),
/// and a negative value when outside (how far past the wall).
pub fn distance_to_border(x: f64, z: f64, border: &WorldBorder) -> f64 {
    let dx = border.radius - (x - border.center_x).abs();
    let dz = border.radius - (z - border.center_z).abs();
    dx.min(dz)
}

/// Damage per tick for an entity that is `distance_outside` blocks beyond the border.
///
/// Formula: 0.2 HP per block, capped at 5.0 HP per tick.
/// Returns 0.0 if `distance_outside` is zero or negative.
pub fn damage_per_tick_outside(distance_outside: f64) -> f32 {
    if distance_outside <= 0.0 {
        return 0.0;
    }
    (0.2 * distance_outside as f32).min(5.0)
}

/// Compute the red warning overlay alpha for a player near the border.
///
/// Returns 0.0 if `distance_to_border` exceeds `warning_distance`,
/// linearly interpolates to 0.8 as the player reaches the border edge.
pub fn border_warning_overlay_alpha(dist_to_border: f64, warning_distance: f64) -> f32 {
    if warning_distance <= 0.0 || dist_to_border >= warning_distance {
        return 0.0;
    }
    if dist_to_border <= 0.0 {
        return 0.8;
    }
    let t = 1.0 - (dist_to_border / warning_distance);
    (t as f32 * 0.8).clamp(0.0, 0.8)
}

/// Returns `true` if position (x, z) is within the warning zone of the border.
///
/// The warning zone is the band of `warning_distance` blocks inside the border,
/// plus everything outside the border.
pub fn is_within_warning_zone(x: f64, z: f64, border: &WorldBorder) -> bool {
    let dist = distance_to_border(x, z, border);
    dist < border.warning_distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_vanilla_defaults() {
        let border = WorldBorder::new();
        assert!((border.center_x - 0.0).abs() < f64::EPSILON);
        assert!((border.center_z - 0.0).abs() < f64::EPSILON);
        assert!((border.radius - 29_999_984.0).abs() < f64::EPSILON);
        assert!((border.warning_distance - 5.0).abs() < f64::EPSILON);
        assert_eq!(border.warning_time_ticks, 15);
    }

    #[test]
    fn default_matches_new() {
        let a = WorldBorder::new();
        let b = WorldBorder::default();
        assert!((a.radius - b.radius).abs() < f64::EPSILON);
        assert_eq!(a.warning_time_ticks, b.warning_time_ticks);
    }

    #[test]
    fn is_outside_border_center_is_inside() {
        let border = WorldBorder::new();
        assert!(!is_outside_border(0.0, 0.0, &border));
    }

    #[test]
    fn is_outside_border_at_edge_is_inside() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        assert!(!is_outside_border(50.0, 0.0, &border));
        assert!(!is_outside_border(0.0, 50.0, &border));
        assert!(!is_outside_border(-50.0, -50.0, &border));
    }

    #[test]
    fn is_outside_border_beyond_edge() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        assert!(is_outside_border(51.0, 0.0, &border));
        assert!(is_outside_border(0.0, -51.0, &border));
        assert!(is_outside_border(51.0, 51.0, &border));
    }

    #[test]
    fn is_outside_border_with_offset_center() {
        let border = WorldBorder {
            center_x: 100.0,
            center_z: 200.0,
            radius: 10.0,
            warning_distance: 5.0,
            warning_time_ticks: 15,
        };
        assert!(!is_outside_border(100.0, 200.0, &border));
        assert!(!is_outside_border(110.0, 200.0, &border));
        assert!(is_outside_border(111.0, 200.0, &border));
        assert!(is_outside_border(100.0, 211.0, &border));
    }

    #[test]
    fn distance_to_border_at_center() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        let dist = distance_to_border(0.0, 0.0, &border);
        assert!((dist - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn distance_to_border_near_edge() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        let dist = distance_to_border(48.0, 0.0, &border);
        assert!((dist - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn distance_to_border_outside_is_negative() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        let dist = distance_to_border(53.0, 0.0, &border);
        assert!((dist - (-3.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn distance_to_border_negative_axis() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        let dist = distance_to_border(0.0, -47.0, &border);
        assert!((dist - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn damage_per_tick_outside_zero_inside() {
        assert!((damage_per_tick_outside(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((damage_per_tick_outside(-5.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_per_tick_outside_scales_linearly() {
        let d1 = damage_per_tick_outside(1.0);
        assert!((d1 - 0.2).abs() < f32::EPSILON);

        let d5 = damage_per_tick_outside(5.0);
        assert!((d5 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_per_tick_outside_capped_at_five() {
        let d = damage_per_tick_outside(100.0);
        assert!((d - 5.0).abs() < f32::EPSILON);

        let d_big = damage_per_tick_outside(1000.0);
        assert!((d_big - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_alpha_far_from_border() {
        let alpha = border_warning_overlay_alpha(10.0, 5.0);
        assert!((alpha - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_alpha_at_border_edge() {
        let alpha = border_warning_overlay_alpha(0.0, 5.0);
        assert!((alpha - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_alpha_at_warning_boundary() {
        let alpha = border_warning_overlay_alpha(5.0, 5.0);
        assert!((alpha - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_alpha_midway() {
        let alpha = border_warning_overlay_alpha(2.5, 5.0);
        // t = 1 - 2.5/5.0 = 0.5, alpha = 0.5 * 0.8 = 0.4
        assert!((alpha - 0.4).abs() < 0.001);
    }

    #[test]
    fn overlay_alpha_beyond_border() {
        // distance_to_border is negative (outside), alpha should be max
        let alpha = border_warning_overlay_alpha(-3.0, 5.0);
        assert!((alpha - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_alpha_zero_warning_distance() {
        let alpha = border_warning_overlay_alpha(1.0, 0.0);
        assert!((alpha - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn warning_zone_center_is_safe() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        assert!(!is_within_warning_zone(0.0, 0.0, &border));
    }

    #[test]
    fn warning_zone_near_edge() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        border.warning_distance = 5.0;
        // 3 blocks from edge => inside warning zone
        assert!(is_within_warning_zone(47.0, 0.0, &border));
        // 6 blocks from edge => outside warning zone
        assert!(!is_within_warning_zone(44.0, 0.0, &border));
    }

    #[test]
    fn warning_zone_outside_border() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        border.warning_distance = 5.0;
        // Outside the border => always in warning zone
        assert!(is_within_warning_zone(55.0, 0.0, &border));
    }

    #[test]
    fn warning_zone_exact_boundary() {
        let mut border = WorldBorder::new();
        border.radius = 50.0;
        border.warning_distance = 5.0;
        // At exactly warning_distance from edge => dist == 5.0 == warning_distance
        // dist < warning_distance is false, so NOT in warning zone
        assert!(!is_within_warning_zone(45.0, 0.0, &border));
    }
}
