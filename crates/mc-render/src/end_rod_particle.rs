//! End rod particle effects — floating white-pink particles with soft glow.

/// A single end rod particle with position, velocity, and age.
pub struct EndRodParticle {
    pub pos: [f32; 3],
    pub velocity: [f32; 3],
    pub age: f32,
}

/// Returns the end rod particle color (white-pink).
pub fn end_rod_color() -> [f32; 3] {
    [1.0, 0.95, 0.98]
}

/// Returns the light level emitted by end rods.
pub fn end_rod_light_level() -> u8 {
    14
}

/// Spawns `count` end rod particles around the given rod position.
///
/// Uses a simple deterministic seed-based approach for reproducible spawns.
pub fn spawn_end_rod_particles(rod_pos: [f32; 3], count: u8, seed: u64) -> Vec<EndRodParticle> {
    let mut particles = Vec::with_capacity(count as usize);
    for i in 0..count {
        let hash = seed.wrapping_mul(6364136223846793005).wrapping_add(i as u64);
        let fx = ((hash >> 0) & 0xFFFF) as f32 / 65535.0 - 0.5;
        let fy = ((hash >> 16) & 0xFFFF) as f32 / 65535.0 - 0.5;
        let fz = ((hash >> 32) & 0xFFFF) as f32 / 65535.0 - 0.5;

        particles.push(EndRodParticle {
            pos: [
                rod_pos[0] + fx * 0.5,
                rod_pos[1] + fy * 0.5,
                rod_pos[2] + fz * 0.5,
            ],
            velocity: [fx * 0.02, fy.abs() * 0.03 + 0.01, fz * 0.02],
            age: 0.0,
        });
    }
    particles
}

/// Ticks a single end rod particle forward by `dt` seconds.
///
/// Returns `true` if the particle is still alive, `false` if it should be removed.
const MAX_AGE: f32 = 2.0;

pub fn tick_end_rod_particle(p: &mut EndRodParticle, dt: f32) -> bool {
    p.pos[0] += p.velocity[0] * dt;
    p.pos[1] += p.velocity[1] * dt;
    p.pos[2] += p.velocity[2] * dt;
    p.age += dt;
    p.age < MAX_AGE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_is_white_pink() {
        let c = end_rod_color();
        assert_eq!(c, [1.0, 0.95, 0.98]);
    }

    #[test]
    fn light_level_is_14() {
        assert_eq!(end_rod_light_level(), 14);
    }

    #[test]
    fn spawns_correct_count() {
        let particles = spawn_end_rod_particles([0.0, 0.0, 0.0], 5, 42);
        assert_eq!(particles.len(), 5);
    }

    #[test]
    fn spawned_particles_start_at_age_zero() {
        let particles = spawn_end_rod_particles([1.0, 2.0, 3.0], 3, 99);
        for p in &particles {
            assert_eq!(p.age, 0.0);
        }
    }

    #[test]
    fn spawn_is_deterministic() {
        let a = spawn_end_rod_particles([0.0, 0.0, 0.0], 4, 123);
        let b = spawn_end_rod_particles([0.0, 0.0, 0.0], 4, 123);
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.pos, pb.pos);
            assert_eq!(pa.velocity, pb.velocity);
        }
    }

    #[test]
    fn tick_advances_position_and_age() {
        let mut p = EndRodParticle {
            pos: [0.0, 0.0, 0.0],
            velocity: [1.0, 2.0, 3.0],
            age: 0.0,
        };
        let alive = tick_end_rod_particle(&mut p, 0.5);
        assert!(alive);
        assert_eq!(p.pos, [0.5, 1.0, 1.5]);
        assert_eq!(p.age, 0.5);
    }

    #[test]
    fn tick_returns_false_when_expired() {
        let mut p = EndRodParticle {
            pos: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            age: 1.9,
        };
        let alive = tick_end_rod_particle(&mut p, 0.2);
        assert!(!alive);
    }

    #[test]
    fn spawn_zero_particles() {
        let particles = spawn_end_rod_particles([0.0, 0.0, 0.0], 0, 1);
        assert!(particles.is_empty());
    }
}
