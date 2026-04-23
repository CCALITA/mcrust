/// Beacon beam rendering data: billboard quads, stained glass color mapping,
/// and pulsing intensity for the vertical beam effect above beacon blocks.

/// Default beam height in blocks (world limit).
pub const DEFAULT_BEAM_HEIGHT: f32 = 256.0;

/// A beacon beam extending upward from a base position.
pub struct BeaconBeam {
    /// World-space position of the beacon block.
    pub base_pos: [f32; 3],
    /// Vertical extent of the beam in blocks.
    pub height: f32,
    /// RGB color of the beam.
    pub color: [f32; 3],
    /// Phase offset for the pulsing animation (radians).
    pub pulse_phase: f32,
}

/// Generates 4 billboard quads (16 vertices) along the beam from base to top.
///
/// Each quad faces the camera via billboarding around the Y axis. The beam is
/// divided into 4 equal vertical segments, each represented by a quad with 4
/// vertices. The quad half-width is 0.5 blocks.
pub fn generate_beam_quads(beam: &BeaconBeam, camera_pos: [f32; 3]) -> Vec<[f32; 3]> {
    let bx = beam.base_pos[0];
    let by = beam.base_pos[1];
    let bz = beam.base_pos[2];

    // Direction from beam to camera in the XZ plane
    let dx = camera_pos[0] - bx;
    let dz = camera_pos[2] - bz;
    let len = (dx * dx + dz * dz).sqrt();

    // Perpendicular direction in XZ for the billboard half-width
    let (right_x, right_z) = if len > 1e-6 {
        // Perpendicular to the camera direction (rotate 90 degrees in XZ)
        (-dz / len, dx / len)
    } else {
        // Camera directly above; arbitrary facing
        (1.0, 0.0)
    };

    let half_width = 0.5;
    let segment_height = beam.height / 4.0;

    let mut vertices = Vec::with_capacity(16);

    for i in 0..4 {
        let y_bottom = by + segment_height * i as f32;
        let y_top = by + segment_height * (i + 1) as f32;

        // Quad corners: bottom-left, bottom-right, top-right, top-left
        vertices.push([
            bx - right_x * half_width,
            y_bottom,
            bz - right_z * half_width,
        ]);
        vertices.push([
            bx + right_x * half_width,
            y_bottom,
            bz + right_z * half_width,
        ]);
        vertices.push([
            bx + right_x * half_width,
            y_top,
            bz + right_z * half_width,
        ]);
        vertices.push([
            bx - right_x * half_width,
            y_top,
            bz - right_z * half_width,
        ]);
    }

    vertices
}

/// Maps a stained glass color index (0..15) to an RGB beam color.
///
/// Color indices follow the Minecraft dye order:
/// 0=white, 1=orange, 2=magenta, 3=light_blue, 4=yellow, 5=lime,
/// 6=pink, 7=gray, 8=light_gray, 9=cyan, 10=purple, 11=blue,
/// 12=brown, 13=green, 14=red, 15=black.
pub fn beam_color_from_glass(glass_color: u8) -> [f32; 3] {
    match glass_color {
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
    }
}

/// Computes the pulsing beam intensity at a given time.
///
/// Returns a value in `[0.6, 1.0]` using a sine wave offset by `phase`.
pub fn pulse_intensity(phase: f32, time: f32) -> f32 {
    let wave = (time + phase).sin(); // range [-1, 1]
    0.8 + 0.2 * wave                // range [0.6, 1.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beam_quads_produce_16_vertices() {
        let beam = BeaconBeam {
            base_pos: [0.0, 64.0, 0.0],
            height: DEFAULT_BEAM_HEIGHT,
            color: [1.0, 1.0, 1.0],
            pulse_phase: 0.0,
        };
        let verts = generate_beam_quads(&beam, [5.0, 64.0, 5.0]);
        assert_eq!(verts.len(), 16);
    }

    #[test]
    fn beam_quads_span_full_height() {
        let beam = BeaconBeam {
            base_pos: [0.0, 10.0, 0.0],
            height: 100.0,
            color: [1.0, 1.0, 1.0],
            pulse_phase: 0.0,
        };
        let verts = generate_beam_quads(&beam, [5.0, 10.0, 5.0]);

        let min_y = verts.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
        let max_y = verts.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max);

        assert!((min_y - 10.0).abs() < 1e-4, "min_y should be 10.0, got {min_y}");
        assert!((max_y - 110.0).abs() < 1e-4, "max_y should be 110.0, got {max_y}");
    }

    #[test]
    fn beam_quads_with_camera_directly_above() {
        let beam = BeaconBeam {
            base_pos: [0.0, 0.0, 0.0],
            height: 10.0,
            color: [1.0, 1.0, 1.0],
            pulse_phase: 0.0,
        };
        // Camera directly above the beam
        let verts = generate_beam_quads(&beam, [0.0, 100.0, 0.0]);
        assert_eq!(verts.len(), 16);
    }

    #[test]
    fn all_16_glass_colors_return_valid_rgb() {
        for i in 0..16u8 {
            let color = beam_color_from_glass(i);
            for c in &color {
                assert!(*c >= 0.0 && *c <= 1.0, "color {i} component out of range: {c}");
            }
        }
    }

    #[test]
    fn invalid_glass_color_returns_white() {
        let color = beam_color_from_glass(16);
        assert_eq!(color, [1.0, 1.0, 1.0]);
        let color = beam_color_from_glass(255);
        assert_eq!(color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn specific_glass_colors_correct() {
        assert_eq!(beam_color_from_glass(0), [1.0, 1.0, 1.0]); // white
        assert_eq!(beam_color_from_glass(14), [0.7, 0.2, 0.2]); // red
        assert_eq!(beam_color_from_glass(15), [0.1, 0.1, 0.1]); // black
    }

    #[test]
    fn pulse_intensity_stays_in_range() {
        // Sample many time values to verify the range [0.6, 1.0]
        for i in 0..1000 {
            let time = i as f32 * 0.01;
            let intensity = pulse_intensity(0.0, time);
            assert!(
                intensity >= 0.6 - 1e-6 && intensity <= 1.0 + 1e-6,
                "intensity {intensity} out of range at time {time}"
            );
        }
    }

    #[test]
    fn pulse_intensity_varies_with_phase() {
        let a = pulse_intensity(0.0, std::f32::consts::FRAC_PI_2);
        let b = pulse_intensity(std::f32::consts::PI, std::f32::consts::FRAC_PI_2);
        // Different phases should give different intensities (unless at a crossing)
        assert!((a - b).abs() > 0.01, "phase should affect intensity");
    }

    #[test]
    fn default_beam_height_is_256() {
        assert_eq!(DEFAULT_BEAM_HEIGHT, 256.0);
    }
}
