//! Optimized frustum culling for batch AABB testing.

/// A frustum plane defined by normal and distance from origin.
/// Plane equation: normal . point + distance = 0.
#[derive(Debug, Clone, Copy)]
pub struct FrustumPlane {
    pub normal: [f32; 3],
    pub distance: f32,
}

/// Result of testing an AABB against the frustum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullResult {
    /// Entirely inside the frustum.
    Inside,
    /// Entirely outside the frustum.
    Outside,
    /// Partially inside (intersects at least one plane).
    Intersecting,
}

/// Extract the six frustum planes from a combined view-projection matrix.
///
/// Uses the Gribb-Hartmann method. Planes are ordered:
/// Left, Right, Bottom, Top, Near, Far.
/// Each plane is normalized so that `normal` is unit length.
pub fn extract_planes(view_proj: &[[f32; 4]; 4]) -> [FrustumPlane; 6] {
    let m = view_proj;

    let raw = [
        // Left:   row3 + row0
        [m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0], m[3][3] + m[3][0]],
        // Right:  row3 - row0
        [m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0], m[3][3] - m[3][0]],
        // Bottom: row3 + row1
        [m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1], m[3][3] + m[3][1]],
        // Top:    row3 - row1
        [m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1], m[3][3] - m[3][1]],
        // Near:   row3 + row2
        [m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2], m[3][3] + m[3][2]],
        // Far:    row3 - row2
        [m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2], m[3][3] - m[3][2]],
    ];

    let mut planes = [FrustumPlane { normal: [0.0; 3], distance: 0.0 }; 6];
    for (i, r) in raw.iter().enumerate() {
        let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        if len > f32::EPSILON {
            let inv = 1.0 / len;
            planes[i] = FrustumPlane {
                normal: [r[0] * inv, r[1] * inv, r[2] * inv],
                distance: r[3] * inv,
            };
        }
    }
    planes
}

/// Test a single AABB against the six frustum planes.
///
/// Uses the p-vertex / n-vertex optimization: for each plane, find the
/// corner closest to the plane (n-vertex) and farthest (p-vertex).
pub fn test_aabb_planes(
    min: [f32; 3],
    max: [f32; 3],
    planes: &[FrustumPlane; 6],
) -> CullResult {
    let mut all_inside = true;

    for plane in planes {
        let n = &plane.normal;

        // p-vertex: the corner farthest along the plane normal
        let px = if n[0] >= 0.0 { max[0] } else { min[0] };
        let py = if n[1] >= 0.0 { max[1] } else { min[1] };
        let pz = if n[2] >= 0.0 { max[2] } else { min[2] };

        // n-vertex: the corner closest along the plane normal
        let nx = if n[0] >= 0.0 { min[0] } else { max[0] };
        let ny = if n[1] >= 0.0 { min[1] } else { max[1] };
        let nz = if n[2] >= 0.0 { min[2] } else { max[2] };

        // If p-vertex is outside, entire AABB is outside
        let p_dist = n[0] * px + n[1] * py + n[2] * pz + plane.distance;
        if p_dist < 0.0 {
            return CullResult::Outside;
        }

        // If n-vertex is outside, AABB intersects this plane
        let n_dist = n[0] * nx + n[1] * ny + n[2] * nz + plane.distance;
        if n_dist < 0.0 {
            all_inside = false;
        }
    }

    if all_inside {
        CullResult::Inside
    } else {
        CullResult::Intersecting
    }
}

/// Batch cull multiple AABBs against the frustum.
///
/// Returns a `Vec<bool>` where `true` means the AABB is at least partially
/// visible (Inside or Intersecting).
pub fn batch_cull(
    aabbs: &[([f32; 3], [f32; 3])],
    planes: &[FrustumPlane; 6],
) -> Vec<bool> {
    aabbs
        .iter()
        .map(|(min, max)| test_aabb_planes(*min, *max, planes) != CullResult::Outside)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an orthographic-like view-proj that defines a box from -1..1 in all axes.
    fn identity_view_proj() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn extract_planes_identity_produces_six_planes() {
        let planes = extract_planes(&identity_view_proj());
        for plane in &planes {
            let len = (plane.normal[0].powi(2)
                + plane.normal[1].powi(2)
                + plane.normal[2].powi(2))
            .sqrt();
            assert!((len - 1.0).abs() < 1e-5, "plane normal should be unit length");
        }
    }

    #[test]
    fn aabb_inside_identity_frustum() {
        let planes = extract_planes(&identity_view_proj());
        let result = test_aabb_planes([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5], &planes);
        assert_eq!(result, CullResult::Inside);
    }

    #[test]
    fn aabb_outside_identity_frustum() {
        let planes = extract_planes(&identity_view_proj());
        let result = test_aabb_planes([2.0, 2.0, 2.0], [3.0, 3.0, 3.0], &planes);
        assert_eq!(result, CullResult::Outside);
    }

    #[test]
    fn aabb_intersecting_identity_frustum() {
        let planes = extract_planes(&identity_view_proj());
        let result = test_aabb_planes([-0.5, -0.5, -0.5], [1.5, 1.5, 1.5], &planes);
        assert_eq!(result, CullResult::Intersecting);
    }

    #[test]
    fn batch_cull_filters_correctly() {
        let planes = extract_planes(&identity_view_proj());
        let aabbs = vec![
            ([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),   // inside
            ([2.0, 2.0, 2.0], [3.0, 3.0, 3.0]),       // outside
            ([-0.5, -0.5, -0.5], [1.5, 1.5, 1.5]),     // intersecting
        ];
        let results = batch_cull(&aabbs, &planes);
        assert_eq!(results, vec![true, false, true]);
    }

    #[test]
    fn batch_cull_empty_input() {
        let planes = extract_planes(&identity_view_proj());
        let results = batch_cull(&[], &planes);
        assert!(results.is_empty());
    }

    #[test]
    fn aabb_outside_each_side() {
        let planes = extract_planes(&identity_view_proj());
        // Outside left
        assert_eq!(
            test_aabb_planes([-3.0, -0.5, -0.5], [-2.0, 0.5, 0.5], &planes),
            CullResult::Outside
        );
        // Outside right
        assert_eq!(
            test_aabb_planes([2.0, -0.5, -0.5], [3.0, 0.5, 0.5], &planes),
            CullResult::Outside
        );
        // Outside bottom
        assert_eq!(
            test_aabb_planes([-0.5, -3.0, -0.5], [0.5, -2.0, 0.5], &planes),
            CullResult::Outside
        );
        // Outside top
        assert_eq!(
            test_aabb_planes([-0.5, 2.0, -0.5], [0.5, 3.0, 0.5], &planes),
            CullResult::Outside
        );
        // Outside near
        assert_eq!(
            test_aabb_planes([-0.5, -0.5, -3.0], [0.5, 0.5, -2.0], &planes),
            CullResult::Outside
        );
        // Outside far
        assert_eq!(
            test_aabb_planes([-0.5, -0.5, 2.0], [0.5, 0.5, 3.0], &planes),
            CullResult::Outside
        );
    }
}
