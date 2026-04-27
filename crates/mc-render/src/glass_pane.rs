/// Glass pane rendering: stained glass colors, thin center post geometry,
/// directional arm vertices, and collision boxes for connected pane blocks.

/// Glass pane state tracking color and NSEW connections to neighbors.
pub struct GlassPaneState {
    /// Dye color index (0..15), matching Minecraft dye order.
    pub color: u8,
    /// Connections to adjacent blocks: [North, South, East, West].
    pub connections: [bool; 4],
}

/// Width of the glass pane center post in block units (2/16).
const PANE_THICKNESS: f32 = 2.0 / 16.0;

/// Half the pane thickness for offset calculations.
const HALF_THICKNESS: f32 = PANE_THICKNESS / 2.0;

/// Maps a dye color index (0..15) to an RGBA color with 0.5 alpha.
///
/// Color indices follow the Minecraft dye order:
/// 0=white, 1=orange, 2=magenta, 3=light_blue, 4=yellow, 5=lime,
/// 6=pink, 7=gray, 8=light_gray, 9=cyan, 10=purple, 11=blue,
/// 12=brown, 13=green, 14=red, 15=black.
pub fn stained_glass_color(dye: u8) -> [f32; 4] {
    let rgb = match dye {
        0 => [1.0, 1.0, 1.0],           // white
        1 => [0.85, 0.52, 0.2],         // orange
        2 => [0.7, 0.32, 0.85],         // magenta
        3 => [0.38, 0.6, 0.85],         // light blue
        4 => [0.95, 0.9, 0.28],         // yellow
        5 => [0.49, 0.83, 0.15],        // lime
        6 => [0.95, 0.55, 0.66],        // pink
        7 => [0.37, 0.37, 0.37],        // gray
        8 => [0.6, 0.6, 0.6],           // light gray
        9 => [0.15, 0.56, 0.6],         // cyan
        10 => [0.5, 0.25, 0.7],         // purple
        11 => [0.2, 0.25, 0.7],         // blue
        12 => [0.45, 0.3, 0.17],        // brown
        13 => [0.33, 0.42, 0.18],       // green
        14 => [0.7, 0.2, 0.2],          // red
        15 => [0.1, 0.1, 0.1],          // black
        _ => [1.0, 1.0, 1.0],           // fallback to white
    };
    [rgb[0], rgb[1], rgb[2], 0.5]
}

/// Generates 8 vertices for the thin center post of a glass pane.
///
/// The post is a 2/16 x 1.0 x 2/16 column centered on the block position.
/// Returns vertices as `[x, y, z]` for the 8 corners of the box.
pub fn glass_pane_center_vertices(pos: [f32; 3]) -> Vec<[f32; 3]> {
    let cx = pos[0] + 0.5;
    let cy = pos[1];
    let cz = pos[2] + 0.5;

    vec![
        // Bottom face (y = cy)
        [cx - HALF_THICKNESS, cy, cz - HALF_THICKNESS],
        [cx + HALF_THICKNESS, cy, cz - HALF_THICKNESS],
        [cx + HALF_THICKNESS, cy, cz + HALF_THICKNESS],
        [cx - HALF_THICKNESS, cy, cz + HALF_THICKNESS],
        // Top face (y = cy + 1.0)
        [cx - HALF_THICKNESS, cy + 1.0, cz - HALF_THICKNESS],
        [cx + HALF_THICKNESS, cy + 1.0, cz - HALF_THICKNESS],
        [cx + HALF_THICKNESS, cy + 1.0, cz + HALF_THICKNESS],
        [cx - HALF_THICKNESS, cy + 1.0, cz + HALF_THICKNESS],
    ]
}

