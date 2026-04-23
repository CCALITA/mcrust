//! Stairs and slabs block shape system.
//!
//! Defines [`StairShape`], [`StairHalf`], and [`StairState`] for stair collision geometry,
//! plus the [`slab_type`] helper and [`STAIR_MATERIALS`] registry.

/// Shape variant for stair blocks, determined by adjacent stair connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StairShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

/// Whether a stair or slab occupies the top or bottom half of the block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StairHalf {
    Top,
    Bottom,
}

/// Full state of a placed stair block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StairState {
    pub shape: StairShape,
    pub half: StairHalf,
    pub facing: u8,
    pub material: u8,
}

impl StairState {
    /// Create a new stair state with the given material, facing direction, and half.
    /// Defaults to [`StairShape::Straight`].
    pub fn new(material: u8, facing: u8, half: StairHalf) -> Self {
        Self {
            shape: StairShape::Straight,
            half,
            facing,
            material,
        }
    }
}

/// Return the slab type encoding: 0 = bottom, 1 = top, 2 = double.
pub fn slab_type(is_top: bool, is_double: bool) -> u8 {
    if is_double {
        2
    } else if is_top {
        1
    } else {
        0
    }
}

/// Compute the axis-aligned bounding boxes for a stair block.
///
/// Each AABB is `[min_x, min_y, min_z, max_x, max_y, max_z]`.
///
/// - **Straight** stairs produce 2 boxes: a half-slab base and a step block whose
///   position depends on `facing` (0=north, 1=south, 2=west, 3=east).
/// - **Inner** corners produce 2 boxes: base slab + a larger L-shaped approximation.
/// - **Outer** corners produce 2 boxes: base slab + a quarter-block step.
///
/// Top-half stairs have their Y coordinates inverted (mirrored about y=0.5).
pub fn stair_collision_boxes(state: &StairState) -> Vec<[f32; 6]> {
    let mut boxes = Vec::new();

    // Base slab (always present)
    let base: [f32; 6] = [0.0, 0.0, 0.0, 1.0, 0.5, 1.0];
    boxes.push(base);

    match state.shape {
        StairShape::Straight => {
            let step = match state.facing {
                0 => [0.0, 0.5, 0.0, 1.0, 1.0, 0.5], // north
                1 => [0.0, 0.5, 0.5, 1.0, 1.0, 1.0], // south
                2 => [0.0, 0.5, 0.0, 0.5, 1.0, 1.0], // west
                _ => [0.5, 0.5, 0.0, 1.0, 1.0, 1.0], // east (3 and fallback)
            };
            boxes.push(step);
        }
        StairShape::InnerLeft | StairShape::InnerRight => {
            // Inner corner: base slab + three-quarter step
            let step = match (state.shape, state.facing) {
                (StairShape::InnerLeft, 0) => [0.0, 0.5, 0.0, 1.0, 1.0, 0.5],
                (StairShape::InnerRight, 0) => [0.0, 0.5, 0.0, 0.5, 1.0, 1.0],
                (StairShape::InnerLeft, 1) => [0.0, 0.5, 0.5, 1.0, 1.0, 1.0],
                (StairShape::InnerRight, 1) => [0.5, 0.5, 0.0, 1.0, 1.0, 1.0],
                (StairShape::InnerLeft, 2) => [0.0, 0.5, 0.0, 0.5, 1.0, 1.0],
                (StairShape::InnerRight, 2) => [0.0, 0.5, 0.0, 1.0, 1.0, 0.5],
                (StairShape::InnerLeft, _) => [0.5, 0.5, 0.0, 1.0, 1.0, 1.0],
                (StairShape::InnerRight, _) => [0.0, 0.5, 0.5, 1.0, 1.0, 1.0],
                _ => unreachable!(),
            };
            boxes.push(step);
        }
        StairShape::OuterLeft | StairShape::OuterRight => {
            // Outer corner: base slab + quarter-block step
            let step = match (state.shape, state.facing) {
                (StairShape::OuterLeft, 0) => [0.0, 0.5, 0.0, 0.5, 1.0, 0.5],
                (StairShape::OuterRight, 0) => [0.5, 0.5, 0.0, 1.0, 1.0, 0.5],
                (StairShape::OuterLeft, 1) => [0.5, 0.5, 0.5, 1.0, 1.0, 1.0],
                (StairShape::OuterRight, 1) => [0.0, 0.5, 0.5, 0.5, 1.0, 1.0],
                (StairShape::OuterLeft, 2) => [0.0, 0.5, 0.5, 0.5, 1.0, 1.0],
                (StairShape::OuterRight, 2) => [0.0, 0.5, 0.0, 0.5, 1.0, 0.5],
                (StairShape::OuterLeft, _) => [0.5, 0.5, 0.0, 1.0, 1.0, 0.5],
                (StairShape::OuterRight, _) => [0.5, 0.5, 0.5, 1.0, 1.0, 1.0],
                _ => unreachable!(),
            };
            boxes.push(step);
        }
    }

    // Top-half stairs: invert Y coordinates (mirror about y = 0.5)
    if state.half == StairHalf::Top {
        for aabb in &mut boxes {
            let old_min_y = aabb[1];
            let old_max_y = aabb[4];
            aabb[1] = 1.0 - old_max_y;
            aabb[4] = 1.0 - old_min_y;
        }
    }

    boxes
}

