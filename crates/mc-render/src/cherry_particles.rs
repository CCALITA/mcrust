//! Cherry blossom leaf particle effects for cherry grove biome trees.
//!
//! Particles spawn within a tree's canopy, drift slowly downward with a gentle
//! sideways sway, and expire after roughly 3 seconds.

/// A single cherry blossom leaf particle with position, velocity, rotation,
/// and age tracking.
#[derive(Debug, Clone)]
pub struct CherryLeafParticle {
    pub pos: [f32; 3],
    pub velocity: [f32; 3],
    pub rotation: f32,
    pub age: f32,
    pub lifetime: f32,
}

/// Spawn `count` cherry leaf particles scattered within a tree canopy.
///
/// Particles are placed randomly (deterministic via `seed`) within a sphere of
/// `canopy_radius` centered at `tree_pos`, and given a slow downward drift with
/// gentle sideways sway.
pub fn spawn_cherry_leaves(
    tree_pos: [f32; 3],
    canopy_radius: f32,
    count: u8,
    seed: u64,
) -> Vec<CherryLeafParticle> {
    let mut particles = Vec::with_capacity(count as usize);
    for i in 0..count {
        let h = hash_u64(seed, i as u32);

        // Scatter within canopy sphere
        let x_off = ((h & 0xFFFF) as f32 / 32767.5 - 1.0) * canopy_radius;
        let y_off = (((h >> 16) & 0xFFFF) as f32 / 32767.5 - 1.0) * canopy_radius * 0.5;
        let z_off = (((h >> 32) & 0xFFFF) as f32 / 32767.5 - 1.0) * canopy_radius;

        // Slow downward drift with gentle sideways sway
        let drift_x = ((h >> 48) & 0xFF) as f32 / 255.0 * 0.4 - 0.2;
        let drift_y = -0.3 - ((h >> 56) & 0xFF) as f32 / 255.0 * 0.2;
        let drift_z = (((h >> 40) & 0xFF) as f32 / 255.0) * 0.4 - 0.2;

        // Initial rotation
        let rotation = ((h >> 8) & 0xFFFF) as f32 / 65535.0 * std::f32::consts::TAU;

        // Lifetime varies ~2.5..3.5 seconds
        let lifetime = 2.5 + ((h >> 24) & 0xFF) as f32 / 255.0;

        particles.push(CherryLeafParticle {
            pos: [
                tree_pos[0] + x_off,
                tree_pos[1] + y_off,
                tree_pos[2] + z_off,
            ],
            velocity: [drift_x, drift_y, drift_z],
            rotation,
            age: 0.0,
            lifetime,
        });
    }
    particles
}

/// Advance a cherry leaf particle by `dt` seconds.
///
/// Applies a sinusoidal sway on the X and Z axes and drifts the particle
/// downward. Returns `false` when the particle has exceeded its lifetime
/// (~3 seconds) and should be removed.
pub fn tick_cherry_leaf(p: &mut CherryLeafParticle, dt: f32) -> bool {
    p.age += dt;

    // Sinusoidal sideways sway for a fluttering effect
    let sway = (p.age * 2.0).sin() * 0.3;
    p.pos[0] += (p.velocity[0] + sway) * dt;
    p.pos[1] += p.velocity[1] * dt;
    p.pos[2] += (p.velocity[2] + sway * 0.7) * dt;

    // Slow rotation over time
    p.rotation += dt * 1.5;

    p.age < p.lifetime
}

/// Returns the characteristic pink color of cherry blossom petals as RGB.
pub fn cherry_leaf_color() -> [f32; 3] {
    [1.0, 0.7, 0.8]
}

/// Returns `true` if the particle position is below the tree canopy,
/// indicating the petal has reached ground level.
///
/// `pos` is the particle position and `tree_pos_y` is the Y coordinate of the
/// tree base. `canopy_height` is the vertical extent of the canopy above the
/// tree base.
pub fn cherry_petal_on_ground(pos: [f32; 3], tree_pos_y: f32, canopy_height: f32) -> bool {
    pos[1] < tree_pos_y - canopy_height
}

