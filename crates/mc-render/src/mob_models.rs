use glam::Vec3;

use crate::entity_render::{MobModel, MobModelPart};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_model_has_correct_part_count() {
        let cases: Vec<(&str, MobModel, usize)> = vec![
            ("zombie", zombie_model(), 6),
            ("skeleton", skeleton_model(), 6),
            ("creeper", creeper_model(), 5),
            ("spider", spider_model(), 10),
            ("pig", pig_model(), 5),
            ("cow", cow_model(), 5),
            ("sheep", sheep_model(), 5),
            ("chicken", chicken_model(), 5),
        ];
        for (name, model, expected) in cases {
            assert_eq!(
                model.parts.len(),
                expected,
                "{name} model should have {expected} parts",
            );
        }
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
}