/// Generates 8 vertices for an arm extending from the center post to a block edge.
///
/// Direction: 0=North (-Z), 1=South (+Z), 2=East (+X), 3=West (-X).
/// Each arm is a thin box from the center post edge to the block boundary.
pub fn glass_pane_arm_vertices(pos: [f32; 3], direction: u8) -> Vec<[f32; 3]> {
    let cx = pos[0] + 0.5;
    let cy = pos[1];
    let cz = pos[2] + 0.5;

    let (x_min, x_max, z_min, z_max) = match direction {
        0 => {
            // North: center to -Z edge, thin in X
            (cx - HALF_THICKNESS, cx + HALF_THICKNESS, pos[2], cz - HALF_THICKNESS)
        }
        1 => {
            // South: center to +Z edge, thin in X
            (cx - HALF_THICKNESS, cx + HALF_THICKNESS, cz + HALF_THICKNESS, pos[2] + 1.0)
        }
        2 => {
            // East: center to +X edge, thin in Z
            (cx + HALF_THICKNESS, pos[0] + 1.0, cz - HALF_THICKNESS, cz + HALF_THICKNESS)
        }
        3 => {
            // West: center to -X edge, thin in Z
            (pos[0], cx - HALF_THICKNESS, cz - HALF_THICKNESS, cz + HALF_THICKNESS)
        }
        _ => return Vec::new(),
    };

    vec![
        // Bottom face
        [x_min, cy, z_min],
        [x_max, cy, z_min],
        [x_max, cy, z_max],
        [x_min, cy, z_max],
        // Top face
        [x_min, cy + 1.0, z_min],
        [x_max, cy + 1.0, z_min],
        [x_max, cy + 1.0, z_max],
        [x_min, cy + 1.0, z_max],
    ]
}