/// Deterministic 64-bit hash from a seed and particle index.
fn hash_u64(seed: u64, index: u32) -> u64 {
    let mut h = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(index as u64)
        .wrapping_mul(1_442_695_040_888_963_407);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_returns_correct_count() {
        let leaves = spawn_cherry_leaves([0.0, 80.0, 0.0], 3.0, 10, 42);
        assert_eq!(leaves.len(), 10);
    }

    #[test]
    fn spawn_zero_count_returns_empty() {
        let leaves = spawn_cherry_leaves([0.0, 80.0, 0.0], 3.0, 0, 42);
        assert!(leaves.is_empty());
    }

    #[test]
    fn spawn_particles_start_at_age_zero() {
        let leaves = spawn_cherry_leaves([0.0, 80.0, 0.0], 3.0, 5, 99);
        for leaf in &leaves {
            assert!((leaf.age).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn spawn_particles_have_positive_lifetime() {
        let leaves = spawn_cherry_leaves([0.0, 80.0, 0.0], 3.0, 20, 123);
        for leaf in &leaves {
            assert!(
                leaf.lifetime >= 2.5 && leaf.lifetime <= 3.5,
                "lifetime should be in [2.5, 3.5]: {}",
                leaf.lifetime
            );
        }
    }

    #[test]
    fn spawn_particles_within_canopy_radius() {
        let center = [10.0, 80.0, 10.0];
        let radius = 4.0;
        let leaves = spawn_cherry_leaves(center, radius, 50, 77);
        for leaf in &leaves {
            let dx = leaf.pos[0] - center[0];
            let dz = leaf.pos[2] - center[2];
            assert!(
                dx.abs() <= radius,
                "x offset should be within canopy: dx={dx}"
            );
            assert!(
                dz.abs() <= radius,
                "z offset should be within canopy: dz={dz}"
            );
        }
    }

    #[test]
    fn spawn_particles_drift_downward() {
        let leaves = spawn_cherry_leaves([0.0, 80.0, 0.0], 3.0, 20, 55);
        for leaf in &leaves {
            assert!(
                leaf.velocity[1] < 0.0,
                "y velocity should be negative (downward): {}",
                leaf.velocity[1]
            );
        }
    }

    #[test]
    fn spawn_is_deterministic() {
        let a = spawn_cherry_leaves([1.0, 80.0, 3.0], 3.0, 10, 42);
        let b = spawn_cherry_leaves([1.0, 80.0, 3.0], 3.0, 10, 42);
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.pos, pb.pos);
            assert_eq!(pa.velocity, pb.velocity);
            assert_eq!(pa.rotation, pb.rotation);
            assert_eq!(pa.lifetime, pb.lifetime);
        }
    }

    #[test]
    fn spawn_different_seeds_differ() {
        let a = spawn_cherry_leaves([0.0, 80.0, 0.0], 3.0, 5, 1);
        let b = spawn_cherry_leaves([0.0, 80.0, 0.0], 3.0, 5, 2);
        let any_differ = a.iter().zip(b.iter()).any(|(pa, pb)| pa.pos != pb.pos);
        assert!(any_differ, "different seeds should produce different positions");
    }

    #[test]
    fn tick_advances_age() {
        let mut p = CherryLeafParticle {
            pos: [0.0, 80.0, 0.0],
            velocity: [0.1, -0.3, 0.1],
            rotation: 0.0,
            age: 0.0,
            lifetime: 3.0,
        };
        let alive = tick_cherry_leaf(&mut p, 0.5);
        assert!(alive);
        assert!((p.age - 0.5).abs() < 1e-5);
    }

    #[test]
    fn tick_moves_particle_downward() {
        let mut p = CherryLeafParticle {
            pos: [0.0, 80.0, 0.0],
            velocity: [0.0, -0.4, 0.0],
            rotation: 0.0,
            age: 0.0,
            lifetime: 3.0,
        };
        let start_y = p.pos[1];
        tick_cherry_leaf(&mut p, 1.0);
        assert!(
            p.pos[1] < start_y,
            "particle should drift downward: {} -> {}",
            start_y,
            p.pos[1]
        );
    }

    #[test]
    fn tick_rotates_particle() {
        let mut p = CherryLeafParticle {
            pos: [0.0, 80.0, 0.0],
            velocity: [0.0, -0.3, 0.0],
            rotation: 0.0,
            age: 0.0,
            lifetime: 3.0,
        };
        tick_cherry_leaf(&mut p, 1.0);
        assert!(p.rotation > 0.0, "rotation should increase: {}", p.rotation);
    }

    #[test]
    fn tick_returns_false_when_expired() {
        let mut p = CherryLeafParticle {
            pos: [0.0, 80.0, 0.0],
            velocity: [0.0, -0.3, 0.0],
            rotation: 0.0,
            age: 0.0,
            lifetime: 3.0,
        };
        // Advance past lifetime
        let alive = tick_cherry_leaf(&mut p, 3.5);
        assert!(!alive);
    }

    #[test]
    fn tick_alive_within_lifetime() {
        let mut p = CherryLeafParticle {
            pos: [0.0, 80.0, 0.0],
            velocity: [0.0, -0.3, 0.0],
            rotation: 0.0,
            age: 0.0,
            lifetime: 3.0,
        };
        assert!(tick_cherry_leaf(&mut p, 1.0));
        assert!(tick_cherry_leaf(&mut p, 1.0));
        assert!(tick_cherry_leaf(&mut p, 0.5));
    }

    #[test]
    fn cherry_leaf_color_is_pink() {
        let color = cherry_leaf_color();
        assert_eq!(color, [1.0, 0.7, 0.8]);
    }

    #[test]
    fn petal_on_ground_below_canopy() {
        let tree_y = 80.0;
        let canopy_height = 5.0;
        let pos = [0.0, 74.0, 0.0]; // below tree_y - canopy_height = 75.0
        assert!(cherry_petal_on_ground(pos, tree_y, canopy_height));
    }

    #[test]
    fn petal_not_on_ground_above_canopy() {
        let tree_y = 80.0;
        let canopy_height = 5.0;
        let pos = [0.0, 78.0, 0.0]; // above tree_y - canopy_height = 75.0
        assert!(!cherry_petal_on_ground(pos, tree_y, canopy_height));
    }

    #[test]
    fn petal_on_ground_at_boundary() {
        let tree_y = 80.0;
        let canopy_height = 5.0;
        let pos = [0.0, 75.0, 0.0]; // exactly at tree_y - canopy_height
        assert!(!cherry_petal_on_ground(pos, tree_y, canopy_height));
    }

    #[test]
    fn sway_produces_lateral_movement() {
        let mut p = CherryLeafParticle {
            pos: [0.0, 80.0, 0.0],
            velocity: [0.0, -0.3, 0.0],
            rotation: 0.0,
            age: 0.0,
            lifetime: 3.0,
        };
        // Tick several times and check that x and z are not zero
        for _ in 0..20 {
            tick_cherry_leaf(&mut p, 0.1);
        }
        let moved_laterally = p.pos[0].abs() > 0.001 || p.pos[2].abs() > 0.001;
        assert!(
            moved_laterally,
            "sway should cause lateral movement: x={}, z={}",
            p.pos[0],
            p.pos[2]
        );
    }
}
