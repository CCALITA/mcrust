//! Firework star visual effects: shapes, explosions, and particle parameters.

/// Shape of a firework explosion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FireworkShape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

/// A firework explosion definition with shape, colors, and effects.
#[derive(Debug, Clone, PartialEq)]
pub struct FireworkExplosion {
    pub shape: FireworkShape,
    pub colors: Vec<[f32; 3]>,
    pub fade_colors: Vec<[f32; 3]>,
    pub trail: bool,
    pub twinkle: bool,
}

/// Returns the number of particles for a given firework shape.
pub fn firework_particle_count(shape: FireworkShape) -> u32 {
    match shape {
        FireworkShape::SmallBall => 100,
        FireworkShape::LargeBall => 200,
        FireworkShape::Star => 150,
        FireworkShape::Creeper => 180,
        FireworkShape::Burst => 250,
    }
}

/// Returns the explosion radius for a given firework shape.
pub fn firework_radius(shape: FireworkShape) -> f32 {
    match shape {
        FireworkShape::SmallBall => 2.0,
        FireworkShape::LargeBall => 4.0,
        FireworkShape::Star => 3.0,
        FireworkShape::Creeper => 3.5,
        FireworkShape::Burst => 5.0,
    }
}

/// Returns the duration of a firework explosion in seconds.
pub fn firework_duration() -> f32 {
    1.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_count_varies_by_shape() {
        assert_eq!(firework_particle_count(FireworkShape::SmallBall), 100);
        assert_eq!(firework_particle_count(FireworkShape::LargeBall), 200);
        assert_eq!(firework_particle_count(FireworkShape::Star), 150);
        assert_eq!(firework_particle_count(FireworkShape::Creeper), 180);
        assert_eq!(firework_particle_count(FireworkShape::Burst), 250);
    }

    #[test]
    fn radius_varies_by_shape() {
        assert!(firework_radius(FireworkShape::SmallBall) < firework_radius(FireworkShape::LargeBall));
        assert_eq!(firework_radius(FireworkShape::Burst), 5.0);
    }

    #[test]
    fn duration_is_constant() {
        assert!((firework_duration() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn explosion_struct_creation() {
        let explosion = FireworkExplosion {
            shape: FireworkShape::Star,
            colors: vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            fade_colors: vec![[1.0, 1.0, 1.0]],
            trail: true,
            twinkle: false,
        };
        assert_eq!(explosion.shape, FireworkShape::Star);
        assert_eq!(explosion.colors.len(), 2);
        assert_eq!(explosion.fade_colors.len(), 1);
        assert!(explosion.trail);
        assert!(!explosion.twinkle);
    }

    #[test]
    fn shape_equality() {
        assert_eq!(FireworkShape::Creeper, FireworkShape::Creeper);
        assert_ne!(FireworkShape::SmallBall, FireworkShape::LargeBall);
    }
}
