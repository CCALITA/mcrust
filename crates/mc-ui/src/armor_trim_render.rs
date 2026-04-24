//! Armor trim rendering data for the HUD / inventory overlay.
//!
//! Minecraft 1.20+ armor trims allow decorating armor pieces with a pattern
//! and a material tint. This module provides the data model and lookup helpers
//! needed by the UI layer to render trim overlays: material colors, pattern UV
//! offsets, glow flags, display names, and priority ordering.

/// Total number of supported trim materials.
const MATERIAL_COUNT: usize = 10;

/// RGB colors for each material index (0..9).
///
/// Order: Iron, Copper, Gold, Lapis, Emerald, Diamond, Netherite, Redstone,
/// Amethyst, Quartz.
const MATERIAL_COLORS: [[f32; 3]; MATERIAL_COUNT] = [
    [0.78, 0.78, 0.78], // 0 Iron      — silver
    [0.72, 0.45, 0.20], // 1 Copper    — orange
    [1.00, 0.82, 0.00], // 2 Gold      — yellow
    [0.15, 0.25, 0.70], // 3 Lapis     — blue
    [0.10, 0.75, 0.30], // 4 Emerald   — green
    [0.25, 0.88, 0.82], // 5 Diamond   — cyan
    [0.20, 0.20, 0.20], // 6 Netherite — dark
    [0.85, 0.10, 0.10], // 7 Redstone  — red
    [0.60, 0.30, 0.80], // 8 Amethyst  — purple
    [0.95, 0.95, 0.95], // 9 Quartz    — white
];

/// Human-readable material names in the same index order.
const MATERIAL_NAMES: [&str; MATERIAL_COUNT] = [
    "Iron",
    "Copper",
    "Gold",
    "Lapis",
    "Emerald",
    "Diamond",
    "Netherite",
    "Redstone",
    "Amethyst",
    "Quartz",
];

/// Priority values for each material (higher = rarer / more prominent).
const MATERIAL_PRIORITIES: [u8; MATERIAL_COUNT] = [
    1, // Iron
    2, // Copper
    5, // Gold
    4, // Lapis
    6, // Emerald
    7, // Diamond
    9, // Netherite
    3, // Redstone
    8, // Amethyst
    0, // Quartz
];

/// Data needed to render an armor trim overlay for a single armor piece.
#[derive(Debug, Clone, PartialEq)]
pub struct TrimRenderInfo {
    /// Pattern identifier (maps to a texture atlas entry).
    pub pattern_id: u16,
    /// RGB tint derived from the trim material.
    pub material_color: [f32; 3],
    /// Whether the trim should glow in the dark (currently always `false`).
    pub glow: bool,
}

/// Return the RGB color for the given material index.
///
/// Unknown indices fall back to white `[1.0, 1.0, 1.0]`.
pub fn material_color(material: u8) -> [f32; 3] {
    MATERIAL_COLORS
        .get(material as usize)
        .copied()
        .unwrap_or([1.0, 1.0, 1.0])
}

/// Return the display name for the given material index.
///
/// Unknown indices return `"Unknown"`.
pub fn material_name(material: u8) -> &'static str {
    MATERIAL_NAMES
        .get(material as usize)
        .copied()
        .unwrap_or("Unknown")
}

/// Compute the texture-atlas UV offset `(u, v)` for a pattern overlay.
///
/// Patterns are laid out in a 16-column atlas; each cell is 1/16 of the
/// texture width and height.
pub fn pattern_overlay_uv(id: u16) -> (f32, f32) {
    let col = (id % 16) as f32;
    let row = (id / 16) as f32;
    (col / 16.0, row / 16.0)
}

/// Whether the given material should glow in the dark.
///
/// Currently no materials glow; returns `false` for all inputs.
pub fn should_glow_in_dark(_material: u8) -> bool {
    false
}

/// Return the priority tier for the given material (higher = rarer).
///
/// Unknown indices return `0`.
pub fn armor_trim_priority(material: u8) -> u8 {
    MATERIAL_PRIORITIES
        .get(material as usize)
        .copied()
        .unwrap_or(0)
}

