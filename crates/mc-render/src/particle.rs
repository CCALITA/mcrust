//! Particle system for visual effects (block breaking, explosions, flames, etc.).

use glam::Vec3;

/// All supported particle effect types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleType {
    BlockBreak,
    Smoke,
    Flame,
    Heart,
    Explosion,
    Rain,
    Splash,
    Redstone,
    Enchant,
    Portal,
    Crit,
    Dust,
    Bubble,
    Drip,
    Snow,
}

/// A single particle with physics and visual properties.
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub particle_type: ParticleType,
    pub color: [f32; 4],
    pub size: f32,
    pub gravity: f32,
}

impl Particle {
    /// Returns `true` if this particle is still alive.
    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }

    /// Returns the alpha value for rendering. Fades out during the last 20% of life.
    pub fn alpha(&self) -> f32 {
        if self.max_lifetime <= 0.0 {
            return 0.0;
        }
        let ratio = self.lifetime / self.max_lifetime;
        if ratio <= 0.2 {
            // Fade from full to zero over the last 20%
            ratio / 0.2
        } else {
            1.0
        }
    }
}

/// Manages a collection of particles: spawning, updating, and culling.
pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleSystem {
    /// Create an empty particle system.
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    /// Spawn a single particle.
    pub fn spawn(
        &mut self,
        ptype: ParticleType,
        pos: Vec3,
        vel: Vec3,
        lifetime: f32,
        color: [f32; 4],
        size: f32,
    ) {
        self.particles.push(Particle {
            position: pos,
            velocity: vel,
            lifetime,
            max_lifetime: lifetime,
            particle_type: ptype,
            color,
            size,
            gravity: default_gravity(ptype),
        });
    }

    /// Advance all particles by `dt` seconds: integrate motion, apply gravity,
    /// age particles, and remove dead ones.
    pub fn tick(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.velocity.y -= p.gravity * dt;
            p.lifetime -= dt;
        }
        self.particles.retain(Particle::is_alive);
    }

    /// Spawn a block-break effect: 10 particles with random-ish outward velocities.
    pub fn spawn_block_break(&mut self, pos: Vec3, color: [f32; 4]) {
        for i in 0..10 {
            let vel = pseudo_random_direction(pos, i);
            self.spawn(ParticleType::BlockBreak, pos, vel, 0.6, color, 0.15);
        }
    }

    /// Spawn an explosion effect: 30 particles with strong outward velocity and smoke color.
    pub fn spawn_explosion(&mut self, center: Vec3) {
        let smoke_color: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
        for i in 0..30 {
            let vel = pseudo_random_direction(center, i) * 3.0;
            self.spawn(ParticleType::Explosion, center, vel, 1.0, smoke_color, 0.25);
        }
    }

    /// Spawn a single flame particle drifting upward.
    pub fn spawn_flame(&mut self, pos: Vec3) {
        let orange: [f32; 4] = [1.0, 0.6, 0.1, 1.0];
        let vel = Vec3::new(0.0, 1.5, 0.0);
        self.spawn(ParticleType::Flame, pos, vel, 0.8, orange, 0.2);
    }

    /// Spawn a single heart particle floating slowly upward.
    pub fn spawn_heart(&mut self, pos: Vec3) {
        let red: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        let vel = Vec3::new(0.0, 0.5, 0.0);
        self.spawn(ParticleType::Heart, pos, vel, 1.5, red, 0.3);
    }

    /// Number of particles currently alive.
    pub fn count(&self) -> usize {
        self.particles.len()
    }

    /// Slice of all alive particles, suitable for building a render batch.
    pub fn alive_particles(&self) -> &[Particle] {
        &self.particles
    }
}

/// Default gravity per particle type. Zero for particles that should float.
fn default_gravity(ptype: ParticleType) -> f32 {
    match ptype {
        ParticleType::BlockBreak => 9.8,
        ParticleType::Explosion => 4.0,
        ParticleType::Smoke => -0.5, // rises
        ParticleType::Flame => -1.0, // rises
        ParticleType::Heart => 0.0,
        ParticleType::Rain => 12.0,
        ParticleType::Splash => 6.0,
        ParticleType::Redstone => 2.0,
        ParticleType::Enchant => -0.5,
        ParticleType::Portal => -0.3,
        ParticleType::Crit => 5.0,
        ParticleType::Dust => 1.0,
        ParticleType::Bubble => -2.0,
        ParticleType::Drip => 9.8,
        ParticleType::Snow => 1.5,
    }
}

/// Simple deterministic pseudo-random direction from position and index.
///
/// Uses a hash-based approach to avoid pulling in the `rand` crate.
fn pseudo_random_direction(pos: Vec3, index: u32) -> Vec3 {
    let hash = simple_hash(pos, index);

    // Extract three components from the hash bits
    let x_bits = (hash & 0xFF) as f32 / 127.5 - 1.0;
    let y_bits = ((hash >> 8) & 0xFF) as f32 / 255.0; // biased upward [0, 1]
    let z_bits = ((hash >> 16) & 0xFF) as f32 / 127.5 - 1.0;

    let dir = Vec3::new(x_bits, y_bits.max(0.2), z_bits);
    let len = dir.length();
    if len < 1e-6 {
        Vec3::Y
    } else {
        dir / len * 2.0 // speed ~2 m/s
    }
}

