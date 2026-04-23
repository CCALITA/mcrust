//! Debug screen (F3) data formatting.
//!
//! Provides [`DebugInfo`] for collecting runtime diagnostics and
//! [`format_debug_lines`] for rendering them as Minecraft-style F3 overlay text.

/// All data displayed on the F3 debug screen.
#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub fps: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub facing_yaw: f32,
    pub facing_pitch: f32,
    pub biome: String,
    pub block_light: u8,
    pub sky_light: u8,
    pub loaded_chunks: usize,
    pub entity_count: usize,
    pub dimension: String,
    pub day: u32,
    pub time_of_day: f32,
    pub seed: u64,
}

/// Return a cardinal or intercardinal direction name for the given yaw angle.
///
/// Yaw is in degrees: North ~ 0/360, East ~ 90, South ~ 180, West ~ 270.
/// Each of the eight sectors spans 45 degrees centered on its principal angle.
pub fn direction_name(yaw: f32) -> &'static str {
    // Normalize yaw into [0, 360).
    let normalized = ((yaw % 360.0) + 360.0) % 360.0;

    match normalized {
        y if y < 22.5 => "North",
        y if y < 67.5 => "North-East",
        y if y < 112.5 => "East",
        y if y < 157.5 => "South-East",
        y if y < 202.5 => "South",
        y if y < 247.5 => "South-West",
        y if y < 292.5 => "West",
        y if y < 337.5 => "North-West",
        _ => "North",
    }
}

/// Convert a normalized time-of-day (0.0..1.0) to a `"HH:MM"` string.
///
/// Mapping: 0.0 = 06:00, 0.25 = 12:00, 0.5 = 18:00, 0.75 = 00:00.
pub fn time_to_hhmm(time_of_day: f32) -> String {
    // time_of_day 0.0 → 6:00 means the offset is +6 hours.
    let total_hours = (time_of_day * 24.0 + 6.0) % 24.0;
    let hours = total_hours as u32;
    let minutes = ((total_hours - hours as f32) * 60.0) as u32;
    format!("{hours:02}:{minutes:02}")
}

/// Format all debug information into lines matching the Minecraft F3 overlay.
pub fn format_debug_lines(info: &DebugInfo) -> Vec<String> {
    let block_x = info.x.floor() as i32;
    let block_y = info.y.floor() as i32;
    let block_z = info.z.floor() as i32;

    let direction = direction_name(info.facing_yaw);
    let time_str = time_to_hhmm(info.time_of_day);

    vec![
        format!("MCRust ({:.0} fps)", info.fps),
        format!("XYZ: {:.2} / {:.2} / {:.2}", info.x, info.y, info.z),
        format!("Block: {} {} {}", block_x, block_y, block_z),
        format!("Chunk: {} {}", info.chunk_x, info.chunk_z),
        format!(
            "Facing: {} ({:.1} / {:.1})",
            direction, info.facing_yaw, info.facing_pitch
        ),
        format!("Biome: {}", info.biome),
        format!("Light: {} sky {}", info.block_light, info.sky_light),
        format!("Chunks: {}", info.loaded_chunks),
        format!("Entities: {}", info.entity_count),
        format!("Day: {} Time: {}", info.day, time_str),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // direction_name
    // ------------------------------------------------------------------

    #[test]
    fn direction_north_at_zero() {
        assert_eq!(direction_name(0.0), "North");
    }

    #[test]
    fn direction_north_at_360() {
        assert_eq!(direction_name(360.0), "North");
    }

    #[test]
    fn direction_east_at_90() {
        assert_eq!(direction_name(90.0), "East");
    }

    #[test]
    fn direction_south_at_180() {
        assert_eq!(direction_name(180.0), "South");
    }

    #[test]
    fn direction_west_at_270() {
        assert_eq!(direction_name(270.0), "West");
    }

    #[test]
    fn direction_north_east_at_45() {
        assert_eq!(direction_name(45.0), "North-East");
    }

    #[test]
    fn direction_south_east_at_135() {
        assert_eq!(direction_name(135.0), "South-East");
    }

    #[test]
    fn direction_south_west_at_225() {
        assert_eq!(direction_name(225.0), "South-West");
    }

    #[test]
    fn direction_north_west_at_315() {
        assert_eq!(direction_name(315.0), "North-West");
    }

    #[test]
    fn direction_negative_yaw() {
        // -90 degrees should normalize to 270 → West
        assert_eq!(direction_name(-90.0), "West");
    }

    #[test]
    fn direction_large_positive_yaw() {
        // 450 degrees should normalize to 90 → East
        assert_eq!(direction_name(450.0), "East");
    }

    // ------------------------------------------------------------------
    // time_to_hhmm
    // ------------------------------------------------------------------

    #[test]
    fn time_zero_is_six_am() {
        assert_eq!(time_to_hhmm(0.0), "06:00");
    }

    #[test]
    fn time_quarter_is_noon() {
        assert_eq!(time_to_hhmm(0.25), "12:00");
    }

    #[test]
    fn time_half_is_six_pm() {
        assert_eq!(time_to_hhmm(0.5), "18:00");
    }

    #[test]
    fn time_three_quarter_is_midnight() {
        assert_eq!(time_to_hhmm(0.75), "00:00");
    }

    // ------------------------------------------------------------------
    // format_debug_lines
    // ------------------------------------------------------------------

    fn sample_info() -> DebugInfo {
        DebugInfo {
            fps: 60.0,
            x: 123.45,
            y: 64.00,
            z: -789.12,
            chunk_x: 7,
            chunk_z: -50,
            facing_yaw: 90.0,
            facing_pitch: -15.0,
            biome: "plains".to_string(),
            block_light: 12,
            sky_light: 15,
            loaded_chunks: 441,
            entity_count: 37,
            dimension: "overworld".to_string(),
            day: 5,
            time_of_day: 0.25,
            seed: 12345,
        }
    }

    #[test]
    fn debug_lines_has_correct_count() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines.len(), 10);
    }

    #[test]
    fn debug_lines_fps() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[0], "MCRust (60 fps)");
    }

    #[test]
    fn debug_lines_xyz() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[1], "XYZ: 123.45 / 64.00 / -789.12");
    }

    #[test]
    fn debug_lines_block() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[2], "Block: 123 64 -790");
    }

    #[test]
    fn debug_lines_chunk() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[3], "Chunk: 7 -50");
    }

    #[test]
    fn debug_lines_facing() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[4], "Facing: East (90.0 / -15.0)");
    }

    #[test]
    fn debug_lines_biome() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[5], "Biome: plains");
    }

    #[test]
    fn debug_lines_light() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[6], "Light: 12 sky 15");
    }

    #[test]
    fn debug_lines_chunks() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[7], "Chunks: 441");
    }

    #[test]
    fn debug_lines_entities() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[8], "Entities: 37");
    }

    #[test]
    fn debug_lines_day_time() {
        let lines = format_debug_lines(&sample_info());
        assert_eq!(lines[9], "Day: 5 Time: 12:00");
    }
}
