/// Name tag rendering: camera-facing billboard quads for entity name display,
/// visibility distance checks, and sneaking transparency.

/// Maximum distance (in blocks) at which a name tag is visible.
const NAME_TAG_VISIBLE_DISTANCE: f32 = 64.0;

/// Alpha value for name tags when the entity is sneaking.
const SNEAKING_ALPHA: f32 = 0.3;

/// Height of the name tag quad in blocks.
const NAME_TAG_HEIGHT: f32 = 0.25;

/// A name tag attached to an entity.
pub struct NameTag {
    /// The text displayed on the name tag.
    pub text: String,
    /// Whether the name tag is currently visible.
    pub visible: bool,
    /// Background transparency (0.0 = fully transparent, 1.0 = fully opaque).
    pub background_alpha: f32,
}

impl NameTag {
    /// Creates a new name tag with default visibility and alpha.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            visible: true,
            background_alpha: 0.5,
        }
    }
}

/// Generates 4 vertices for a camera-facing billboard quad centered at `pos`.
///
/// The quad is `text_width` blocks wide and `NAME_TAG_HEIGHT` (0.25) blocks
/// tall, oriented to always face the camera position in the XZ plane.
pub fn name_tag_billboard(
    pos: [f32; 3],
    camera_pos: [f32; 3],
    text_width: f32,
) -> Vec<[f32; 3]> {
    let dx = camera_pos[0] - pos[0];
    let dz = camera_pos[2] - pos[2];
    let len = (dx * dx + dz * dz).sqrt();

    // Perpendicular direction in XZ for the billboard width
    let (right_x, right_z) = if len > 1e-6 {
        (-dz / len, dx / len)
    } else {
        // Camera directly above; arbitrary facing
        (1.0, 0.0)
    };

    let half_w = text_width / 2.0;
    let half_h = NAME_TAG_HEIGHT / 2.0;

    vec![
        [
            pos[0] - right_x * half_w,
            pos[1] - half_h,
            pos[2] - right_z * half_w,
        ],
        [
            pos[0] + right_x * half_w,
            pos[1] - half_h,
            pos[2] + right_z * half_w,
        ],
        [
            pos[0] + right_x * half_w,
            pos[1] + half_h,
            pos[2] + right_z * half_w,
        ],
        [
            pos[0] - right_x * half_w,
            pos[1] + half_h,
            pos[2] - right_z * half_w,
        ],
    ]
}

/// Returns the maximum render distance for name tags (64 blocks).
pub fn name_tag_visible_distance() -> f32 {
    NAME_TAG_VISIBLE_DISTANCE
}

/// Returns the background alpha used when the entity is sneaking (0.3).
pub fn sneaking_name_tag_alpha() -> f32 {
    SNEAKING_ALPHA
}

/// Determines whether a name tag should be rendered given the distance
/// to the camera and whether the entity is sneaking.
///
/// Name tags are hidden beyond 64 blocks. When sneaking, they are only
/// visible within 4 blocks.
pub fn should_render_name_tag(distance: f32, sneaking: bool) -> bool {
    if distance > NAME_TAG_VISIBLE_DISTANCE {
        return false;
    }
    if sneaking {
        return distance <= 4.0;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_name_tag_has_defaults() {
        let tag = NameTag::new("Steve");
        assert_eq!(tag.text, "Steve");
        assert!(tag.visible);
        assert!((tag.background_alpha - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn new_name_tag_accepts_string() {
        let tag = NameTag::new(String::from("Alex"));
        assert_eq!(tag.text, "Alex");
    }

    #[test]
    fn billboard_produces_4_vertices() {
        let verts = name_tag_billboard([0.0, 2.0, 0.0], [5.0, 2.0, 5.0], 1.0);
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn billboard_quad_height_is_correct() {
        let verts = name_tag_billboard([0.0, 10.0, 0.0], [5.0, 10.0, 0.0], 2.0);
        let min_y = verts.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
        let max_y = verts.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max);
        let height = max_y - min_y;
        assert!(
            (height - NAME_TAG_HEIGHT).abs() < 1e-5,
            "quad height should be {NAME_TAG_HEIGHT}, got {height}"
        );
    }

    #[test]
    fn billboard_with_camera_directly_above() {
        let verts = name_tag_billboard([0.0, 5.0, 0.0], [0.0, 100.0, 0.0], 1.5);
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn billboard_width_scales_with_text_width() {
        let narrow = name_tag_billboard([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], 1.0);
        let wide = name_tag_billboard([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], 3.0);

        let narrow_span = (narrow[1][2] - narrow[0][2]).abs();
        let wide_span = (wide[1][2] - wide[0][2]).abs();
        assert!(
            wide_span > narrow_span,
            "wider text_width should produce wider quad"
        );
    }

    #[test]
    fn visible_distance_is_64() {
        assert!((name_tag_visible_distance() - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sneaking_alpha_is_0_3() {
        assert!((sneaking_name_tag_alpha() - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn render_within_range_not_sneaking() {
        assert!(should_render_name_tag(10.0, false));
        assert!(should_render_name_tag(63.9, false));
    }

    #[test]
    fn no_render_beyond_max_distance() {
        assert!(!should_render_name_tag(64.1, false));
        assert!(!should_render_name_tag(100.0, false));
    }

    #[test]
    fn sneaking_visible_within_4_blocks() {
        assert!(should_render_name_tag(3.0, true));
        assert!(should_render_name_tag(4.0, true));
    }

    #[test]
    fn sneaking_hidden_beyond_4_blocks() {
        assert!(!should_render_name_tag(4.1, true));
        assert!(!should_render_name_tag(30.0, true));
    }

    #[test]
    fn sneaking_hidden_beyond_max_distance() {
        assert!(!should_render_name_tag(65.0, true));
    }

    #[test]
    fn render_at_exact_max_distance() {
        assert!(should_render_name_tag(64.0, false));
    }

    #[test]
    fn render_at_zero_distance() {
        assert!(should_render_name_tag(0.0, false));
        assert!(should_render_name_tag(0.0, true));
    }
}
