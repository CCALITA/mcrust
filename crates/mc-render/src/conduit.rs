//! Conduit beam visuals: activation state, beam quads, aura particles.
//!
//! A conduit activates when surrounded by at least 16 prismarine blocks in its
//! activation frame and grants an effect range of 16-96 blocks depending on
//! the prismarine count.

/// Minimum prismarine blocks required for a conduit to activate.
pub const CONDUIT_ACTIVATION_THRESHOLD: u8 = 16;

/// Maximum effect range of a fully powered conduit, in blocks.
pub const CONDUIT_MAX_RANGE: u8 = 96;

/// Radius (in blocks) at which aura particles orbit the conduit.
pub const CONDUIT_AURA_RADIUS: f32 = 1.5;

/// Number of aura particles spawned per tick around an active conduit.
pub const CONDUIT_AURA_PARTICLE_COUNT: usize = 4;

/// Number of beam quads (each contributing 4 vertices) drawn for the attack beam.
pub const CONDUIT_BEAM_QUAD_COUNT: usize = 4;

/// Half-width of the beam in blocks.
const BEAM_HALF_WIDTH: f32 = 0.1;

/// Runtime state for a single conduit block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConduitState {
    pub active: bool,
    pub attack_target: Option<[f32; 3]>,
    pub prismarine_count: u8,
}

impl ConduitState {
    /// Builds a `ConduitState` from the surrounding prismarine count.
    pub fn from_prismarine_count(prismarine_count: u8) -> Self {
        Self {
            active: conduit_active(prismarine_count),
            attack_target: None,
            prismarine_count,
        }
    }
}

/// Returns true when a conduit has enough prismarine blocks to activate.
pub fn conduit_active(prismarine_count: u8) -> bool {
    prismarine_count >= CONDUIT_ACTIVATION_THRESHOLD
}

/// Returns the effect range in blocks for the given prismarine count.
///
/// - Returns `0` when below the activation threshold.
/// - Otherwise returns `(prismarine_count / 7) * 16`, clamped to [`CONDUIT_MAX_RANGE`].
pub fn conduit_range(prismarine_count: u8) -> u8 {
    if !conduit_active(prismarine_count) {
        return 0;
    }
    let steps = (prismarine_count / 7) as u32;
    let range = steps.saturating_mul(16);
    range.min(CONDUIT_MAX_RANGE as u32) as u8
}

/// Returns the conduit's signature teal beam color.
pub fn conduit_beam_color() -> [f32; 3] {
    [0.2, 0.85, 0.85]
}

/// Returns a smooth pulsation factor in `[0.7, 1.0]` for the beam intensity.
pub fn conduit_pulse(time: f32) -> f32 {
    let s = time.sin();
    // Map sin in [-1, 1] to [0.7, 1.0]
    0.85 + s * 0.15
}

/// Returns 16 vertices forming 4 quads for the conduit's attack beam.
///
/// The beam is built as a square cross-section tube between `conduit_pos`
/// and `target_pos`. Vertices are emitted as 4 quads (top, bottom, left, right
/// faces of the tube). Each quad contributes 4 vertices, totaling 16.
pub fn conduit_beam_vertices(conduit_pos: [f32; 3], target_pos: [f32; 3]) -> Vec<[f32; 3]> {
    let h = BEAM_HALF_WIDTH;
    let [sx, sy, sz] = conduit_pos;
    let [ex, ey, ez] = target_pos;

    let mut verts = Vec::with_capacity(CONDUIT_BEAM_QUAD_COUNT * 4);

    // Top face (+Y)
    verts.push([sx - h, sy + h, sz - h]);
    verts.push([sx + h, sy + h, sz - h]);
    verts.push([ex + h, ey + h, ez - h]);
    verts.push([ex - h, ey + h, ez - h]);

    // Bottom face (-Y)
    verts.push([sx - h, sy - h, sz + h]);
    verts.push([sx + h, sy - h, sz + h]);
    verts.push([ex + h, ey - h, ez + h]);
    verts.push([ex - h, ey - h, ez + h]);

    // Left face (-X)
    verts.push([sx - h, sy - h, sz - h]);
    verts.push([sx - h, sy + h, sz - h]);
    verts.push([ex - h, ey + h, ez - h]);
    verts.push([ex - h, ey - h, ez - h]);

    // Right face (+X)
    verts.push([sx + h, sy - h, sz + h]);
    verts.push([sx + h, sy + h, sz + h]);
    verts.push([ex + h, ey + h, ez + h]);
    verts.push([ex + h, ey - h, ez + h]);

    verts
}

