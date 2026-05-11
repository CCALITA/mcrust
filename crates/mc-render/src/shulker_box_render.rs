//! Shulker box color rendering, lid animation, and vertex generation.

/// Returns the RGB color for a dye index (0–15) or undyed (any other value → purple).
pub fn shulker_box_color(dye: u8) -> [f32; 3] {
    match dye {
        0 => [0.98, 0.98, 0.98],   // White
        1 => [0.95, 0.57, 0.19],   // Orange
        2 => [0.76, 0.38, 0.76],   // Magenta
        3 => [0.40, 0.60, 0.85],   // Light Blue
        4 => [0.98, 0.86, 0.24],   // Yellow
        5 => [0.49, 0.73, 0.18],   // Lime
        6 => [0.93, 0.55, 0.67],   // Pink
        7 => [0.30, 0.30, 0.30],   // Gray
        8 => [0.60, 0.60, 0.56],   // Light Gray
        9 => [0.15, 0.56, 0.60],   // Cyan
        10 => [0.50, 0.25, 0.70],  // Purple
        11 => [0.20, 0.25, 0.65],  // Blue
        12 => [0.45, 0.30, 0.17],  // Brown
        13 => [0.33, 0.42, 0.18],  // Green
        14 => [0.65, 0.20, 0.20],  // Red
        15 => [0.10, 0.10, 0.10],  // Black
        _ => [0.50, 0.25, 0.70],   // Undyed → purple
    }
}

/// Converts an open progress (0.0 = closed, 1.0 = fully open) to a lid angle in degrees (0–90).
pub fn shulker_lid_angle(open_progress: f32) -> f32 {
    open_progress.clamp(0.0, 1.0) * 90.0
}

/// Generates simplified box vertices for a shulker box at `pos` with the given `color` and
/// `lid_angle` (in degrees). Returns vertex positions for the base and lid.
pub fn shulker_box_vertices(pos: [f32; 3], color: [f32; 3], lid_angle: f32) -> Vec<[f32; 3]> {
    let [x, y, z] = pos;
    let _color = color; // Color would be used in a full implementation for vertex attributes
    let clamped_angle = lid_angle.clamp(0.0, 90.0);
    let angle_rad = clamped_angle.to_radians();

    let mut vertices = Vec::with_capacity(16);

    // Base box (bottom half, 0.0–0.5 in local Y)
    vertices.push([x, y, z]);
    vertices.push([x + 1.0, y, z]);
    vertices.push([x + 1.0, y, z + 1.0]);
    vertices.push([x, y, z + 1.0]);
    vertices.push([x, y + 0.5, z]);
    vertices.push([x + 1.0, y + 0.5, z]);
    vertices.push([x + 1.0, y + 0.5, z + 1.0]);
    vertices.push([x, y + 0.5, z + 1.0]);

    // Lid (top half, rotated around the back edge at y+0.5)
    let lid_height = 0.5;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // Front-bottom of lid (at y+0.5, rotated)
    let ly = y + 0.5 + lid_height * cos_a;
    let lz_offset = lid_height * sin_a;

    vertices.push([x, y + 0.5, z - lz_offset]);
    vertices.push([x + 1.0, y + 0.5, z - lz_offset]);
    vertices.push([x + 1.0, y + 0.5, z + 1.0 - lz_offset]);
    vertices.push([x, y + 0.5, z + 1.0 - lz_offset]);
    vertices.push([x, ly, z - lz_offset]);
    vertices.push([x + 1.0, ly, z - lz_offset]);
    vertices.push([x + 1.0, ly, z + 1.0 - lz_offset]);
    vertices.push([x, ly, z + 1.0 - lz_offset]);

    vertices
}

/// Returns the scale factor for shulker boxes when rendered as items.
pub fn shulker_box_item_scale() -> f32 {
    0.65
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_returns_white_for_dye_0() {
        let c = shulker_box_color(0);
        assert_eq!(c, [0.98, 0.98, 0.98]);
    }

    #[test]
    fn color_returns_black_for_dye_15() {
        let c = shulker_box_color(15);
        assert_eq!(c, [0.10, 0.10, 0.10]);
    }

    #[test]
    fn color_returns_purple_for_undyed() {
        let undyed = shulker_box_color(255);
        let purple = shulker_box_color(10);
        assert_eq!(undyed, purple);
    }

    #[test]
    fn all_16_dyes_are_distinct() {
        let colors: Vec<[f32; 3]> = (0..16).map(shulker_box_color).collect();
        for i in 0..16 {
            for j in (i + 1)..16 {
                assert_ne!(colors[i], colors[j], "dye {i} and {j} collide");
            }
        }
    }

    #[test]
    fn lid_angle_closed() {
        assert!((shulker_lid_angle(0.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lid_angle_fully_open() {
        assert!((shulker_lid_angle(1.0) - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lid_angle_half_open() {
        assert!((shulker_lid_angle(0.5) - 45.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lid_angle_clamps_below_zero() {
        assert!((shulker_lid_angle(-1.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lid_angle_clamps_above_one() {
        assert!((shulker_lid_angle(2.0) - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn vertices_returns_16_points() {
        let verts = shulker_box_vertices([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.0);
        assert_eq!(verts.len(), 16);
    }

    #[test]
    fn vertices_base_starts_at_pos() {
        let verts = shulker_box_vertices([5.0, 10.0, 3.0], [1.0, 0.0, 0.0], 0.0);
        assert_eq!(verts[0], [5.0, 10.0, 3.0]);
    }

    #[test]
    fn item_scale_is_0_65() {
        assert!((shulker_box_item_scale() - 0.65).abs() < f32::EPSILON);
    }
}
