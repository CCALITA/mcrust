//! Fog and distance rendering settings for different dimensions and conditions.

/// Shape of the fog volume.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FogShape {
    Sphere,
    Cylinder,
}

/// Configuration for distance-based fog rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct FogSettings {
    /// Distance at which fog begins (fully clear before this).
    pub start: f32,
    /// Distance at which fog is fully opaque.
    pub end: f32,
    /// RGB color of the fog.
    pub color: [f32; 3],
    /// Geometric shape of the fog volume.
    pub shape: FogShape,
}

/// Calculate fog intensity via linear interpolation.
///
/// Returns 0.0 if `distance <= settings.start`, 1.0 if `distance >= settings.end`,
/// and a linear blend in between.
pub fn calculate_fog(distance: f32, settings: &FogSettings) -> f32 {
    if distance <= settings.start {
        return 0.0;
    }
    if distance >= settings.end {
        return 1.0;
    }
    let range = settings.end - settings.start;
    if range <= 0.0 {
        return 1.0;
    }
    (distance - settings.start) / range
}

/// Default overworld fog based on render distance (in chunks).
///
/// Start = `(render_distance - 2) * 16`, end = `render_distance * 16`,
/// sky-blue color.
pub fn default_fog(render_distance: i32) -> FogSettings {
    FogSettings {
        start: (render_distance - 2) as f32 * 16.0,
        end: render_distance as f32 * 16.0,
        color: [0.53, 0.81, 0.92],
        shape: FogShape::Sphere,
    }
}

/// Fog settings for underwater visibility.
pub fn underwater_fog() -> FogSettings {
    FogSettings {
        start: 0.0,
        end: 48.0,
        color: [0.0, 0.05, 0.2],
        shape: FogShape::Sphere,
    }
}

/// Fog settings for the Nether dimension.
pub fn nether_fog() -> FogSettings {
    FogSettings {
        start: 0.0,
        end: 64.0,
        color: [0.2, 0.03, 0.03],
        shape: FogShape::Sphere,
    }
}

/// Fog settings for the End dimension.
pub fn end_fog() -> FogSettings {
    FogSettings {
        start: 0.0,
        end: 128.0,
        color: [0.06, 0.0, 0.06],
        shape: FogShape::Sphere,
    }
}

/// Select fog settings for a dimension.
///
/// Dimension codes: 0 = Overworld, 1 = Nether, 2 = End.
/// Unknown dimensions fall back to overworld defaults.
pub fn fog_for_dimension(dimension: u8, render_distance: i32) -> FogSettings {
    match dimension {
        0 => default_fog(render_distance),
        1 => nether_fog(),
        2 => end_fog(),
        _ => default_fog(render_distance),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_zero_at_start() {
        let settings = FogSettings {
            start: 10.0,
            end: 100.0,
            color: [1.0, 1.0, 1.0],
            shape: FogShape::Sphere,
        };
        assert_eq!(calculate_fog(10.0, &settings), 0.0);
        assert_eq!(calculate_fog(5.0, &settings), 0.0);
    }

    #[test]
    fn fog_one_at_end() {
        let settings = FogSettings {
            start: 10.0,
            end: 100.0,
            color: [1.0, 1.0, 1.0],
            shape: FogShape::Sphere,
        };
        assert_eq!(calculate_fog(100.0, &settings), 1.0);
        assert_eq!(calculate_fog(200.0, &settings), 1.0);
    }

    #[test]
    fn fog_linear_at_midpoint() {
        let settings = FogSettings {
            start: 0.0,
            end: 100.0,
            color: [1.0, 1.0, 1.0],
            shape: FogShape::Sphere,
        };
        let fog = calculate_fog(50.0, &settings);
        assert!((fog - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fog_linear_quarter() {
        let settings = FogSettings {
            start: 20.0,
            end: 60.0,
            color: [0.0, 0.0, 0.0],
            shape: FogShape::Cylinder,
        };
        let fog = calculate_fog(30.0, &settings);
        assert!((fog - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn default_fog_settings() {
        let fog = default_fog(8);
        assert_eq!(fog.start, 96.0); // (8 - 2) * 16
        assert_eq!(fog.end, 128.0); // 8 * 16
        assert_eq!(fog.color, [0.53, 0.81, 0.92]);
        assert_eq!(fog.shape, FogShape::Sphere);
    }

    #[test]
    fn underwater_fog_settings() {
        let fog = underwater_fog();
        assert_eq!(fog.start, 0.0);
        assert_eq!(fog.end, 48.0);
        assert_eq!(fog.color, [0.0, 0.05, 0.2]);
    }

    #[test]
    fn nether_fog_settings() {
        let fog = nether_fog();
        assert_eq!(fog.start, 0.0);
        assert_eq!(fog.end, 64.0);
        assert_eq!(fog.color, [0.2, 0.03, 0.03]);
    }

    #[test]
    fn end_fog_settings() {
        let fog = end_fog();
        assert_eq!(fog.start, 0.0);
        assert_eq!(fog.end, 128.0);
        assert_eq!(fog.color, [0.06, 0.0, 0.06]);
    }

    #[test]
    fn dimension_overworld() {
        let fog = fog_for_dimension(0, 12);
        assert_eq!(fog, default_fog(12));
    }

    #[test]
    fn dimension_nether() {
        let fog = fog_for_dimension(1, 12);
        assert_eq!(fog, nether_fog());
    }

    #[test]
    fn dimension_end() {
        let fog = fog_for_dimension(2, 12);
        assert_eq!(fog, end_fog());
    }

    #[test]
    fn dimension_unknown_falls_back_to_overworld() {
        let fog = fog_for_dimension(99, 10);
        assert_eq!(fog, default_fog(10));
    }

    #[test]
    fn fog_degenerate_range() {
        let settings = FogSettings {
            start: 50.0,
            end: 50.0,
            color: [0.0, 0.0, 0.0],
            shape: FogShape::Sphere,
        };
        // When start == end, anything at or beyond start is fully fogged
        assert_eq!(calculate_fog(50.0, &settings), 0.0);
        assert_eq!(calculate_fog(51.0, &settings), 1.0);
    }
}
