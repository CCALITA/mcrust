use glam::Vec3;

use crate::mob_models;

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
// Dispatch
// ---------------------------------------------------------------------------

/// Return the appropriate `MobModel` for a `MobKind` ordinal.
///
/// Ordinals follow the `MobKind` enum declaration order:
///   0=Zombie, 1=Skeleton, 2=Creeper, 3=Spider,
///   4=Pig, 5=Cow, 6=Sheep, 7=Chicken.
pub fn model_for_mob(kind: u8) -> MobModel {
    match kind {
        0 => mob_models::zombie_model(),
        1 => mob_models::skeleton_model(),
        2 => mob_models::creeper_model(),
        3 => mob_models::spider_model(),
        4 => mob_models::pig_model(),
        5 => mob_models::cow_model(),
        6 => mob_models::sheep_model(),
        7 => mob_models::chicken_model(),
        _ => mob_models::zombie_model(), // fallback
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
        let mut model = mob_models::zombie_model();

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
        let mut model = mob_models::zombie_model();
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
        let mut model = mob_models::zombie_model();
        animate_walk(&mut model, 1.0, 5.0);

        let head = model.parts.iter().find(|p| p.name == "head").unwrap();
        assert!(
            head.rotation.x.abs() < f32::EPSILON,
            "head should not move during walk animation",
        );
    }

    #[test]
    fn animate_idle_bobs_head() {
        let mut model = mob_models::zombie_model();
        animate_idle(&mut model, 1.0);

        let head = model.parts.iter().find(|p| p.name == "head").unwrap();
        assert!(
            head.rotation.x.abs() > f32::EPSILON,
            "head should bob during idle animation",
        );
    }

    #[test]
    fn animate_walk_quadruped_legs() {
        let mut model = mob_models::pig_model();
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
        let mut model = mob_models::spider_model();
        animate_walk(&mut model, 1.0, 5.0);

        let l1 = model.parts.iter().find(|p| p.name == "left_leg_1").unwrap();
        let l2 = model.parts.iter().find(|p| p.name == "left_leg_2").unwrap();

        assert!(l1.rotation.x.abs() > f32::EPSILON);
        // Adjacent legs on the same side swing in opposite directions
        assert!((l1.rotation.x + l2.rotation.x).abs() < f32::EPSILON);
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
