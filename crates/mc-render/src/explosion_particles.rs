//! Detailed explosion particle effects with smoke trails, flash intensity, and
//! per-particle aging. Complements the simpler `ParticleSystem::spawn_explosion`
//! in [`crate::particle`] with physically-inspired scattering and color transitions.

/// A single explosion particle with position, velocity, and age tracking.
#[derive(Debug, Clone)]
pub struct ExplosionParticle {
    pub pos: [f32; 3],
    pub velocity: [f32; 3],
    pub lifetime: f32,
    pub age: f32,
    pub size: f32,
}

/// Spawn `count` explosion particles radiating outward from `center`.
///
/// `blast_power` scales the initial velocity (typical TNT = 4.0).
/// `seed` feeds a deterministic hash so identical calls produce identical results.
pub fn spawn_explosion(
    center: [f32; 3],
    blast_power: f32,
    count: u32,
    seed: u64,
) -> Vec<ExplosionParticle> {
    let mut particles = Vec::with_capacity(count as usize);
    for i in 0..count {
        let h = hash_u64(seed, i);

        // Map hash bits to a direction in [-1, 1] per axis, biased slightly upward on Y.
        let x = ((h & 0xFFFF) as f32 / 32767.5) - 1.0;
        let y = (((h >> 16) & 0xFFFF) as f32 / 65535.0).max(0.1); // upward bias
        let z = (((h >> 32) & 0xFFFF) as f32 / 32767.5) - 1.0;

        let len = (x * x + y * y + z * z).sqrt().max(1e-6);
        let speed = blast_power * (0.5 + ((h >> 48) & 0xFF) as f32 / 510.0);

        let nx = x / len * speed;
        let ny = y / len * speed;
        let nz = z / len * speed;

        // Lifetime varies slightly per particle (0.6..1.2 seconds scaled by blast_power/4).
        let life_factor = (blast_power / 4.0).clamp(0.5, 3.0);
        let base_life = 0.6 + ((h >> 56) & 0xFF) as f32 / 425.0; // ~0.6..1.2
        let lifetime = base_life * life_factor;

        let size = 0.15 + ((h >> 8) & 0xFF) as f32 / 1700.0; // ~0.15..0.30

        particles.push(ExplosionParticle {
            pos: center,
            velocity: [nx, ny, nz],
            lifetime,
            age: 0.0,
            size,
        });
    }
    particles
}

/// Advance a particle by `dt` seconds. Applies simple gravity and drag.
///
/// Returns `true` if the particle is still alive after the tick, `false` if it
/// has exceeded its lifetime and should be removed.
pub fn tick_explosion_particle(p: &mut ExplosionParticle, dt: f32) -> bool {
    p.age += dt;

    // Drag factor: particles decelerate over time (exponential decay feel).
    let drag = 0.96_f32;

    p.velocity[0] *= drag;
    p.velocity[1] *= drag;
    p.velocity[2] *= drag;

    // Gravity pulls downward.
    p.velocity[1] -= 4.0 * dt;

    p.pos[0] += p.velocity[0] * dt;
    p.pos[1] += p.velocity[1] * dt;
    p.pos[2] += p.velocity[2] * dt;

    p.age < p.lifetime
}

/// Compute the RGBA color of explosion smoke at a given point in its life.
///
/// Transitions from bright orange/yellow near birth to dark gray near death,
/// fading out alpha in the final 30% of the lifetime.
pub fn smoke_color(age: f32, lifetime: f32) -> [f32; 4] {
    if lifetime <= 0.0 {
        return [0.0, 0.0, 0.0, 0.0];
    }

    let t = (age / lifetime).clamp(0.0, 1.0);

    // Lerp from orange (0.9, 0.6, 0.2) to dark gray (0.2, 0.2, 0.2).
    let r = 0.9 - 0.7 * t;
    let g = 0.6 - 0.4 * t;
    let b = 0.2;

    // Alpha: full until 70%, then linear fade to zero.
    let a = if t < 0.7 { 1.0 } else { 1.0 - (t - 0.7) / 0.3 };

    [r, g, b, a]
}

