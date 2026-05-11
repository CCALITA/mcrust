//! Mob pose animations: body offsets, leg angles, and pose transitions.

/// Possible poses for a mob entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MobPose {
    Standing,
    Sitting,
    Sleeping,
    Swimming,
    Climbing,
    Dying,
}

/// Returns the body position offset for a given pose.
pub fn pose_body_offset(pose: MobPose) -> [f32; 3] {
    match pose {
        MobPose::Standing => [0.0, 0.0, 0.0],
        MobPose::Sitting => [0.0, -0.4, 0.0],
        MobPose::Sleeping => [0.0, -0.8, 0.2],
        MobPose::Swimming => [0.0, -0.3, 0.4],
        MobPose::Climbing => [0.0, 0.1, -0.2],
        MobPose::Dying => [0.0, -0.6, 0.3],
    }
}

/// Returns the leg rotation angles (left_leg, right_leg) as euler angles [x, y, z] in radians.
pub fn pose_leg_angles(pose: MobPose) -> ([f32; 3], [f32; 3]) {
    match pose {
        MobPose::Standing => ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
        MobPose::Sitting => ([-1.4, 0.0, 0.0], [-1.4, 0.0, 0.0]),
        MobPose::Sleeping => ([-1.57, 0.0, 0.0], [-1.57, 0.0, 0.0]),
        MobPose::Swimming => ([0.3, 0.0, 0.0], [-0.3, 0.0, 0.0]),
        MobPose::Climbing => ([0.8, 0.0, 0.0], [-0.8, 0.0, 0.0]),
        MobPose::Dying => ([-0.5, 0.0, 0.3], [-0.5, 0.0, -0.3]),
    }
}

/// Returns the speed factor for transitioning between poses.
pub fn pose_transition_speed() -> f32 {
    5.0
}

/// Linearly interpolates between two 3D vectors by factor `t` (clamped to [0, 1]).
pub fn lerp_pose(from: [f32; 3], to: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        from[0] + (to[0] - from[0]) * t,
        from[1] + (to[1] - from[1]) * t,
        from[2] + (to[2] - from[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_has_zero_offset() {
        assert_eq!(pose_body_offset(MobPose::Standing), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn sitting_lowers_body() {
        let offset = pose_body_offset(MobPose::Sitting);
        assert!(offset[1] < 0.0);
    }

    #[test]
    fn standing_legs_are_neutral() {
        let (left, right) = pose_leg_angles(MobPose::Standing);
        assert_eq!(left, [0.0, 0.0, 0.0]);
        assert_eq!(right, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn sitting_legs_bend() {
        let (left, right) = pose_leg_angles(MobPose::Sitting);
        assert!(left[0] < 0.0);
        assert!(right[0] < 0.0);
    }

    #[test]
    fn transition_speed_is_five() {
        assert_eq!(pose_transition_speed(), 5.0);
    }

    #[test]
    fn lerp_at_zero_returns_from() {
        let from = [1.0, 2.0, 3.0];
        let to = [4.0, 5.0, 6.0];
        assert_eq!(lerp_pose(from, to, 0.0), from);
    }

    #[test]
    fn lerp_at_one_returns_to() {
        let from = [1.0, 2.0, 3.0];
        let to = [4.0, 5.0, 6.0];
        assert_eq!(lerp_pose(from, to, 1.0), to);
    }

    #[test]
    fn lerp_at_half() {
        let result = lerp_pose([0.0, 0.0, 0.0], [2.0, 4.0, 6.0], 0.5);
        assert_eq!(result, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn lerp_clamps_t_above_one() {
        let result = lerp_pose([0.0, 0.0, 0.0], [2.0, 4.0, 6.0], 2.0);
        assert_eq!(result, [2.0, 4.0, 6.0]);
    }

    #[test]
    fn lerp_clamps_t_below_zero() {
        let result = lerp_pose([0.0, 0.0, 0.0], [2.0, 4.0, 6.0], -1.0);
        assert_eq!(result, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn all_poses_have_distinct_offsets() {
        let poses = [
            MobPose::Standing,
            MobPose::Sitting,
            MobPose::Sleeping,
            MobPose::Swimming,
            MobPose::Climbing,
            MobPose::Dying,
        ];
        for i in 0..poses.len() {
            for j in (i + 1)..poses.len() {
                assert_ne!(
                    pose_body_offset(poses[i]),
                    pose_body_offset(poses[j]),
                    "Poses {:?} and {:?} should have different offsets",
                    poses[i],
                    poses[j]
                );
            }
        }
    }
}