/// Returns AABB collision boxes for a glass pane as `[x_min, y_min, z_min, x_max, y_max, z_max]`.
///
/// Always includes the center post. Adds an arm box for each active connection
/// in `connections` (North, South, East, West).
pub fn glass_pane_collision_box(connections: [bool; 4]) -> Vec<[f32; 6]> {
    let c = 0.5;
    let mut boxes = Vec::with_capacity(5);

    // Center post
    boxes.push([
        c - HALF_THICKNESS,
        0.0,
        c - HALF_THICKNESS,
        c + HALF_THICKNESS,
        1.0,
        c + HALF_THICKNESS,
    ]);

    // North arm (-Z)
    if connections[0] {
        boxes.push([c - HALF_THICKNESS, 0.0, 0.0, c + HALF_THICKNESS, 1.0, c - HALF_THICKNESS]);
    }
    // South arm (+Z)
    if connections[1] {
        boxes.push([c - HALF_THICKNESS, 0.0, c + HALF_THICKNESS, c + HALF_THICKNESS, 1.0, 1.0]);
    }
    // East arm (+X)
    if connections[2] {
        boxes.push([c + HALF_THICKNESS, 0.0, c - HALF_THICKNESS, 1.0, 1.0, c + HALF_THICKNESS]);
    }
    // West arm (-X)
    if connections[3] {
        boxes.push([0.0, 0.0, c - HALF_THICKNESS, c - HALF_THICKNESS, 1.0, c + HALF_THICKNESS]);
    }

    boxes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_16_stained_glass_colors_valid_with_half_alpha() {
        for i in 0..16u8 {
            let color = stained_glass_color(i);
            for c in &color[..3] {
                assert!(*c >= 0.0 && *c <= 1.0, "dye {i} RGB out of range: {c}");
            }
            assert!(
                (color[3] - 0.5).abs() < 1e-6,
                "dye {i} alpha should be 0.5, got {}",
                color[3]
            );
        }
    }

    #[test]
    fn invalid_dye_returns_white_with_half_alpha() {
        let color = stained_glass_color(16);
        assert_eq!(color, [1.0, 1.0, 1.0, 0.5]);
        let color = stained_glass_color(255);
        assert_eq!(color, [1.0, 1.0, 1.0, 0.5]);
    }

    #[test]
    fn specific_stained_glass_colors() {
        assert_eq!(stained_glass_color(0), [1.0, 1.0, 1.0, 0.5]); // white
        assert_eq!(stained_glass_color(11), [0.2, 0.25, 0.7, 0.5]); // blue
        assert_eq!(stained_glass_color(15), [0.1, 0.1, 0.1, 0.5]); // black
    }

    #[test]
    fn center_vertices_produces_8_points() {
        let verts = glass_pane_center_vertices([0.0, 64.0, 0.0]);
        assert_eq!(verts.len(), 8);
    }

    #[test]
    fn center_vertices_span_full_block_height() {
        let verts = glass_pane_center_vertices([0.0, 10.0, 0.0]);
        let min_y = verts.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
        let max_y = verts.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!((min_y - 10.0).abs() < 1e-6);
        assert!((max_y - 11.0).abs() < 1e-6);
    }

    #[test]
    fn center_vertices_are_thin() {
        let verts = glass_pane_center_vertices([0.0, 0.0, 0.0]);
        let min_x = verts.iter().map(|v| v[0]).fold(f32::INFINITY, f32::min);
        let max_x = verts.iter().map(|v| v[0]).fold(f32::NEG_INFINITY, f32::max);
        let width = max_x - min_x;
        assert!((width - PANE_THICKNESS).abs() < 1e-6, "center width should be 2/16, got {width}");
    }

    #[test]
    fn arm_vertices_produces_8_points_per_direction() {
        for dir in 0..4u8 {
            let verts = glass_pane_arm_vertices([0.0, 0.0, 0.0], dir);
            assert_eq!(verts.len(), 8, "direction {dir} should produce 8 vertices");
        }
    }

    #[test]
    fn arm_invalid_direction_returns_empty() {
        let verts = glass_pane_arm_vertices([0.0, 0.0, 0.0], 4);
        assert!(verts.is_empty());
        let verts = glass_pane_arm_vertices([0.0, 0.0, 0.0], 255);
        assert!(verts.is_empty());
    }

    #[test]
    fn north_arm_extends_to_negative_z_edge() {
        let verts = glass_pane_arm_vertices([0.0, 0.0, 0.0], 0);
        let min_z = verts.iter().map(|v| v[2]).fold(f32::INFINITY, f32::min);
        assert!((min_z - 0.0).abs() < 1e-6, "north arm should reach z=0.0");
    }

    #[test]
    fn south_arm_extends_to_positive_z_edge() {
        let verts = glass_pane_arm_vertices([0.0, 0.0, 0.0], 1);
        let max_z = verts.iter().map(|v| v[2]).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_z - 1.0).abs() < 1e-6, "south arm should reach z=1.0");
    }

    #[test]
    fn east_arm_extends_to_positive_x_edge() {
        let verts = glass_pane_arm_vertices([0.0, 0.0, 0.0], 2);
        let max_x = verts.iter().map(|v| v[0]).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_x - 1.0).abs() < 1e-6, "east arm should reach x=1.0");
    }

    #[test]
    fn west_arm_extends_to_negative_x_edge() {
        let verts = glass_pane_arm_vertices([0.0, 0.0, 0.0], 3);
        let min_x = verts.iter().map(|v| v[0]).fold(f32::INFINITY, f32::min);
        assert!((min_x - 0.0).abs() < 1e-6, "west arm should reach x=0.0");
    }

    #[test]
    fn collision_box_center_only_when_no_connections() {
        let boxes = glass_pane_collision_box([false, false, false, false]);
        assert_eq!(boxes.len(), 1, "no connections should yield center box only");
    }

    #[test]
    fn collision_box_all_connections_yields_5_boxes() {
        let boxes = glass_pane_collision_box([true, true, true, true]);
        assert_eq!(boxes.len(), 5, "all connections should yield 5 boxes");
    }

    #[test]
    fn collision_box_partial_connections() {
        let boxes = glass_pane_collision_box([true, false, true, false]);
        assert_eq!(boxes.len(), 3, "N+E connections should yield 3 boxes");
    }

    #[test]
    fn collision_boxes_have_valid_aabb() {
        let boxes = glass_pane_collision_box([true, true, true, true]);
        for (i, b) in boxes.iter().enumerate() {
            assert!(b[0] <= b[3], "box {i}: x_min <= x_max");
            assert!(b[1] <= b[4], "box {i}: y_min <= y_max");
            assert!(b[2] <= b[5], "box {i}: z_min <= z_max");
        }
    }

    #[test]
    fn collision_center_box_is_thin() {
        let boxes = glass_pane_collision_box([false, false, false, false]);
        let center = &boxes[0];
        let x_width = center[3] - center[0];
        let z_width = center[5] - center[2];
        assert!((x_width - PANE_THICKNESS).abs() < 1e-6);
        assert!((z_width - PANE_THICKNESS).abs() < 1e-6);
        assert!((center[4] - center[1] - 1.0).abs() < 1e-6, "center should be 1 block tall");
    }

    #[test]
    fn glass_pane_state_stores_color_and_connections() {
        let state = GlassPaneState {
            color: 3,
            connections: [true, false, true, false],
        };
        assert_eq!(state.color, 3);
        assert_eq!(state.connections, [true, false, true, false]);
    }
}