/// Compute the visual flash intensity at `distance` blocks from an explosion
/// with the given `blast_power`.
///
/// Returns a value in `[0.0, 1.0]`. The flash follows an inverse-square falloff
/// with a small bias to avoid division by zero, clamped to 1.0 at the epicenter.
pub fn flash_intensity(distance: f32, blast_power: f32) -> f32 {
    if blast_power <= 0.0 {
        return 0.0;
    }
    // Effective radius scales with blast_power.
    let radius = blast_power * 2.0;
    let intensity = radius * radius / (distance * distance + 1.0);
    intensity.clamp(0.0, 1.0)
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
        let particles = spawn_explosion([0.0, 64.0, 0.0], 4.0, 50, 42);
        assert_eq!(particles.len(), 50);
    }

    #[test]
    fn spawn_zero_count_returns_empty() {
        let particles = spawn_explosion([0.0, 0.0, 0.0], 4.0, 0, 1);
        assert!(particles.is_empty());
    }

    #[test]
    fn spawn_particles_start_at_center() {
        let center = [10.0, 20.0, 30.0];
        let particles = spawn_explosion(center, 4.0, 5, 99);
        for p in &particles {
            assert_eq!(p.pos, center);
        }
    }

    #[test]
    fn spawn_particles_start_with_zero_age() {
        let particles = spawn_explosion([0.0, 0.0, 0.0], 4.0, 10, 7);
        for p in &particles {
            assert!((p.age).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn spawn_particles_have_positive_lifetime() {
        let particles = spawn_explosion([0.0, 0.0, 0.0], 4.0, 20, 123);
        for p in &particles {
            assert!(
                p.lifetime > 0.0,
                "lifetime should be positive: {}",
                p.lifetime
            );
        }
    }

    #[test]
    fn spawn_particles_have_nonzero_velocity() {
        let particles = spawn_explosion([0.0, 0.0, 0.0], 4.0, 20, 55);
        for p in &particles {
            let speed_sq = p.velocity[0] * p.velocity[0]
                + p.velocity[1] * p.velocity[1]
                + p.velocity[2] * p.velocity[2];
            assert!(speed_sq > 0.01, "velocity should be nonzero");
        }
    }

    #[test]
    fn spawn_is_deterministic() {
        let a = spawn_explosion([1.0, 2.0, 3.0], 4.0, 10, 42);
        let b = spawn_explosion([1.0, 2.0, 3.0], 4.0, 10, 42);
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.velocity, pb.velocity);
            assert_eq!(pa.lifetime, pb.lifetime);
            assert_eq!(pa.size, pb.size);
        }
    }

    #[test]
    fn spawn_different_seeds_differ() {
        let a = spawn_explosion([0.0, 0.0, 0.0], 4.0, 5, 1);
        let b = spawn_explosion([0.0, 0.0, 0.0], 4.0, 5, 2);
        let any_differ = a
            .iter()
            .zip(b.iter())
            .any(|(pa, pb)| pa.velocity != pb.velocity);
        assert!(
            any_differ,
            "different seeds should produce different velocities"
        );
    }

    #[test]
    fn tick_advances_age() {
        let mut p = ExplosionParticle {
            pos: [0.0, 10.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            lifetime: 2.0,
            age: 0.0,
            size: 0.2,
        };
        let alive = tick_explosion_particle(&mut p, 0.5);
        assert!(alive);
        assert!((p.age - 0.5).abs() < 1e-5);
    }

    #[test]
    fn tick_moves_position() {
        let mut p = ExplosionParticle {
            pos: [0.0, 0.0, 0.0],
            velocity: [10.0, 0.0, 0.0],
            lifetime: 5.0,
            age: 0.0,
            size: 0.2,
        };
        tick_explosion_particle(&mut p, 1.0);
        assert!(p.pos[0] > 5.0, "particle should move in x: {}", p.pos[0]);
    }

    #[test]
    fn tick_applies_gravity() {
        let mut p = ExplosionParticle {
            pos: [0.0, 100.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            lifetime: 5.0,
            age: 0.0,
            size: 0.2,
        };
        tick_explosion_particle(&mut p, 1.0);
        assert!(p.velocity[1] < 0.0, "gravity should pull velocity down");
    }

    #[test]
    fn tick_returns_false_when_expired() {
        let mut p = ExplosionParticle {
            pos: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            lifetime: 0.5,
            age: 0.0,
            size: 0.2,
        };
        // First tick: 0.3s elapsed, still alive
        assert!(tick_explosion_particle(&mut p, 0.3));
        // Second tick: 0.6s elapsed, past 0.5 lifetime
        assert!(!tick_explosion_particle(&mut p, 0.3));
    }

    #[test]
    fn smoke_color_starts_orange() {
        let c = smoke_color(0.0, 1.0);
        assert!(
            (c[0] - 0.9).abs() < 1e-5,
            "red channel should be 0.9: {}",
            c[0]
        );
        assert!(
            (c[1] - 0.6).abs() < 1e-5,
            "green channel should be 0.6: {}",
            c[1]
        );
        assert!(
            (c[2] - 0.2).abs() < 1e-5,
            "blue channel should be 0.2: {}",
            c[2]
        );
        assert!((c[3] - 1.0).abs() < 1e-5, "alpha should be 1.0: {}", c[3]);
    }

    #[test]
    fn smoke_color_ends_dark_gray_faded() {
        let c = smoke_color(1.0, 1.0);
        assert!((c[0] - 0.2).abs() < 1e-5, "red at end: {}", c[0]);
        assert!((c[1] - 0.2).abs() < 1e-5, "green at end: {}", c[1]);
        assert!((c[2] - 0.2).abs() < 1e-5, "blue at end: {}", c[2]);
        assert!(c[3] < 0.01, "alpha at end should be ~0: {}", c[3]);
    }

    #[test]
    fn smoke_color_alpha_full_at_midpoint() {
        let c = smoke_color(0.5, 1.0);
        assert!(
            (c[3] - 1.0).abs() < 1e-5,
            "alpha at midpoint should be 1.0: {}",
            c[3]
        );
    }

    #[test]
    fn smoke_color_alpha_fades_in_last_30_percent() {
        // At t=0.85 (85% through), alpha should be about 0.5
        let c = smoke_color(0.85, 1.0);
        assert!(c[3] < 1.0, "alpha should have started fading: {}", c[3]);
        assert!(c[3] > 0.0, "alpha should still be positive: {}", c[3]);
        assert!(
            (c[3] - 0.5).abs() < 0.05,
            "alpha at 85% should be ~0.5: {}",
            c[3]
        );
    }

    #[test]
    fn smoke_color_zero_lifetime_returns_transparent() {
        let c = smoke_color(0.5, 0.0);
        assert_eq!(c, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn flash_intensity_at_epicenter_is_max() {
        let i = flash_intensity(0.0, 4.0);
        assert!((i - 1.0).abs() < 1e-5, "intensity at center: {i}");
    }

    #[test]
    fn flash_intensity_decreases_with_distance() {
        let near = flash_intensity(2.0, 4.0);
        let far = flash_intensity(10.0, 4.0);
        assert!(
            near > far,
            "near ({near}) should be greater than far ({far})"
        );
    }

    #[test]
    fn flash_intensity_scales_with_blast_power() {
        let weak = flash_intensity(5.0, 2.0);
        let strong = flash_intensity(5.0, 8.0);
        assert!(
            strong > weak,
            "stronger blast ({strong}) should be brighter than weaker ({weak})"
        );
    }

    #[test]
    fn flash_intensity_zero_blast_is_zero() {
        let i = flash_intensity(5.0, 0.0);
        assert!(
            (i).abs() < 1e-5,
            "zero blast should give zero intensity: {i}"
        );
    }

    #[test]
    fn flash_intensity_never_exceeds_one() {
        let i = flash_intensity(0.1, 100.0);
        assert!(i <= 1.0, "intensity should be clamped to 1.0: {i}");
    }

    #[test]
    fn blast_power_affects_velocity_magnitude() {
        let weak = spawn_explosion([0.0, 0.0, 0.0], 1.0, 10, 42);
        let strong = spawn_explosion([0.0, 0.0, 0.0], 8.0, 10, 42);

        let avg_speed = |particles: &[ExplosionParticle]| -> f32 {
            let total: f32 = particles
                .iter()
                .map(|p| {
                    (p.velocity[0] * p.velocity[0]
                        + p.velocity[1] * p.velocity[1]
                        + p.velocity[2] * p.velocity[2])
                        .sqrt()
                })
                .sum();
            total / particles.len() as f32
        };

        assert!(
            avg_speed(&strong) > avg_speed(&weak),
            "stronger blast should produce faster particles"
        );
    }

    #[test]
    fn particle_size_is_in_expected_range() {
        let particles = spawn_explosion([0.0, 0.0, 0.0], 4.0, 100, 77);
        for p in &particles {
            assert!(p.size >= 0.15, "size too small: {}", p.size);
            assert!(p.size <= 0.35, "size too large: {}", p.size);
        }
    }
}
