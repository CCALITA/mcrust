//! Visual effects for guardian laser beams, totem activations, and sweep attacks.

/// A guardian's charging laser beam between two points.
#[derive(Debug, Clone)]
pub struct GuardianBeam {
    pub origin: [f32; 3],
    pub target: [f32; 3],
    pub charge_progress: f32,
}

/// Interpolate beam color from cyan to purple as charge progresses from 0 to 1.
///
/// At `charge_progress = 0.0`: cyan `[0.2, 0.8, 0.9]`
/// At `charge_progress = 1.0`: purple `[0.6, 0.1, 0.8]`
pub fn guardian_beam_color(charge_progress: f32) -> [f32; 3] {
    let t = charge_progress.clamp(0.0, 1.0);
    let cyan: [f32; 3] = [0.2, 0.8, 0.9];
    let purple: [f32; 3] = [0.6, 0.1, 0.8];
    [
        cyan[0] + (purple[0] - cyan[0]) * t,
        cyan[1] + (purple[1] - cyan[1]) * t,
        cyan[2] + (purple[2] - cyan[2]) * t,
    ]
}

/// Generate a line of quad vertices from `beam.origin` to `beam.target`.
///
/// Returns a series of points along the beam axis, suitable for building
/// billboard quads during rendering. The number of segments scales with
/// beam length to keep visual density consistent.
pub fn generate_guardian_beam(beam: &GuardianBeam) -> Vec<[f32; 3]> {
    let dx = beam.target[0] - beam.origin[0];
    let dy = beam.target[1] - beam.origin[1];
    let dz = beam.target[2] - beam.origin[2];
    let length = (dx * dx + dy * dy + dz * dz).sqrt();

    if length < 1e-6 {
        return vec![beam.origin];
    }

    // One segment per 0.5 blocks, minimum 2 vertices (start + end).
    let segment_count = ((length / 0.5).ceil() as usize).max(1);
    let vertex_count = segment_count + 1;

    let mut vertices = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let t = i as f32 / segment_count as f32;
        vertices.push([
            beam.origin[0] + dx * t,
            beam.origin[1] + dy * t,
            beam.origin[2] + dz * t,
        ]);
    }

    vertices
}

/// A totem of undying activation effect.
#[derive(Debug, Clone)]
pub struct TotemActivation {
    pub pos: [f32; 3],
    pub time_elapsed: f32,
}

/// Generate totem particle positions and colors in an emerald-green swirl pattern.
///
/// Each particle is `(position, color)`. The swirl expands outward over time
/// and particles are distributed evenly around the activation center.
pub fn totem_particles(activation: &TotemActivation, count: u8) -> Vec<([f32; 3], [f32; 3])> {
    let mut particles = Vec::with_capacity(count as usize);
    let t = activation.time_elapsed;

    for i in 0..count {
        let fraction = i as f32 / count.max(1) as f32;
        let angle = fraction * std::f32::consts::TAU + t * 2.0;
        let radius = 0.5 + t * 0.8;

        let x = activation.pos[0] + angle.cos() * radius;
        let y = activation.pos[1] + fraction * 2.0 + t * 0.5;
        let z = activation.pos[2] + angle.sin() * radius;

        // Emerald green with slight per-particle variation
        let green_variation = 0.6 + fraction * 0.4;
        let color: [f32; 3] = [0.1, green_variation, 0.15];

        particles.push(([x, y, z], color));
    }

    particles
}

