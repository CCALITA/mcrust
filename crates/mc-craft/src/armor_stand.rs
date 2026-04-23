//! Armor stand pose system.
//!
//! Defines pose presets and equipment data for armor stand entities.

use std::f32::consts::FRAC_PI_2;

/// Euler angles (x, y, z) in radians for a single body part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmorStandPose {
    pub head: [f32; 3],
    pub body: [f32; 3],
    pub left_arm: [f32; 3],
    pub right_arm: [f32; 3],
    pub left_leg: [f32; 3],
    pub right_leg: [f32; 3],
}

impl Default for ArmorStandPose {
    fn default() -> Self {
        Self {
            head: [0.0; 3],
            body: [0.0; 3],
            left_arm: [0.0; 3],
            right_arm: [0.0; 3],
            left_leg: [0.0; 3],
            right_leg: [0.0; 3],
        }
    }
}

/// Named preset poses for armor stands.
///
/// Each entry is a `(&str, ArmorStandPose)` pair.
pub const PRESET_POSES: [(&str, ArmorStandPose); 6] = [
    // default: all zeros
    (
        "default",
        ArmorStandPose {
            head: [0.0; 3],
            body: [0.0; 3],
            left_arm: [0.0; 3],
            right_arm: [0.0; 3],
            left_leg: [0.0; 3],
            right_leg: [0.0; 3],
        },
    ),
    // zombie: arms forward at -90 degrees (negative FRAC_PI_2)
    (
        "zombie",
        ArmorStandPose {
            head: [0.0; 3],
            body: [0.0; 3],
            left_arm: [-FRAC_PI_2, 0.0, 0.0],
            right_arm: [-FRAC_PI_2, 0.0, 0.0],
            left_leg: [0.0; 3],
            right_leg: [0.0; 3],
        },
    ),
    // t-pose: arms out at +-90 degrees on the z-axis
    (
        "t-pose",
        ArmorStandPose {
            head: [0.0; 3],
            body: [0.0; 3],
            left_arm: [0.0, 0.0, FRAC_PI_2],
            right_arm: [0.0, 0.0, -FRAC_PI_2],
            left_leg: [0.0; 3],
            right_leg: [0.0; 3],
        },
    ),
    // walking: alternating legs and arms
    (
        "walking",
        ArmorStandPose {
            head: [0.0; 3],
            body: [0.0; 3],
            left_arm: [0.5, 0.0, 0.0],
            right_arm: [-0.5, 0.0, 0.0],
            left_leg: [-0.5, 0.0, 0.0],
            right_leg: [0.5, 0.0, 0.0],
        },
    ),
    // attention: arms at sides, legs together (same as default but named)
    (
        "attention",
        ArmorStandPose {
            head: [0.0; 3],
            body: [0.0; 3],
            left_arm: [0.0; 3],
            right_arm: [0.0; 3],
            left_leg: [0.0; 3],
            right_leg: [0.0; 3],
        },
    ),
    // waving: right arm up at 45 degrees (negative on z to raise outward)
    (
        "waving",
        ArmorStandPose {
            head: [0.0; 3],
            body: [0.0; 3],
            left_arm: [0.0; 3],
            right_arm: [-0.7853982, 0.0, -0.7853982], // -PI/4 on x and z
            left_leg: [0.0; 3],
            right_leg: [0.0; 3],
        },
    ),
];

/// Equipment slot indices:
/// 0 = head, 1 = chest, 2 = legs, 3 = feet, 4 = mainhand, 5 = offhand.
const SLOT_NAMES: [&str; 6] = ["head", "chest", "legs", "feet", "mainhand", "offhand"];

/// Returns the human-readable name for an equipment slot index.
///
/// Slots: 0=head, 1=chest, 2=legs, 3=feet, 4=mainhand, 5=offhand.
///
/// # Panics
///
/// Panics if `slot >= 6`.
pub fn equipment_slot_name(slot: usize) -> &'static str {
    SLOT_NAMES[slot]
}

/// Data for an armor stand entity.
///
/// `equipment` holds optional item IDs for each of the 6 slots.
#[derive(Debug, Clone)]
pub struct ArmorStandData {
    /// Equipment in slots: head, chest, legs, feet, mainhand, offhand.
    pub equipment: [Option<u16>; 6],
    /// Current pose of the armor stand.
    pub pose: ArmorStandPose,
    /// Whether to render arms.
    pub show_arms: bool,
    /// Whether this is a small (baby-sized) armor stand.
    pub small: bool,
    /// Whether to hide the stone base plate.
    pub no_base_plate: bool,
    /// Whether the armor stand is invisible (equipment still renders).
    pub invisible: bool,
}

impl ArmorStandData {
    /// Creates a new armor stand with default settings.
    pub fn new() -> Self {
        Self {
            equipment: [None; 6],
            pose: ArmorStandPose::default(),
            show_arms: false,
            small: false,
            no_base_plate: false,
            invisible: false,
        }
    }
}

impl Default for ArmorStandData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pose_is_all_zeros() {
        let pose = ArmorStandPose::default();
        assert_eq!(pose.head, [0.0; 3]);
        assert_eq!(pose.body, [0.0; 3]);
        assert_eq!(pose.left_arm, [0.0; 3]);
        assert_eq!(pose.right_arm, [0.0; 3]);
        assert_eq!(pose.left_leg, [0.0; 3]);
        assert_eq!(pose.right_leg, [0.0; 3]);
    }

    #[test]
    fn preset_count_is_six() {
        assert_eq!(PRESET_POSES.len(), 6);
    }

    #[test]
    fn preset_names_are_correct() {
        let names: Vec<&str> = PRESET_POSES.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            names,
            vec!["default", "zombie", "t-pose", "walking", "attention", "waving"]
        );
    }

    #[test]
    fn equipment_slot_names() {
        assert_eq!(equipment_slot_name(0), "head");
        assert_eq!(equipment_slot_name(1), "chest");
        assert_eq!(equipment_slot_name(2), "legs");
        assert_eq!(equipment_slot_name(3), "feet");
        assert_eq!(equipment_slot_name(4), "mainhand");
        assert_eq!(equipment_slot_name(5), "offhand");
    }

    #[test]
    #[should_panic]
    fn equipment_slot_name_out_of_bounds() {
        equipment_slot_name(6);
    }

    #[test]
    fn new_armor_stand_has_empty_equipment() {
        let stand = ArmorStandData::new();
        for slot in &stand.equipment {
            assert!(slot.is_none());
        }
    }

    #[test]
    fn new_armor_stand_defaults() {
        let stand = ArmorStandData::new();
        assert!(!stand.show_arms);
        assert!(!stand.small);
        assert!(!stand.no_base_plate);
        assert!(!stand.invisible);
        assert_eq!(stand.pose, ArmorStandPose::default());
    }

    #[test]
    fn small_armor_stand() {
        let mut stand = ArmorStandData::new();
        stand.small = true;
        assert!(stand.small);
    }

    #[test]
    fn zombie_pose_has_arms_forward() {
        let (_, pose) = &PRESET_POSES[1];
        assert_eq!(pose.left_arm[0], -FRAC_PI_2);
        assert_eq!(pose.right_arm[0], -FRAC_PI_2);
    }
}
