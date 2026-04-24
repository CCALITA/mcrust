//! Rendering data for dropped items in the world.
//!
//! Dropped items either render as a flat camera-facing billboard quad
//! (for non-block items) or as a small cube (for block items). Both
//! types bob up and down and rotate slowly while sitting on the ground.

/// Rotation speed in radians per second.
const ROTATION_SPEED: f32 = 0.5;

/// Vertical bob amplitude in blocks.
const BOB_AMPLITUDE: f32 = 0.05;

/// Bob frequency multiplier.
const BOB_FREQUENCY: f32 = 2.0;

/// Scale of the flat billboard quad (in blocks).
const FLAT_QUAD_SCALE: f32 = 0.5;

/// Scale of the block-item cube (in blocks).
const BLOCK_CUBE_SCALE: f32 = 0.25;

/// Item IDs below this value are treated as blocks.
const BLOCK_ID_THRESHOLD: u16 = 1000;

/// Per-frame render data for a single dropped item entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DroppedItemRender {
    /// Item ID (block id if `< 1000`, otherwise a generic item).
    pub item_id: u16,
    /// World position (x, y, z) of the item drop's pivot.
    pub position: [f32; 3],
    /// Current Y-axis rotation in radians.
    pub rotation: f32,
    /// Current vertical bob offset in blocks.
    pub bob_offset: f32,
    /// Optional stack count to display next to the item ("x12" etc.).
    pub count_label: u8,
}

impl DroppedItemRender {
    /// Create a new dropped-item render entry with default rotation/bob.
    pub fn new(item_id: u16, pos: [f32; 3]) -> Self {
        Self {
            item_id,
            position: pos,
            rotation: 0.0,
            bob_offset: 0.0,
            count_label: 1,
        }
    }
}

/// Advance the rotation and bob animation of a dropped item.
///
/// `dt` is the frame delta in seconds; `time` is the total elapsed time
/// (used to keep bobbing in phase across all dropped items).
pub fn tick_dropped_item(item: &mut DroppedItemRender, dt: f32, time: f32) {
    item.rotation = (item.rotation + ROTATION_SPEED * dt).rem_euclid(std::f32::consts::TAU);
    item.bob_offset = (time * BOB_FREQUENCY).sin() * BOB_AMPLITUDE;
}

/// Build a 4-vertex camera-facing billboard quad for a flat (non-block) item.
///
/// The quad is centered at `item.position + bob_offset` and its plane is
/// perpendicular to the horizontal vector toward the camera. Vertex order
/// is bottom-left, bottom-right, top-right, top-left (CCW when viewed from
/// the camera).
pub fn flat_item_quad(item: &DroppedItemRender, camera_pos: [f32; 3]) -> Vec<[f32; 3]> {
    let center = [
        item.position[0],
        item.position[1] + item.bob_offset,
        item.position[2],
    ];

    // Horizontal vector from item -> camera (we ignore Y for an upright quad).
    let dx = camera_pos[0] - center[0];
    let dz = camera_pos[2] - center[2];
    let len = (dx * dx + dz * dz).sqrt();

    // Right vector is perpendicular to the view direction in the XZ plane.
    let (right_x, right_z) = if len > f32::EPSILON {
        (-dz / len, dx / len)
    } else {
        (1.0, 0.0)
    };

    let half = FLAT_QUAD_SCALE * 0.5;
    let rx = right_x * half;
    let rz = right_z * half;

    let bl = [center[0] - rx, center[1] - half, center[2] - rz];
    let br = [center[0] + rx, center[1] - half, center[2] + rz];
    let tr = [center[0] + rx, center[1] + half, center[2] + rz];
    let tl = [center[0] - rx, center[1] + half, center[2] - rz];

    vec![bl, br, tr, tl]
}

/// Build the 24 face-corner vertices for a small cube representing a
/// dropped block item. The cube is centered at `item.position +
/// bob_offset` and uses [`BLOCK_CUBE_SCALE`] as its full edge length.
///
/// Vertices are emitted as 6 quads (one per face), in face order:
/// `+X, -X, +Y, -Y, +Z, -Z`. Each face lists its 4 corners in CCW
/// order when viewed from outside the cube, for a total of 24 vertices.
pub fn block_item_cube(item: &DroppedItemRender) -> Vec<[f32; 3]> {
    let half = BLOCK_CUBE_SCALE * 0.5;
    let cx = item.position[0];
    let cy = item.position[1] + item.bob_offset;
    let cz = item.position[2];

    let xn = cx - half;
    let xp = cx + half;
    let yn = cy - half;
    let yp = cy + half;
    let zn = cz - half;
    let zp = cz + half;

    vec![
        // +X face (looking toward -X)
        [xp, yn, zn],
        [xp, yn, zp],
        [xp, yp, zp],
        [xp, yp, zn],
        // -X face
        [xn, yn, zp],
        [xn, yn, zn],
        [xn, yp, zn],
        [xn, yp, zp],
        // +Y face (top)
        [xn, yp, zn],
        [xp, yp, zn],
        [xp, yp, zp],
        [xn, yp, zp],
        // -Y face (bottom)
        [xn, yn, zp],
        [xp, yn, zp],
        [xp, yn, zn],
        [xn, yn, zn],
        // +Z face
        [xp, yn, zp],
        [xn, yn, zp],
        [xn, yp, zp],
        [xp, yp, zp],
        // -Z face
        [xn, yn, zn],
        [xp, yn, zn],
        [xp, yp, zn],
        [xn, yp, zn],
    ]
}