/// Fast integer hash mixing position and index for deterministic randomness.
fn simple_hash(pos: Vec3, index: u32) -> u32 {
    let xb = pos.x.to_bits();
    let yb = pos.y.to_bits();
    let zb = pos.z.to_bits();

    let mut h = xb.wrapping_mul(73_856_093)
        ^ yb.wrapping_mul(19_349_663)
        ^ zb.wrapping_mul(83_492_791)
        ^ index.wrapping_mul(49_979_693);

    // Avalanche mix
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_adds_particle() {
        let mut ps = ParticleSystem::new();
        assert_eq!(ps.count(), 0);
        ps.spawn(ParticleType::Dust, Vec3::ZERO, Vec3::Y, 1.0, [1.0; 4], 0.1);
        assert_eq!(ps.count(), 1);
    }

    #[test]
    fn tick_reduces_lifetime() {
        let mut ps = ParticleSystem::new();
        ps.spawn(
            ParticleType::Dust,
            Vec3::ZERO,
            Vec3::ZERO,
            2.0,
            [1.0; 4],
            0.1,
        );
        ps.tick(0.5);
        let p = &ps.alive_particles()[0];
        assert!((p.lifetime - 1.5).abs() < 1e-5);
    }

    #[test]
    fn dead_particles_are_removed() {
        let mut ps = ParticleSystem::new();
        ps.spawn(
            ParticleType::Dust,
            Vec3::ZERO,
            Vec3::ZERO,
            0.5,
            [1.0; 4],
            0.1,
        );
        assert_eq!(ps.count(), 1);
        ps.tick(1.0); // lifetime goes to -0.5 -> dead
        assert_eq!(ps.count(), 0);
    }

    #[test]
    fn block_break_spawns_10_particles() {
        let mut ps = ParticleSystem::new();
        ps.spawn_block_break(Vec3::new(5.0, 10.0, 5.0), [0.6, 0.4, 0.2, 1.0]);
        assert_eq!(ps.count(), 10);
    }

    #[test]
    fn explosion_spawns_30_particles() {
        let mut ps = ParticleSystem::new();
        ps.spawn_explosion(Vec3::new(0.0, 64.0, 0.0));
        assert_eq!(ps.count(), 30);
    }

    #[test]
    fn alpha_is_full_when_plenty_of_life() {
        let p = Particle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 1.0,
            max_lifetime: 1.0,
            particle_type: ParticleType::Dust,
            color: [1.0; 4],
            size: 0.1,
            gravity: 0.0,
        };
        assert!((p.alpha() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn alpha_fades_near_end_of_life() {
        let p = Particle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 0.1, // 10% of max -> within last 20%
            max_lifetime: 1.0,
            particle_type: ParticleType::Dust,
            color: [1.0; 4],
            size: 0.1,
            gravity: 0.0,
        };
        let alpha = p.alpha();
        assert!(alpha < 1.0, "alpha should be less than 1.0, got {alpha}");
        assert!(alpha > 0.0, "alpha should be positive, got {alpha}");
        // 0.1 / 1.0 = 0.1 ratio, 0.1 / 0.2 = 0.5
        assert!((alpha - 0.5).abs() < 1e-5);
    }

    #[test]
    fn alpha_is_zero_when_dead() {
        let p = Particle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
            max_lifetime: 1.0,
            particle_type: ParticleType::Dust,
            color: [1.0; 4],
            size: 0.1,
            gravity: 0.0,
        };
        assert!((p.alpha()).abs() < 1e-5);
    }

    #[test]
    fn flame_spawns_upward() {
        let mut ps = ParticleSystem::new();
        ps.spawn_flame(Vec3::ZERO);
        assert_eq!(ps.count(), 1);
        let p = &ps.alive_particles()[0];
        assert!(p.velocity.y > 0.0);
        assert_eq!(p.particle_type, ParticleType::Flame);
    }

    #[test]
    fn heart_spawns_slowly_upward() {
        let mut ps = ParticleSystem::new();
        ps.spawn_heart(Vec3::ZERO);
        assert_eq!(ps.count(), 1);
        let p = &ps.alive_particles()[0];
        assert!(p.velocity.y > 0.0);
        assert!(p.velocity.y < 1.0); // slow
        assert_eq!(p.particle_type, ParticleType::Heart);
    }

    #[test]
    fn tick_applies_gravity() {
        let mut ps = ParticleSystem::new();
        ps.spawn(
            ParticleType::BlockBreak,
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 5.0, 0.0),
            5.0,
            [1.0; 4],
            0.1,
        );
        let initial_vy = ps.alive_particles()[0].velocity.y;
        ps.tick(1.0);
        let after_vy = ps.alive_particles()[0].velocity.y;
        assert!(
            after_vy < initial_vy,
            "gravity should reduce upward velocity: before={initial_vy}, after={after_vy}"
        );
    }

    #[test]
    fn tick_moves_position() {
        let mut ps = ParticleSystem::new();
        ps.spawn(
            ParticleType::Heart,
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            5.0,
            [1.0; 4],
            0.1,
        );
        ps.tick(1.0);
        let p = &ps.alive_particles()[0];
        assert!(
            p.position.x > 0.9,
            "particle should have moved in x: {}",
            p.position.x
        );
    }

    #[test]
    fn pseudo_random_directions_are_distinct() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let d0 = pseudo_random_direction(pos, 0);
        let d1 = pseudo_random_direction(pos, 1);
        // Directions should differ
        assert!(
            (d0 - d1).length() > 0.01,
            "directions should be distinct: {d0:?} vs {d1:?}"
        );
    }

    #[test]
    fn default_creates_empty_system() {
        let ps = ParticleSystem::default();
        assert_eq!(ps.count(), 0);
    }
}