/// Returns 4 cyan aura particles orbiting the conduit at radius
/// [`CONDUIT_AURA_RADIUS`].
///
/// Each entry is `(position, color)` — the color is the conduit's teal hue.
pub fn conduit_aura_particles(time: f32, conduit_pos: [f32; 3]) -> Vec<([f32; 3], [f32; 3])> {
    let color = conduit_beam_color();
    let [cx, cy, cz] = conduit_pos;
    let mut out = Vec::with_capacity(CONDUIT_AURA_PARTICLE_COUNT);

    for i in 0..CONDUIT_AURA_PARTICLE_COUNT {
        let phase = (i as f32) * (std::f32::consts::TAU / CONDUIT_AURA_PARTICLE_COUNT as f32);
        let angle = time + phase;
        let x = cx + CONDUIT_AURA_RADIUS * angle.cos();
        let z = cz + CONDUIT_AURA_RADIUS * angle.sin();
        // Small vertical bob so the aura doesn't lie in a single plane
        let y = cy + (angle * 0.5).sin() * 0.25;
        out.push(([x, y, z], color));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_threshold_is_16_prismarine() {
        assert!(!conduit_active(0));
        assert!(!conduit_active(15));
        assert!(conduit_active(16));
        assert!(conduit_active(42));
    }

    #[test]
    fn range_is_zero_below_threshold() {
        assert_eq!(conduit_range(0), 0);
        assert_eq!(conduit_range(15), 0);
    }

    #[test]
    fn range_steps_with_prismarine_count() {
        // 16/7 = 2 -> 32 blocks
        assert_eq!(conduit_range(16), 32);
        // 21/7 = 3 -> 48 blocks
        assert_eq!(conduit_range(21), 48);
        // 42/7 = 6 -> 96 blocks (max)
        assert_eq!(conduit_range(42), 96);
    }

    #[test]
    fn range_is_clamped_to_max() {
        assert_eq!(conduit_range(255), CONDUIT_MAX_RANGE);
        assert_eq!(conduit_range(100), CONDUIT_MAX_RANGE);
    }

    #[test]
    fn beam_has_sixteen_vertices() {
        let verts = conduit_beam_vertices([0.0, 0.0, 0.0], [0.0, 10.0, 0.0]);
        assert_eq!(verts.len(), CONDUIT_BEAM_QUAD_COUNT * 4);
        assert_eq!(verts.len(), 16);
    }

    #[test]
    fn aura_returns_four_particles() {
        let particles = conduit_aura_particles(0.0, [0.0, 64.0, 0.0]);
        assert_eq!(particles.len(), CONDUIT_AURA_PARTICLE_COUNT);
        assert_eq!(particles.len(), 4);
    }

    #[test]
    fn aura_particles_orbit_at_expected_radius() {
        let center = [10.0, 64.0, -5.0];
        for (pos, _color) in conduit_aura_particles(1.23, center) {
            let dx = pos[0] - center[0];
            let dz = pos[2] - center[2];
            let r = (dx * dx + dz * dz).sqrt();
            assert!((r - CONDUIT_AURA_RADIUS).abs() < 1e-4, "radius was {r}");
        }
    }

    #[test]
    fn aura_color_matches_beam_color() {
        let particles = conduit_aura_particles(0.0, [0.0, 0.0, 0.0]);
        let beam = conduit_beam_color();
        for (_, color) in particles {
            assert_eq!(color, beam);
        }
    }

    #[test]
    fn beam_color_is_teal() {
        let c = conduit_beam_color();
        assert_eq!(c, [0.2, 0.85, 0.85]);
    }

    #[test]
    fn pulse_stays_within_expected_band() {
        // Sample a wide range of time values and verify range invariants.
        for i in 0..1000 {
            let t = i as f32 * 0.0173;
            let p = conduit_pulse(t);
            assert!(p >= 0.7 - 1e-5, "pulse {p} below 0.7 at t={t}");
            assert!(p <= 1.0 + 1e-5, "pulse {p} above 1.0 at t={t}");
        }
    }

    #[test]
    fn pulse_hits_endpoints_at_sin_extremes() {
        // sin = -1 at 3π/2 -> pulse = 0.7; sin = 1 at π/2 -> pulse = 1.0
        let low = conduit_pulse(3.0 * std::f32::consts::FRAC_PI_2);
        let high = conduit_pulse(std::f32::consts::FRAC_PI_2);
        assert!((low - 0.7).abs() < 1e-5);
        assert!((high - 1.0).abs() < 1e-5);
    }

    #[test]
    fn state_from_prismarine_count_is_consistent() {
        let inactive = ConduitState::from_prismarine_count(10);
        assert!(!inactive.active);
        assert_eq!(inactive.prismarine_count, 10);
        assert!(inactive.attack_target.is_none());

        let active = ConduitState::from_prismarine_count(16);
        assert!(active.active);
        assert_eq!(active.prismarine_count, 16);
    }
}
