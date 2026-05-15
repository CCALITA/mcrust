//! Screen-space quad generation for fullscreen effects, UI elements, and overlays.

/// Returns 3 vertices for a fullscreen triangle that covers the entire NDC quad.
/// Uses the over-sized triangle trick: (-1,-1), (3,-1), (-1,3).
pub fn fullscreen_triangle_vertices() -> Vec<[f32; 2]> {
    vec![[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]
}

/// Converts a screen-space rectangle to 6 NDC vertices (2 triangles).
///
/// `x`, `y` are the top-left corner in pixels. `w`, `h` are dimensions in pixels.
/// `screen_w`, `screen_h` are the viewport dimensions.
pub fn screen_rect_vertices(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    screen_w: f32,
    screen_h: f32,
) -> Vec<[f32; 2]> {
    let to_ndc_x = |px: f32| px / screen_w * 2.0 - 1.0;
    let to_ndc_y = |py: f32| 1.0 - py / screen_h * 2.0;

    let left = to_ndc_x(x);
    let right = to_ndc_x(x + w);
    let top = to_ndc_y(y);
    let bottom = to_ndc_y(y + h);

    vec![
        // First triangle
        [left, top],
        [right, top],
        [left, bottom],
        // Second triangle
        [right, top],
        [right, bottom],
        [left, bottom],
    ]
}

/// Returns 4 line endpoints in NDC for a crosshair centered on screen.
///
/// The crosshair consists of a horizontal and vertical line segment,
/// each `size` pixels long, centered at the screen center.
pub fn crosshair_lines(screen_w: f32, screen_h: f32, size: f32) -> Vec<[f32; 2]> {
    let half_w = size / screen_w;
    let half_h = size / screen_h;

    vec![
        // Horizontal line
        [-half_w, 0.0],
        [half_w, 0.0],
        // Vertical line
        [0.0, -half_h],
        [0.0, half_h],
    ]
}

/// Returns a bottom-left positioned health bar rectangle as (x, y, width, height) in NDC.
///
/// The bar scales horizontally based on `health / max_health`.
/// `screen_w` is used for consistent bar sizing relative to the viewport.
pub fn health_bar_rect(health: f32, max_health: f32, screen_w: f32) -> (f32, f32, f32, f32) {
    let bar_width_px = 182.0; // Minecraft default HUD bar width
    let bar_height_px = 10.0;

    let ratio = (health / max_health).clamp(0.0, 1.0);
    let full_width_ndc = bar_width_px / screen_w * 2.0;
    let height_ndc = bar_height_px / screen_w * 2.0; // Use screen_w for aspect-correct sizing

    let width = full_width_ndc * ratio;
    let x = -1.0 + 0.02; // Small margin from left edge
    let y = -1.0 + 0.02; // Small margin from bottom edge

    (x, y, width, height_ndc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullscreen_triangle_has_three_vertices() {
        let verts = fullscreen_triangle_vertices();
        assert_eq!(verts.len(), 3);
        assert_eq!(verts[0], [-1.0, -1.0]);
        assert_eq!(verts[1], [3.0, -1.0]);
        assert_eq!(verts[2], [-1.0, 3.0]);
    }

    #[test]
    fn screen_rect_produces_six_vertices() {
        let verts = screen_rect_vertices(0.0, 0.0, 800.0, 600.0, 800.0, 600.0);
        assert_eq!(verts.len(), 6);
        // Full screen rect should span -1..1 in both axes
        assert!((verts[0][0] - (-1.0)).abs() < 1e-5);
        assert!((verts[0][1] - 1.0).abs() < 1e-5);
        assert!((verts[4][0] - 1.0).abs() < 1e-5);
        assert!((verts[4][1] - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn screen_rect_partial_coverage() {
        let verts = screen_rect_vertices(200.0, 150.0, 400.0, 300.0, 800.0, 600.0);
        assert_eq!(verts.len(), 6);
        // x=200 on 800 wide: 200/800*2 - 1 = -0.5
        assert!((verts[0][0] - (-0.5)).abs() < 1e-5);
        // y=150 on 600 tall: 1 - 150/600*2 = 0.5
        assert!((verts[0][1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn crosshair_lines_are_centered() {
        let lines = crosshair_lines(800.0, 600.0, 20.0);
        assert_eq!(lines.len(), 4);
        // Horizontal line centered at y=0
        assert!((lines[0][1]).abs() < 1e-5);
        assert!((lines[1][1]).abs() < 1e-5);
        // Vertical line centered at x=0
        assert!((lines[2][0]).abs() < 1e-5);
        assert!((lines[3][0]).abs() < 1e-5);
        // Symmetric
        assert!((lines[0][0] + lines[1][0]).abs() < 1e-5);
        assert!((lines[2][1] + lines[3][1]).abs() < 1e-5);
    }

    #[test]
    fn health_bar_scales_with_health() {
        let (_, _, full_w, _) = health_bar_rect(20.0, 20.0, 800.0);
        let (_, _, half_w, _) = health_bar_rect(10.0, 20.0, 800.0);
        assert!((half_w - full_w / 2.0).abs() < 1e-5);
    }

    #[test]
    fn health_bar_clamps_to_max() {
        let (_, _, over_w, _) = health_bar_rect(30.0, 20.0, 800.0);
        let (_, _, full_w, _) = health_bar_rect(20.0, 20.0, 800.0);
        assert!((over_w - full_w).abs() < 1e-5);
    }

    #[test]
    fn health_bar_zero_health() {
        let (_, _, w, _) = health_bar_rect(0.0, 20.0, 800.0);
        assert!(w.abs() < 1e-5);
    }

    #[test]
    fn health_bar_positioned_bottom_left() {
        let (x, y, _, _) = health_bar_rect(20.0, 20.0, 800.0);
        assert!(x > -1.0);
        assert!(x < 0.0);
        assert!(y > -1.0);
        assert!(y < 0.0);
    }
}
