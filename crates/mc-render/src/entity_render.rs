use glam::Vec3;

/// A single box-shaped part of a mob model, defined in Minecraft's 1/16-block
/// pixel coordinate system.
#[derive(Debug, Clone)]
pub struct MobModelPart {
    pub name: &'static str,
    /// Position offset from the model origin (in 1/16 block units).
    pub offset: Vec3,
    /// Width (x), height (y), depth (z) of the box (in 1/16 block units).
    pub size: Vec3,
    /// Pivot point for rotation (in 1/16 block units).
    pub pivot: Vec3,
    /// Current rotation in radians around each axis.
    pub rotation: Vec3,
    /// UV texture offset (u, v) in the texture atlas.
    pub tex_offset: (u16, u16),
}

/// A complete mob model composed of named box parts.
#[derive(Debug, Clone)]
pub struct MobModel {
    pub parts: Vec<MobModelPart>,
}

/// Per-entity data needed by the renderer each frame.
#[derive(Debug, Clone, Copy)]
pub struct EntityRenderData {
    pub position: Vec3,
    pub yaw: f32,
    pub model_type: u8,
    pub animation_time: f32,
}

// ---------------------------------------------------------------------------
// Model definitions — Minecraft-accurate proportions (1/16 block pixel units)
// ---------------------------------------------------------------------------