/// Returns true if the given item id corresponds to a block (cube model).
pub fn is_block_item(item_id: u16) -> bool {
    item_id < BLOCK_ID_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_defaults() {
        let item = DroppedItemRender::new(42, [1.0, 2.0, 3.0]);
        assert_eq!(item.item_id, 42);
        assert_eq!(item.position, [1.0, 2.0, 3.0]);
        assert_eq!(item.rotation, 0.0);
        assert_eq!(item.bob_offset, 0.0);
        assert_eq!(item.count_label, 1);
    }

    #[test]
    fn tick_advances_rotation_at_half_rad_per_sec() {
        let mut item = DroppedItemRender::new(1, [0.0, 0.0, 0.0]);
        tick_dropped_item(&mut item, 1.0, 0.0);
        assert!((item.rotation - 0.5).abs() < 1e-6);
    }

    #[test]
    fn tick_wraps_rotation_within_tau() {
        let mut item = DroppedItemRender::new(1, [0.0, 0.0, 0.0]);
        item.rotation = std::f32::consts::TAU - 0.1;
        tick_dropped_item(&mut item, 1.0, 0.0);
        assert!(item.rotation >= 0.0);
        assert!(item.rotation < std::f32::consts::TAU);
    }

    #[test]
    fn tick_bob_uses_sine_of_double_time() {
        let mut item = DroppedItemRender::new(1, [0.0, 0.0, 0.0]);
        let t = 1.0_f32;
        tick_dropped_item(&mut item, 0.0, t);
        let expected = (t * BOB_FREQUENCY).sin() * BOB_AMPLITUDE;
        assert!((item.bob_offset - expected).abs() < 1e-6);
    }

    #[test]
    fn tick_bob_zero_at_time_zero() {
        let mut item = DroppedItemRender::new(1, [0.0, 0.0, 0.0]);
        tick_dropped_item(&mut item, 0.0, 0.0);
        assert!(item.bob_offset.abs() < 1e-6);
    }

    #[test]
    fn flat_quad_has_four_vertices() {
        let item = DroppedItemRender::new(2000, [0.0, 0.0, 0.0]);
        let verts = flat_item_quad(&item, [5.0, 0.0, 0.0]);
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn flat_quad_vertical_extent_matches_scale() {
        let item = DroppedItemRender::new(2000, [0.0, 10.0, 0.0]);
        let verts = flat_item_quad(&item, [10.0, 10.0, 0.0]);
        let min_y = verts.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
        let max_y = verts.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y - min_y - FLAT_QUAD_SCALE).abs() < 1e-6);
    }

    #[test]
    fn flat_quad_is_perpendicular_to_camera_direction() {
        // Camera on +X axis -> quad should extend along Z.
        let item = DroppedItemRender::new(2000, [0.0, 0.0, 0.0]);
        let verts = flat_item_quad(&item, [10.0, 0.0, 0.0]);
        let min_z = verts.iter().map(|v| v[2]).fold(f32::INFINITY, f32::min);
        let max_z = verts.iter().map(|v| v[2]).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_z - min_z - FLAT_QUAD_SCALE).abs() < 1e-6);
    }

    #[test]
    fn flat_quad_handles_camera_at_item_position() {
        // Degenerate case: camera coincides with item -> still emit a quad.
        let item = DroppedItemRender::new(2000, [0.0, 0.0, 0.0]);
        let verts = flat_item_quad(&item, [0.0, 0.0, 0.0]);
        assert_eq!(verts.len(), 4);
        assert!(verts.iter().all(|v| v.iter().all(|c| c.is_finite())));
    }

    #[test]
    fn block_cube_has_24_vertices() {
        let item = DroppedItemRender::new(1, [0.0, 0.0, 0.0]);
        let verts = block_item_cube(&item);
        assert_eq!(verts.len(), 24);
    }

    #[test]
    fn block_cube_has_correct_edge_length() {
        let item = DroppedItemRender::new(1, [10.0, 20.0, 30.0]);
        let verts = block_item_cube(&item);
        for axis in 0..3 {
            let min = verts.iter().map(|v| v[axis]).fold(f32::INFINITY, f32::min);
            let max = verts.iter().map(|v| v[axis]).fold(f32::NEG_INFINITY, f32::max);
            assert!((max - min - BLOCK_CUBE_SCALE).abs() < 1e-6);
        }
    }

    #[test]
    fn block_cube_centered_on_position_with_bob() {
        let mut item = DroppedItemRender::new(1, [0.0, 0.0, 0.0]);
        item.bob_offset = 0.05;
        let verts = block_item_cube(&item);
        let avg_y: f32 = verts.iter().map(|v| v[1]).sum::<f32>() / verts.len() as f32;
        assert!((avg_y - 0.05).abs() < 1e-6);
    }

    #[test]
    fn is_block_item_threshold_at_1000() {
        assert!(is_block_item(0));
        assert!(is_block_item(999));
        assert!(!is_block_item(1000));
        assert!(!is_block_item(2000));
    }
}
