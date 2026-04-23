//! Crosshair overlay rendering for the HUD.
//!
//! Generates a simple + shaped crosshair centered on screen,
//! in both pixel-space and normalized device coordinates.

use bytemuck::{Pod, Zeroable};

/// 2D screen-space vertex for crosshair rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CrosshairVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

/// Returns the crosshair half-extent in pixels.
pub fn crosshair_size() -> f32 {
    20.0
}

/// Returns the default crosshair color (white at 70% opacity).
pub fn crosshair_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 0.7]
}

/// Generates vertices for a + shaped crosshair centered on screen.
///
/// Returns 4 vertices:
/// - Vertices 0-1: horizontal line from `(cx - size, cy)` to `(cx + size, cy)`
/// - Vertices 2-3: vertical line from `(cx, cy - size)` to `(cx, cy + size)`
pub fn generate_crosshair(screen_width: f32, screen_height: f32) -> Vec<CrosshairVertex> {
    let cx = screen_width / 2.0;
    let cy = screen_height / 2.0;
    let size = crosshair_size();
    let color = crosshair_color();

    vec![
        // Horizontal line
        CrosshairVertex {
            position: [cx - size, cy],
            color,
        },
        CrosshairVertex {
            position: [cx + size, cy],
            color,
        },
        // Vertical line
        CrosshairVertex {
            position: [cx, cy - size],
            color,
        },
        CrosshairVertex {
            position: [cx, cy + size],
            color,
        },
    ]
}

/// Generates crosshair line endpoints in normalized device coordinates (-1..1).
///
/// Returns 4 `[f32; 2]` points representing the same + shape as
/// [`generate_crosshair`], but mapped to NDC where the screen spans
/// `(-1, -1)` at the bottom-left to `(1, 1)` at the top-right.
pub fn crosshair_ndc(screen_width: f32, screen_height: f32) -> Vec<[f32; 2]> {
    let size = crosshair_size();
    let ndc_half_w = size / (screen_width / 2.0);
    let ndc_half_h = size / (screen_height / 2.0);

    vec![
        // Horizontal line (centered at origin in NDC)
        [-ndc_half_w, 0.0],
        [ndc_half_w, 0.0],
        // Vertical line
        [0.0, -ndc_half_h],
        [0.0, ndc_half_h],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_crosshair_returns_four_vertices() {
        let verts = generate_crosshair(800.0, 600.0);
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn generate_crosshair_center_position() {
        let verts = generate_crosshair(800.0, 600.0);
        let cx = 400.0_f32;
        let cy = 300.0_f32;
        let size = crosshair_size();

        // Horizontal line endpoints
        assert!((verts[0].position[0] - (cx - size)).abs() < f32::EPSILON);
        assert!((verts[0].position[1] - cy).abs() < f32::EPSILON);
        assert!((verts[1].position[0] - (cx + size)).abs() < f32::EPSILON);
        assert!((verts[1].position[1] - cy).abs() < f32::EPSILON);

        // Vertical line endpoints
        assert!((verts[2].position[0] - cx).abs() < f32::EPSILON);
        assert!((verts[2].position[1] - (cy - size)).abs() < f32::EPSILON);
        assert!((verts[3].position[0] - cx).abs() < f32::EPSILON);
        assert!((verts[3].position[1] - (cy + size)).abs() < f32::EPSILON);
    }

    #[test]
    fn generate_crosshair_uses_correct_color() {
        let verts = generate_crosshair(1920.0, 1080.0);
        let expected = crosshair_color();
        for v in &verts {
            assert_eq!(v.color, expected);
        }
    }

    #[test]
    fn crosshair_ndc_returns_four_points() {
        let pts = crosshair_ndc(800.0, 600.0);
        assert_eq!(pts.len(), 4);
    }

    #[test]
    fn crosshair_ndc_values_in_range() {
        let pts = crosshair_ndc(800.0, 600.0);
        for pt in &pts {
            assert!(pt[0] >= -1.0 && pt[0] <= 1.0, "x={} out of NDC range", pt[0]);
            assert!(pt[1] >= -1.0 && pt[1] <= 1.0, "y={} out of NDC range", pt[1]);
        }
    }

    #[test]
    fn crosshair_ndc_centered_at_origin() {
        let pts = crosshair_ndc(1024.0, 768.0);
        // Horizontal line: y should be 0
        assert!((pts[0][1]).abs() < f32::EPSILON);
        assert!((pts[1][1]).abs() < f32::EPSILON);
        // Vertical line: x should be 0
        assert!((pts[2][0]).abs() < f32::EPSILON);
        assert!((pts[3][0]).abs() < f32::EPSILON);
    }

    #[test]
    fn crosshair_ndc_symmetric() {
        let pts = crosshair_ndc(800.0, 600.0);
        // Horizontal: left and right should be symmetric
        assert!((pts[0][0] + pts[1][0]).abs() < f32::EPSILON);
        // Vertical: top and bottom should be symmetric
        assert!((pts[2][1] + pts[3][1]).abs() < f32::EPSILON);
    }

    #[test]
    fn crosshair_ndc_different_screen_sizes() {
        let pts_small = crosshair_ndc(640.0, 480.0);
        let pts_large = crosshair_ndc(3840.0, 2160.0);

        // Larger screen => smaller NDC extent (same pixel size, more pixels)
        assert!(pts_small[1][0].abs() > pts_large[1][0].abs());
        assert!(pts_small[3][1].abs() > pts_large[3][1].abs());

        // All still within NDC range
        for pt in pts_small.iter().chain(pts_large.iter()) {
            assert!(pt[0] >= -1.0 && pt[0] <= 1.0);
            assert!(pt[1] >= -1.0 && pt[1] <= 1.0);
        }
    }

    #[test]
    fn crosshair_size_returns_expected() {
        assert!((crosshair_size() - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn crosshair_color_returns_expected() {
        let c = crosshair_color();
        assert!((c[0] - 1.0).abs() < f32::EPSILON);
        assert!((c[1] - 1.0).abs() < f32::EPSILON);
        assert!((c[2] - 1.0).abs() < f32::EPSILON);
        assert!((c[3] - 0.7).abs() < f32::EPSILON);
    }
}
