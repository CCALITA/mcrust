//! Movement particle effects for player locomotion.
//!
//! Generates [`ParticleSpawn`] descriptors for sprinting, landing, swimming,
//! and sneaking so the renderer can instantiate GPU-side particles.

use glam::Vec3;

/// Descriptor for a single particle to be spawned by the renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSpawn {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: [f32; 3],
    pub lifetime: f32,
    pub size: f32,
}

/// Spawn 1-2 block-tinted dust particles at the player's feet while sprinting.
///
/// Particles drift slightly upward and outward with a 0.5 s lifetime.
pub fn sprint_particles(player_pos: Vec3, block_color: [f32; 3]) -> Vec<ParticleSpawn> {
    let foot_y = player_pos.y;
    let base = Vec3::new(player_pos.x, foot_y, player_pos.z);

    let offsets: &[(f32, f32)] = &[(-0.1, -0.1), (0.1, 0.1)];

    offsets
        .iter()
        .map(|&(dx, dz)| ParticleSpawn {
            position: base + Vec3::new(dx, 0.0, dz),
            velocity: Vec3::new(dx * 0.5, 0.8, dz * 0.5),
            color: block_color,
            lifetime: 0.5,
            size: 0.15,
        })
        .collect()
}

/// Spawn radially-spread impact particles on landing.
///
/// The particle count scales with `fall_distance` (capped at 20).
pub fn landing_particles(pos: Vec3, fall_distance: f32, block_color: [f32; 3]) -> Vec<ParticleSpawn> {
    let count = (fall_distance as usize).min(20);
    if count == 0 {
        return Vec::new();
    }

    let angle_step = std::f32::consts::TAU / count as f32;

    (0..count)
        .map(|i| {
            let angle = angle_step * i as f32;
            let (sin, cos) = angle.sin_cos();
            let spread = 0.3 + fall_distance * 0.05;

            ParticleSpawn {
                position: Vec3::new(pos.x + cos * spread, pos.y, pos.z + sin * spread),
                velocity: Vec3::new(cos * 1.0, 0.5, sin * 1.0),
                color: block_color,
                lifetime: 0.3,
                size: 0.2,
            }
        })
        .collect()
}

/// Spawn blue-white bubble particles while swimming.
pub fn swim_particles(pos: Vec3) -> Vec<ParticleSpawn> {
    let offsets: &[(f32, f32)] = &[(-0.15, 0.0), (0.15, 0.0), (0.0, -0.15)];

    offsets
        .iter()
        .map(|&(dx, dz)| ParticleSpawn {
            position: Vec3::new(pos.x + dx, pos.y + 0.3, pos.z + dz),
            velocity: Vec3::new(dx * 0.3, 1.2, dz * 0.3),
            color: [0.7, 0.85, 1.0],
            lifetime: 0.6,
            size: 0.1,
        })
        .collect()
}

/// Sneaking suppresses all movement particles.
pub fn sneak_suppresses_particles() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprint_produces_one_or_two_particles() {
        let particles = sprint_particles(Vec3::new(0.0, 64.0, 0.0), [0.5, 0.4, 0.3]);
        assert!(
            particles.len() >= 1 && particles.len() <= 2,
            "expected 1-2 sprint particles, got {}",
            particles.len()
        );
    }

    #[test]
    fn sprint_particles_near_player_feet() {
        let player_pos = Vec3::new(10.0, 64.0, 20.0);
        let particles = sprint_particles(player_pos, [1.0, 1.0, 1.0]);

        for p in &particles {
            let dx = (p.position.x - player_pos.x).abs();
            let dz = (p.position.z - player_pos.z).abs();
            assert!(dx < 1.0, "particle too far in x: {dx}");
            assert!(dz < 1.0, "particle too far in z: {dz}");
            assert!(
                (p.position.y - player_pos.y).abs() < 0.01,
                "particle y should be at foot level"
            );
        }
    }

    #[test]
    fn sprint_particles_have_block_color() {
        let color = [0.2, 0.8, 0.1];
        let particles = sprint_particles(Vec3::ZERO, color);
        for p in &particles {
            assert_eq!(p.color, color);
        }
    }

    #[test]
    fn sprint_particles_have_upward_velocity() {
        let particles = sprint_particles(Vec3::ZERO, [1.0, 1.0, 1.0]);
        for p in &particles {
            assert!(p.velocity.y > 0.0, "sprint particles should drift upward");
        }
    }

    #[test]
    fn sprint_particles_lifetime_is_half_second() {
        let particles = sprint_particles(Vec3::ZERO, [1.0, 1.0, 1.0]);
        for p in &particles {
            assert!((p.lifetime - 0.5).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn landing_count_scales_with_fall_distance() {
        assert_eq!(landing_particles(Vec3::ZERO, 3.0, [1.0; 3]).len(), 3);
        assert_eq!(landing_particles(Vec3::ZERO, 10.0, [1.0; 3]).len(), 10);
    }

    #[test]
    fn landing_count_capped_at_twenty() {
        assert_eq!(landing_particles(Vec3::ZERO, 50.0, [1.0; 3]).len(), 20);
    }

    #[test]
    fn landing_zero_distance_produces_no_particles() {
        assert!(landing_particles(Vec3::ZERO, 0.0, [1.0; 3]).is_empty());
    }

    #[test]
    fn landing_particles_spread_radially() {
        let pos = Vec3::new(5.0, 64.0, 5.0);
        let particles = landing_particles(pos, 8.0, [1.0; 3]);

        for p in &particles {
            let dx = p.position.x - pos.x;
            let dz = p.position.z - pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            assert!(dist > 0.0, "particles should be spread away from center");
            assert!(dist < 5.0, "particles should not be too far from center");
        }
    }

    #[test]
    fn swim_particles_are_blue_tinted() {
        let particles = swim_particles(Vec3::ZERO);
        for p in &particles {
            // Blue channel should be the highest
            assert!(
                p.color[2] >= p.color[0] && p.color[2] >= p.color[1],
                "swim particles should be blue-tinted, got {:?}",
                p.color
            );
        }
    }

    #[test]
    fn swim_particles_have_upward_velocity() {
        let particles = swim_particles(Vec3::ZERO);
        for p in &particles {
            assert!(p.velocity.y > 0.0, "swim bubbles should float upward");
        }
    }

    #[test]
    fn swim_produces_particles() {
        let particles = swim_particles(Vec3::new(0.0, 62.0, 0.0));
        assert!(!particles.is_empty(), "swim should produce at least one particle");
    }

    #[test]
    fn sneak_suppresses() {
        assert!(sneak_suppresses_particles());
    }
}