/// Block materials that have stair variants.
pub const STAIR_MATERIALS: &[&str] = &[
    "oak",
    "cobblestone",
    "stone_brick",
    "birch",
    "spruce",
    "jungle",
    "dark_oak",
    "brick",
    "sandstone",
    "quartz",
    "nether_brick",
    "purpur",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_stair_has_two_collision_boxes() {
        let state = StairState::new(0, 0, StairHalf::Bottom);
        let boxes = stair_collision_boxes(&state);
        assert_eq!(boxes.len(), 2);
    }

    #[test]
    fn top_half_straight_stair_has_two_collision_boxes() {
        let state = StairState::new(0, 1, StairHalf::Top);
        let boxes = stair_collision_boxes(&state);
        assert_eq!(boxes.len(), 2);
    }

    #[test]
    fn bottom_slab_type_is_zero() {
        assert_eq!(slab_type(false, false), 0);
    }

    #[test]
    fn top_slab_type_is_one() {
        assert_eq!(slab_type(true, false), 1);
    }

    #[test]
    fn double_slab_type_is_two() {
        assert_eq!(slab_type(false, true), 2);
        assert_eq!(slab_type(true, true), 2);
    }

    #[test]
    fn stair_materials_count() {
        assert_eq!(STAIR_MATERIALS.len(), 12);
    }

    #[test]
    fn stair_shape_variants_are_distinct() {
        let variants = [
            StairShape::Straight,
            StairShape::InnerLeft,
            StairShape::InnerRight,
            StairShape::OuterLeft,
            StairShape::OuterRight,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn new_defaults_to_straight_shape() {
        let state = StairState::new(5, 2, StairHalf::Bottom);
        assert_eq!(state.shape, StairShape::Straight);
        assert_eq!(state.material, 5);
        assert_eq!(state.facing, 2);
        assert_eq!(state.half, StairHalf::Bottom);
    }

    #[test]
    fn top_half_inverts_y_coordinates() {
        let bottom = StairState::new(0, 0, StairHalf::Bottom);
        let top = StairState::new(0, 0, StairHalf::Top);
        let bottom_boxes = stair_collision_boxes(&bottom);
        let top_boxes = stair_collision_boxes(&top);

        // Base slab: bottom [0,0,0,1,0.5,1] -> top [0,0.5,0,1,1,1]
        assert_eq!(top_boxes[0][1], 0.5);
        assert_eq!(top_boxes[0][4], 1.0);

        // Step: bottom [0,0.5,0,1,1,0.5] -> top [0,0,0,1,0.5,0.5]
        assert_eq!(top_boxes[1][1], 0.0);
        assert_eq!(top_boxes[1][4], 0.5);

        // Both should have 2 boxes
        assert_eq!(bottom_boxes.len(), top_boxes.len());
    }

    #[test]
    fn inner_corner_has_two_collision_boxes() {
        let mut state = StairState::new(0, 0, StairHalf::Bottom);
        state.shape = StairShape::InnerLeft;
        let boxes = stair_collision_boxes(&state);
        assert_eq!(boxes.len(), 2);
    }

    #[test]
    fn outer_corner_has_two_collision_boxes() {
        let mut state = StairState::new(0, 0, StairHalf::Bottom);
        state.shape = StairShape::OuterRight;
        let boxes = stair_collision_boxes(&state);
        assert_eq!(boxes.len(), 2);
    }
}