/// Return the total number of supported trim materials.
pub fn total_materials() -> usize {
    MATERIAL_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TrimRenderInfo construction ─────────────────────────────────

    #[test]
    fn trim_render_info_fields() {
        let info = TrimRenderInfo {
            pattern_id: 3,
            material_color: [0.78, 0.78, 0.78],
            glow: false,
        };
        assert_eq!(info.pattern_id, 3);
        assert_eq!(info.material_color, [0.78, 0.78, 0.78]);
        assert!(!info.glow);
    }

    #[test]
    fn trim_render_info_clone_eq() {
        let a = TrimRenderInfo {
            pattern_id: 1,
            material_color: [0.5, 0.5, 0.5],
            glow: false,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ── material_color ──────────────────────────────────────────────

    #[test]
    fn iron_is_silver() {
        let c = material_color(0);
        assert!((c[0] - 0.78).abs() < f32::EPSILON);
        assert!((c[1] - 0.78).abs() < f32::EPSILON);
        assert!((c[2] - 0.78).abs() < f32::EPSILON);
    }

    #[test]
    fn copper_is_orange() {
        let c = material_color(1);
        assert!((c[0] - 0.72).abs() < f32::EPSILON);
        assert!((c[1] - 0.45).abs() < f32::EPSILON);
        assert!((c[2] - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn gold_is_yellow() {
        let c = material_color(2);
        assert!((c[0] - 1.00).abs() < f32::EPSILON);
        assert!((c[1] - 0.82).abs() < f32::EPSILON);
        assert!((c[2] - 0.00).abs() < f32::EPSILON);
    }

    #[test]
    fn lapis_is_blue() {
        let c = material_color(3);
        assert!((c[0] - 0.15).abs() < f32::EPSILON);
        assert!((c[1] - 0.25).abs() < f32::EPSILON);
        assert!((c[2] - 0.70).abs() < f32::EPSILON);
    }

    #[test]
    fn emerald_is_green() {
        let c = material_color(4);
        assert!((c[0] - 0.10).abs() < f32::EPSILON);
        assert!((c[1] - 0.75).abs() < f32::EPSILON);
        assert!((c[2] - 0.30).abs() < f32::EPSILON);
    }

    #[test]
    fn diamond_is_cyan() {
        let c = material_color(5);
        assert!((c[0] - 0.25).abs() < f32::EPSILON);
        assert!((c[1] - 0.88).abs() < f32::EPSILON);
        assert!((c[2] - 0.82).abs() < f32::EPSILON);
    }

    #[test]
    fn netherite_is_dark() {
        let c = material_color(6);
        assert!((c[0] - 0.20).abs() < f32::EPSILON);
        assert!((c[1] - 0.20).abs() < f32::EPSILON);
        assert!((c[2] - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn redstone_is_red() {
        let c = material_color(7);
        assert!((c[0] - 0.85).abs() < f32::EPSILON);
        assert!((c[1] - 0.10).abs() < f32::EPSILON);
        assert!((c[2] - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn amethyst_is_purple() {
        let c = material_color(8);
        assert!((c[0] - 0.60).abs() < f32::EPSILON);
        assert!((c[1] - 0.30).abs() < f32::EPSILON);
        assert!((c[2] - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn quartz_is_white() {
        let c = material_color(9);
        assert!((c[0] - 0.95).abs() < f32::EPSILON);
        assert!((c[1] - 0.95).abs() < f32::EPSILON);
        assert!((c[2] - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn unknown_material_color_is_white() {
        let c = material_color(255);
        assert_eq!(c, [1.0, 1.0, 1.0]);
    }

    // ── material_name ───────────────────────────────────────────────

    #[test]
    fn all_material_names() {
        let expected = [
            "Iron", "Copper", "Gold", "Lapis", "Emerald",
            "Diamond", "Netherite", "Redstone", "Amethyst", "Quartz",
        ];
        for (i, name) in expected.iter().enumerate() {
            assert_eq!(material_name(i as u8), *name);
        }
    }

    #[test]
    fn unknown_material_name() {
        assert_eq!(material_name(10), "Unknown");
        assert_eq!(material_name(255), "Unknown");
    }

    // ── pattern_overlay_uv ──────────────────────────────────────────

    #[test]
    fn pattern_zero_is_origin() {
        let (u, v) = pattern_overlay_uv(0);
        assert!((u - 0.0).abs() < f32::EPSILON);
        assert!((v - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pattern_one_is_first_column() {
        let (u, v) = pattern_overlay_uv(1);
        assert!((u - 1.0 / 16.0).abs() < f32::EPSILON);
        assert!((v - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pattern_wraps_at_16() {
        let (u, v) = pattern_overlay_uv(16);
        assert!((u - 0.0).abs() < f32::EPSILON);
        assert!((v - 1.0 / 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pattern_17_is_second_row_second_col() {
        let (u, v) = pattern_overlay_uv(17);
        assert!((u - 1.0 / 16.0).abs() < f32::EPSILON);
        assert!((v - 1.0 / 16.0).abs() < f32::EPSILON);
    }

    // ── should_glow_in_dark ─────────────────────────────────────────

    #[test]
    fn no_material_glows() {
        for m in 0..=255u8 {
            assert!(!should_glow_in_dark(m));
        }
    }

    // ── armor_trim_priority ─────────────────────────────────────────

    #[test]
    fn netherite_highest_priority() {
        let max = (0..MATERIAL_COUNT as u8)
            .max_by_key(|m| armor_trim_priority(*m))
            .expect("non-empty");
        assert_eq!(max, 6); // Netherite
        assert_eq!(armor_trim_priority(6), 9);
    }

    #[test]
    fn quartz_lowest_priority() {
        let min = (0..MATERIAL_COUNT as u8)
            .min_by_key(|m| armor_trim_priority(*m))
            .expect("non-empty");
        assert_eq!(min, 9); // Quartz
        assert_eq!(armor_trim_priority(9), 0);
    }

    #[test]
    fn unknown_priority_is_zero() {
        assert_eq!(armor_trim_priority(200), 0);
    }

    // ── total_materials ─────────────────────────────────────────────

    #[test]
    fn total_is_ten() {
        assert_eq!(total_materials(), 10);
    }
}