/// Zombie: head, body, left_arm, right_arm, left_leg, right_leg (6 parts).
pub fn zombie_model() -> MobModel {
    MobModel {
        parts: vec![
            MobModelPart {
                name: "head",
                offset: Vec3::new(-4.0, 24.0, -4.0),
                size: Vec3::new(8.0, 8.0, 8.0),
                pivot: Vec3::new(0.0, 24.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            MobModelPart {
                name: "body",
                offset: Vec3::new(-4.0, 12.0, -2.0),
                size: Vec3::new(8.0, 12.0, 4.0),
                pivot: Vec3::new(0.0, 12.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (16, 16),
            },
            MobModelPart {
                name: "left_arm",
                offset: Vec3::new(4.0, 12.0, -2.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(6.0, 24.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (40, 16),
            },
            MobModelPart {
                name: "right_arm",
                offset: Vec3::new(-8.0, 12.0, -2.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(-6.0, 24.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (40, 16),
            },
            MobModelPart {
                name: "left_leg",
                offset: Vec3::new(0.0, 0.0, -2.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(2.0, 12.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "right_leg",
                offset: Vec3::new(-4.0, 0.0, -2.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(-2.0, 12.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
        ],
    }
}

/// Skeleton: same layout as zombie but thinner arms (2x12x2) (6 parts).
pub fn skeleton_model() -> MobModel {
    MobModel {
        parts: vec![
            MobModelPart {
                name: "head",
                offset: Vec3::new(-4.0, 24.0, -4.0),
                size: Vec3::new(8.0, 8.0, 8.0),
                pivot: Vec3::new(0.0, 24.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            MobModelPart {
                name: "body",
                offset: Vec3::new(-4.0, 12.0, -2.0),
                size: Vec3::new(8.0, 12.0, 4.0),
                pivot: Vec3::new(0.0, 12.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (16, 16),
            },
            MobModelPart {
                name: "left_arm",
                offset: Vec3::new(4.0, 12.0, -1.0),
                size: Vec3::new(2.0, 12.0, 2.0),
                pivot: Vec3::new(5.0, 24.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (40, 16),
            },
            MobModelPart {
                name: "right_arm",
                offset: Vec3::new(-6.0, 12.0, -1.0),
                size: Vec3::new(2.0, 12.0, 2.0),
                pivot: Vec3::new(-5.0, 24.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (40, 16),
            },
            MobModelPart {
                name: "left_leg",
                offset: Vec3::new(0.0, 0.0, -2.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(2.0, 12.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "right_leg",
                offset: Vec3::new(-4.0, 0.0, -2.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(-2.0, 12.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
        ],
    }
}

/// Creeper: head, body, 4 short legs (5 parts).
pub fn creeper_model() -> MobModel {
    MobModel {
        parts: vec![
            MobModelPart {
                name: "head",
                offset: Vec3::new(-4.0, 18.0, -4.0),
                size: Vec3::new(8.0, 8.0, 8.0),
                pivot: Vec3::new(0.0, 18.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            MobModelPart {
                name: "body",
                offset: Vec3::new(-4.0, 6.0, -2.0),
                size: Vec3::new(8.0, 12.0, 4.0),
                pivot: Vec3::new(0.0, 6.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (16, 16),
            },
            MobModelPart {
                name: "front_left_leg",
                offset: Vec3::new(0.0, 0.0, -4.0),
                size: Vec3::new(4.0, 6.0, 4.0),
                pivot: Vec3::new(2.0, 6.0, -2.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "front_right_leg",
                offset: Vec3::new(-4.0, 0.0, -4.0),
                size: Vec3::new(4.0, 6.0, 4.0),
                pivot: Vec3::new(-2.0, 6.0, -2.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "back_left_leg",
                offset: Vec3::new(0.0, 0.0, 0.0),
                size: Vec3::new(4.0, 6.0, 4.0),
                pivot: Vec3::new(2.0, 6.0, 2.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
        ],
    }
}

/// Spider: head, body, 8 legs (10 parts).
pub fn spider_model() -> MobModel {
    let leg_size = Vec3::new(2.0, 8.0, 2.0);
    let leg_angle = 0.6; // ~34 degrees outward splay

    MobModel {
        parts: vec![
            // Head
            MobModelPart {
                name: "head",
                offset: Vec3::new(-4.0, 8.0, -11.0),
                size: Vec3::new(8.0, 8.0, 6.0),
                pivot: Vec3::new(0.0, 11.0, -8.0),
                rotation: Vec3::ZERO,
                tex_offset: (32, 4),
            },
            // Body (abdomen)
            MobModelPart {
                name: "body",
                offset: Vec3::new(-6.0, 5.0, -3.0),
                size: Vec3::new(12.0, 8.0, 10.0),
                pivot: Vec3::new(0.0, 9.0, 2.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            // Right legs (4), front to back
            MobModelPart {
                name: "right_leg_1",
                offset: Vec3::new(-7.0, 1.0, -5.0),
                size: leg_size,
                pivot: Vec3::new(-6.0, 9.0, -4.0),
                rotation: Vec3::new(0.0, 0.0, -leg_angle),
                tex_offset: (18, 0),
            },
            MobModelPart {
                name: "right_leg_2",
                offset: Vec3::new(-7.0, 1.0, -2.0),
                size: leg_size,
                pivot: Vec3::new(-6.0, 9.0, -1.0),
                rotation: Vec3::new(0.0, 0.0, -leg_angle),
                tex_offset: (18, 0),
            },
            MobModelPart {
                name: "right_leg_3",
                offset: Vec3::new(-7.0, 1.0, 1.0),
                size: leg_size,
                pivot: Vec3::new(-6.0, 9.0, 2.0),
                rotation: Vec3::new(0.0, 0.0, -leg_angle),
                tex_offset: (18, 0),
            },
            MobModelPart {
                name: "right_leg_4",
                offset: Vec3::new(-7.0, 1.0, 4.0),
                size: leg_size,
                pivot: Vec3::new(-6.0, 9.0, 5.0),
                rotation: Vec3::new(0.0, 0.0, -leg_angle),
                tex_offset: (18, 0),
            },
            // Left legs (4), front to back
            MobModelPart {
                name: "left_leg_1",
                offset: Vec3::new(5.0, 1.0, -5.0),
                size: leg_size,
                pivot: Vec3::new(6.0, 9.0, -4.0),
                rotation: Vec3::new(0.0, 0.0, leg_angle),
                tex_offset: (18, 0),
            },
            MobModelPart {
                name: "left_leg_2",
                offset: Vec3::new(5.0, 1.0, -2.0),
                size: leg_size,
                pivot: Vec3::new(6.0, 9.0, -1.0),
                rotation: Vec3::new(0.0, 0.0, leg_angle),
                tex_offset: (18, 0),
            },
            MobModelPart {
                name: "left_leg_3",
                offset: Vec3::new(5.0, 1.0, 1.0),
                size: leg_size,
                pivot: Vec3::new(6.0, 9.0, 2.0),
                rotation: Vec3::new(0.0, 0.0, leg_angle),
                tex_offset: (18, 0),
            },
            MobModelPart {
                name: "left_leg_4",
                offset: Vec3::new(5.0, 1.0, 4.0),
                size: leg_size,
                pivot: Vec3::new(6.0, 9.0, 5.0),
                rotation: Vec3::new(0.0, 0.0, leg_angle),
                tex_offset: (18, 0),
            },
        ],
    }
}

/// Pig: head, body (horizontal), 4 short legs (5 parts).
pub fn pig_model() -> MobModel {
    MobModel {
        parts: vec![
            MobModelPart {
                name: "head",
                offset: Vec3::new(-4.0, 8.0, -11.0),
                size: Vec3::new(8.0, 8.0, 8.0),
                pivot: Vec3::new(0.0, 12.0, -7.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            MobModelPart {
                name: "body",
                offset: Vec3::new(-7.0, 6.0, -4.0),
                size: Vec3::new(14.0, 8.0, 8.0),
                pivot: Vec3::new(0.0, 10.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (28, 8),
            },
            MobModelPart {
                name: "front_left_leg",
                offset: Vec3::new(1.0, 0.0, -5.0),
                size: Vec3::new(4.0, 6.0, 4.0),
                pivot: Vec3::new(3.0, 6.0, -3.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "front_right_leg",
                offset: Vec3::new(-5.0, 0.0, -5.0),
                size: Vec3::new(4.0, 6.0, 4.0),
                pivot: Vec3::new(-3.0, 6.0, -3.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "back_left_leg",
                offset: Vec3::new(1.0, 0.0, 3.0),
                size: Vec3::new(4.0, 6.0, 4.0),
                pivot: Vec3::new(3.0, 6.0, 5.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
        ],
    }
}

/// Cow: head (with horns implied by wider box), body, 4 legs (5 parts).
pub fn cow_model() -> MobModel {
    MobModel {
        parts: vec![
            MobModelPart {
                name: "head",
                offset: Vec3::new(-4.0, 16.0, -10.0),
                size: Vec3::new(8.0, 8.0, 6.0),
                pivot: Vec3::new(0.0, 20.0, -7.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            MobModelPart {
                name: "body",
                offset: Vec3::new(-7.0, 6.0, -4.0),
                size: Vec3::new(14.0, 10.0, 8.0),
                pivot: Vec3::new(0.0, 11.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (18, 4),
            },
            MobModelPart {
                name: "front_left_leg",
                offset: Vec3::new(1.0, 0.0, -5.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(3.0, 12.0, -3.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "front_right_leg",
                offset: Vec3::new(-5.0, 0.0, -5.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(-3.0, 12.0, -3.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "back_left_leg",
                offset: Vec3::new(1.0, 0.0, 3.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(3.0, 12.0, 5.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
        ],
    }
}

/// Sheep: head, woolly body, 4 legs (5 parts).
pub fn sheep_model() -> MobModel {
    MobModel {
        parts: vec![
            MobModelPart {
                name: "head",
                offset: Vec3::new(-4.0, 16.0, -10.0),
                size: Vec3::new(8.0, 6.0, 6.0),
                pivot: Vec3::new(0.0, 19.0, -7.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            MobModelPart {
                name: "body",
                offset: Vec3::new(-6.0, 6.0, -5.0),
                size: Vec3::new(12.0, 10.0, 10.0),
                pivot: Vec3::new(0.0, 11.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (28, 8),
            },
            MobModelPart {
                name: "front_left_leg",
                offset: Vec3::new(1.0, 0.0, -5.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(3.0, 12.0, -3.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "front_right_leg",
                offset: Vec3::new(-5.0, 0.0, -5.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(-3.0, 12.0, -3.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
            MobModelPart {
                name: "back_left_leg",
                offset: Vec3::new(1.0, 0.0, 3.0),
                size: Vec3::new(4.0, 12.0, 4.0),
                pivot: Vec3::new(3.0, 12.0, 5.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 16),
            },
        ],
    }
}

/// Chicken: head, body, left_wing, right_wing, left_leg (thin) (5 parts).
pub fn chicken_model() -> MobModel {
    MobModel {
        parts: vec![
            MobModelPart {
                name: "head",
                offset: Vec3::new(-2.0, 9.0, -5.0),
                size: Vec3::new(4.0, 6.0, 3.0),
                pivot: Vec3::new(0.0, 12.0, -3.5),
                rotation: Vec3::ZERO,
                tex_offset: (0, 0),
            },
            MobModelPart {
                name: "body",
                offset: Vec3::new(-3.0, 4.0, -2.0),
                size: Vec3::new(6.0, 6.0, 4.0),
                pivot: Vec3::new(0.0, 7.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (0, 9),
            },
            MobModelPart {
                name: "left_wing",
                offset: Vec3::new(3.0, 4.0, -1.5),
                size: Vec3::new(4.0, 4.0, 1.0),
                pivot: Vec3::new(3.0, 8.0, -1.0),
                rotation: Vec3::ZERO,
                tex_offset: (24, 13),
            },
            MobModelPart {
                name: "right_wing",
                offset: Vec3::new(-7.0, 4.0, -1.5),
                size: Vec3::new(4.0, 4.0, 1.0),
                pivot: Vec3::new(-3.0, 8.0, -1.0),
                rotation: Vec3::ZERO,
                tex_offset: (24, 13),
            },
            MobModelPart {
                name: "left_leg",
                offset: Vec3::new(0.0, 0.0, -1.0),
                size: Vec3::new(1.0, 5.0, 1.0),
                pivot: Vec3::new(1.0, 4.0, 0.0),
                rotation: Vec3::ZERO,
                tex_offset: (26, 0),
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Return the appropriate `MobModel` for a `MobKind` ordinal.
///
/// Ordinals follow the `MobKind` enum declaration order:
///   0=Zombie, 1=Skeleton, 2=Creeper, 3=Spider,
///   4=Pig, 5=Cow, 6=Sheep, 7=Chicken.
pub fn model_for_mob(kind: u8) -> MobModel {
    match kind {
        0 => zombie_model(),
        1 => skeleton_model(),
        2 => creeper_model(),
        3 => spider_model(),
        4 => pig_model(),
        5 => cow_model(),
        6 => sheep_model(),
        7 => chicken_model(),
        _ => zombie_model(), // fallback
    }
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

/// Swing legs and arms sinusoidally for a walking animation.
///
/// Legs rotate around the X-axis: `sin(time * speed) * 0.7`.
/// Arms swing in the opposite phase. Head stays still.
pub fn animate_walk(model: &mut MobModel, time: f32, speed: f32) {
    let swing = (time * speed).sin() * 0.7;

    for part in &mut model.parts {
        match part.name {
            // Bipedal mobs (zombie, skeleton)
            "left_leg" => part.rotation.x = swing,
            "right_leg" => part.rotation.x = -swing,
            "left_arm" => part.rotation.x = -swing,
            "right_arm" => part.rotation.x = swing,

            // Quadruped front/back legs (creeper, pig, cow, sheep)
            "front_left_leg" => part.rotation.x = swing,
            "front_right_leg" => part.rotation.x = -swing,
            "back_left_leg" => part.rotation.x = -swing,
            "back_right_leg" => part.rotation.x = swing,

            // Spider legs — alternate pairs
            "left_leg_1" | "left_leg_3" => part.rotation.x = swing,
            "left_leg_2" | "left_leg_4" => part.rotation.x = -swing,
            "right_leg_1" | "right_leg_3" => part.rotation.x = -swing,
            "right_leg_2" | "right_leg_4" => part.rotation.x = swing,

            // Chicken wings flap during walk
            "left_wing" => part.rotation.z = -swing * 0.5,
            "right_wing" => part.rotation.z = swing * 0.5,

            _ => {}
        }
    }
}

/// Subtle idle animation: gentle head bob.
pub fn animate_idle(model: &mut MobModel, time: f32) {
    let bob = (time * 1.5).sin() * 0.05;

    for part in &mut model.parts {
        if part.name == "head" {
            part.rotation.x = bob;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zombie_model_has_six_parts() {
        let model = zombie_model();
        assert_eq!(model.parts.len(), 6);
    }

    #[test]
    fn skeleton_model_has_six_parts() {
        let model = skeleton_model();
        assert_eq!(model.parts.len(), 6);
    }

    #[test]
    fn creeper_model_has_five_parts() {
        let model = creeper_model();
        assert_eq!(model.parts.len(), 5);
    }

    #[test]
    fn spider_model_has_ten_parts() {
        let model = spider_model();
        assert_eq!(model.parts.len(), 10);
    }

    #[test]
    fn pig_model_has_five_parts() {
        let model = pig_model();
        assert_eq!(model.parts.len(), 5);
    }

    #[test]
    fn cow_model_has_five_parts() {
        let model = cow_model();
        assert_eq!(model.parts.len(), 5);
    }

    #[test]
    fn sheep_model_has_five_parts() {
        let model = sheep_model();
        assert_eq!(model.parts.len(), 5);
    }

    #[test]
    fn chicken_model_has_five_parts() {
        let model = chicken_model();
        assert_eq!(model.parts.len(), 5);
    }

    #[test]
    fn model_for_mob_returns_correct_model() {
        // Zombie (ordinal 0) has 6 parts
        assert_eq!(model_for_mob(0).parts.len(), 6);
        // Skeleton (ordinal 1) has 6 parts
        assert_eq!(model_for_mob(1).parts.len(), 6);
        // Creeper (ordinal 2) has 5 parts
        assert_eq!(model_for_mob(2).parts.len(), 5);
        // Spider (ordinal 3) has 10 parts
        assert_eq!(model_for_mob(3).parts.len(), 10);
        // Pig (ordinal 4) has 5 parts
        assert_eq!(model_for_mob(4).parts.len(), 5);
        // Cow (ordinal 5) has 5 parts
        assert_eq!(model_for_mob(5).parts.len(), 5);
        // Sheep (ordinal 6) has 5 parts
        assert_eq!(model_for_mob(6).parts.len(), 5);
        // Chicken (ordinal 7) has 5 parts
        assert_eq!(model_for_mob(7).parts.len(), 5);
        // Unknown falls back to zombie
        assert_eq!(model_for_mob(255).parts.len(), 6);
    }

    #[test]
    fn animate_walk_changes_leg_rotation() {
        let mut model = zombie_model();

        // All rotations start at zero
        for part in &model.parts {
            assert!(
                part.rotation.x.abs() < f32::EPSILON,
                "part '{}' should start with zero x rotation",
                part.name,
            );
        }

        animate_walk(&mut model, 1.0, 5.0);

        let left_leg = model.parts.iter().find(|p| p.name == "left_leg").unwrap();
        let right_leg = model.parts.iter().find(|p| p.name == "right_leg").unwrap();

        // Legs must have non-zero rotation after animation
        assert!(
            left_leg.rotation.x.abs() > f32::EPSILON,
            "left_leg should have non-zero rotation after walk animation",
        );
        assert!(
            right_leg.rotation.x.abs() > f32::EPSILON,
            "right_leg should have non-zero rotation after walk animation",
        );

        // Left and right legs swing in opposite directions
        assert!(
            (left_leg.rotation.x + right_leg.rotation.x).abs() < f32::EPSILON,
            "legs should swing in opposite directions",
        );
    }

    #[test]
    fn animate_walk_changes_arm_rotation() {
        let mut model = zombie_model();
        animate_walk(&mut model, 1.0, 5.0);

        let left_arm = model.parts.iter().find(|p| p.name == "left_arm").unwrap();
        let right_arm = model.parts.iter().find(|p| p.name == "right_arm").unwrap();

        assert!(left_arm.rotation.x.abs() > f32::EPSILON);
        assert!(right_arm.rotation.x.abs() > f32::EPSILON);

        // Arms swing opposite to each other (and opposite to same-side leg)
        assert!(
            (left_arm.rotation.x + right_arm.rotation.x).abs() < f32::EPSILON,
            "arms should swing in opposite directions",
        );
    }

    #[test]
    fn animate_walk_head_stays_still() {
        let mut model = zombie_model();
        animate_walk(&mut model, 1.0, 5.0);

        let head = model.parts.iter().find(|p| p.name == "head").unwrap();
        assert!(
            head.rotation.x.abs() < f32::EPSILON,
            "head should not move during walk animation",
        );
    }

    #[test]
    fn animate_idle_bobs_head() {
        let mut model = zombie_model();
        animate_idle(&mut model, 1.0);

        let head = model.parts.iter().find(|p| p.name == "head").unwrap();
        assert!(
            head.rotation.x.abs() > f32::EPSILON,
            "head should bob during idle animation",
        );
    }

    #[test]
    fn animate_walk_quadruped_legs() {
        let mut model = pig_model();
        animate_walk(&mut model, 1.0, 5.0);

        let fl = model
            .parts
            .iter()
            .find(|p| p.name == "front_left_leg")
            .unwrap();
        let fr = model
            .parts
            .iter()
            .find(|p| p.name == "front_right_leg")
            .unwrap();

        assert!(fl.rotation.x.abs() > f32::EPSILON);
        assert!((fl.rotation.x + fr.rotation.x).abs() < f32::EPSILON);
    }

    #[test]
    fn animate_walk_spider_legs() {
        let mut model = spider_model();
        animate_walk(&mut model, 1.0, 5.0);

        let l1 = model.parts.iter().find(|p| p.name == "left_leg_1").unwrap();
        let l2 = model.parts.iter().find(|p| p.name == "left_leg_2").unwrap();

        assert!(l1.rotation.x.abs() > f32::EPSILON);
        // Adjacent legs on the same side swing in opposite directions
        assert!((l1.rotation.x + l2.rotation.x).abs() < f32::EPSILON);
    }

    #[test]
    fn skeleton_arms_are_thinner_than_zombie() {
        let zombie = zombie_model();
        let skeleton = skeleton_model();

        let z_arm = zombie.parts.iter().find(|p| p.name == "left_arm").unwrap();
        let s_arm = skeleton
            .parts
            .iter()
            .find(|p| p.name == "left_arm")
            .unwrap();

        assert!(
            s_arm.size.x < z_arm.size.x,
            "skeleton arms should be thinner than zombie arms",
        );
        assert!(
            (s_arm.size.x - 2.0).abs() < f32::EPSILON,
            "skeleton arm width should be 2 pixels",
        );
    }

    #[test]
    fn entity_render_data_fields() {
        let data = EntityRenderData {
            position: Vec3::new(1.0, 2.0, 3.0),
            yaw: 90.0,
            model_type: 0,
            animation_time: 1.5,
        };
        assert!((data.position.x - 1.0).abs() < f32::EPSILON);
        assert!((data.yaw - 90.0).abs() < f32::EPSILON);
        assert_eq!(data.model_type, 0);
        assert!((data.animation_time - 1.5).abs() < f32::EPSILON);
    }
}