/// Generate 8 vertices forming a 120-degree sweep attack arc.
///
/// The arc is centered on `yaw` and lies in the horizontal (XZ) plane at `origin`.
/// Vertices are evenly spaced across the arc at the given `radius`.
pub fn sweep_attack_arc(origin: [f32; 3], yaw: f32, radius: f32) -> Vec<[f32; 3]> {
    let vertex_count = 8;
    let arc_angle = std::f32::consts::FRAC_PI_3 * 2.0; // 120 degrees
    let start_angle = yaw - arc_angle / 2.0;

    let mut vertices = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let t = i as f32 / (vertex_count - 1) as f32;
        let angle = start_angle + arc_angle * t;

        vertices.push([
            origin[0] + angle.cos() * radius,
            origin[1],
            origin[2] + angle.sin() * radius,
        ]);
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Guardian beam color lerp ──────────────────────────────────────

    #[test]
    fn color_at_zero_charge_is_cyan() {
        let c = guardian_beam_color(0.0);
        assert!((c[0] - 0.2).abs() < 1e-5);
        assert!((c[1] - 0.8).abs() < 1e-5);
        assert!((c[2] - 0.9).abs() < 1e-5);
    }

    #[test]
    fn color_at_full_charge_is_purple() {
        let c = guardian_beam_color(1.0);
        assert!((c[0] - 0.6).abs() < 1e-5);
        assert!((c[1] - 0.1).abs() < 1e-5);
        assert!((c[2] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn color_at_half_charge_is_midpoint() {
        let c = guardian_beam_color(0.5);
        assert!((c[0] - 0.4).abs() < 1e-5);
        assert!((c[1] - 0.45).abs() < 1e-5);
        assert!((c[2] - 0.85).abs() < 1e-5);
    }

    #[test]
    fn color_clamps_negative_charge() {
        let c = guardian_beam_color(-1.0);
        let cyan = guardian_beam_color(0.0);
        assert!((c[0] - cyan[0]).abs() < 1e-5);
        assert!((c[1] - cyan[1]).abs() < 1e-5);
        assert!((c[2] - cyan[2]).abs() < 1e-5);
    }

    #[test]
    fn color_clamps_above_one() {
        let c = guardian_beam_color(2.0);
        let purple = guardian_beam_color(1.0);
        assert!((c[0] - purple[0]).abs() < 1e-5);
        assert!((c[1] - purple[1]).abs() < 1e-5);
        assert!((c[2] - purple[2]).abs() < 1e-5);
    }

    // ── Guardian beam vertex generation ───────────────────────────────

    #[test]
    fn beam_vertices_start_at_origin_and_end_at_target() {
        let beam = GuardianBeam {
            origin: [0.0, 0.0, 0.0],
            target: [4.0, 0.0, 0.0],
            charge_progress: 0.5,
        };
        let verts = generate_guardian_beam(&beam);
        assert!(
            verts.len() >= 2,
            "need at least start and end, got {}",
            verts.len()
        );

        let first = verts.first().expect("non-empty");
        let last = verts.last().expect("non-empty");

        assert!((first[0] - 0.0).abs() < 1e-5);
        assert!((last[0] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn beam_vertex_count_scales_with_length() {
        let short_beam = GuardianBeam {
            origin: [0.0, 0.0, 0.0],
            target: [1.0, 0.0, 0.0],
            charge_progress: 0.0,
        };
        let long_beam = GuardianBeam {
            origin: [0.0, 0.0, 0.0],
            target: [10.0, 0.0, 0.0],
            charge_progress: 0.0,
        };
        let short_count = generate_guardian_beam(&short_beam).len();
        let long_count = generate_guardian_beam(&long_beam).len();
        assert!(
            long_count > short_count,
            "longer beam should have more vertices: short={short_count}, long={long_count}"
        );
    }

    #[test]
    fn zero_length_beam_returns_single_vertex() {
        let beam = GuardianBeam {
            origin: [5.0, 3.0, 1.0],
            target: [5.0, 3.0, 1.0],
            charge_progress: 1.0,
        };
        let verts = generate_guardian_beam(&beam);
        assert_eq!(verts.len(), 1);
        assert!((verts[0][0] - 5.0).abs() < 1e-5);
    }

    // ── Totem particle generation ─────────────────────────────────────

    #[test]
    fn totem_particle_count_matches_requested() {
        let activation = TotemActivation {
            pos: [0.0, 64.0, 0.0],
            time_elapsed: 0.5,
        };
        let particles = totem_particles(&activation, 16);
        assert_eq!(particles.len(), 16);
    }

    #[test]
    fn totem_zero_count_returns_empty() {
        let activation = TotemActivation {
            pos: [0.0, 0.0, 0.0],
            time_elapsed: 1.0,
        };
        let particles = totem_particles(&activation, 0);
        assert!(particles.is_empty());
    }

    #[test]
    fn totem_colors_are_emerald_green() {
        let activation = TotemActivation {
            pos: [0.0, 0.0, 0.0],
            time_elapsed: 0.0,
        };
        let particles = totem_particles(&activation, 8);
        for (_pos, color) in &particles {
            // Green channel should be dominant
            assert!(color[1] > color[0], "green should exceed red");
            assert!(color[1] > color[2], "green should exceed blue");
        }
    }

    #[test]
    fn totem_particles_expand_over_time() {
        let early = TotemActivation {
            pos: [0.0, 0.0, 0.0],
            time_elapsed: 0.0,
        };
        let later = TotemActivation {
            pos: [0.0, 0.0, 0.0],
            time_elapsed: 2.0,
        };
        let early_particles = totem_particles(&early, 4);
        let later_particles = totem_particles(&later, 4);

        // Compute average distance from center in XZ plane
        let avg_dist = |ps: &[([f32; 3], [f32; 3])]| -> f32 {
            let sum: f32 = ps
                .iter()
                .map(|(p, _)| (p[0] * p[0] + p[2] * p[2]).sqrt())
                .sum();
            sum / ps.len() as f32
        };

        assert!(
            avg_dist(&later_particles) > avg_dist(&early_particles),
            "particles should spread out over time"
        );
    }

    // ── Sweep attack arc ──────────────────────────────────────────────

    #[test]
    fn sweep_returns_8_vertices() {
        let verts = sweep_attack_arc([0.0, 0.0, 0.0], 0.0, 2.0);
        assert_eq!(verts.len(), 8);
    }

    #[test]
    fn sweep_vertices_at_correct_radius() {
        let origin = [1.0, 5.0, 3.0];
        let radius = 2.5;
        let verts = sweep_attack_arc(origin, 0.0, radius);

        for v in &verts {
            let dx = v[0] - origin[0];
            let dz = v[2] - origin[2];
            let dist = (dx * dx + dz * dz).sqrt();
            assert!(
                (dist - radius).abs() < 1e-4,
                "vertex should be at radius {radius}, got {dist}"
            );
        }
    }

    #[test]
    fn sweep_vertices_at_origin_y() {
        let origin = [0.0, 10.0, 0.0];
        let verts = sweep_attack_arc(origin, 1.0, 3.0);
        for v in &verts {
            assert!(
                (v[1] - origin[1]).abs() < 1e-5,
                "sweep arc should lie at origin y"
            );
        }
    }

    #[test]
    fn sweep_arc_spans_120_degrees() {
        let origin = [0.0, 0.0, 0.0];
        let verts = sweep_attack_arc(origin, 0.0, 1.0);

        // Compute the angle of first and last vertex relative to origin
        let angle_of = |v: &[f32; 3]| (v[2] - origin[2]).atan2(v[0] - origin[0]);

        let first_angle = angle_of(&verts[0]);
        let last_angle = angle_of(&verts[7]);

        // The angular span should be ~120 degrees (2*PI/3 radians)
        let mut diff = (last_angle - first_angle).abs();
        if diff > std::f32::consts::PI {
            diff = std::f32::consts::TAU - diff;
        }
        let expected = std::f32::consts::FRAC_PI_3 * 2.0;
        assert!(
            (diff - expected).abs() < 0.01,
            "arc should span ~120 degrees ({expected:.4} rad), got {diff:.4} rad"
        );
    }
}
