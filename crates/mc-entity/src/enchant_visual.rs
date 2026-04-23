//! Enchantment glow and visual particle effects.
//!
//! Provides [`EnchantGlowColor`] for rendering the shimmer overlay on enchanted
//! items, plus particle generators for specific enchantments such as Fire Aspect,
//! Frost Walker, Thorns, and Sharpness.

use glam::Vec3;

/// Color and intensity descriptor for the enchantment shimmer overlay.
#[derive(Debug, Clone, PartialEq)]
pub struct EnchantGlowColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub intensity: f32,
}

/// Return the purple glow color for an enchanted item.
///
/// Intensity scales from 0.4 (one enchantment) up to 0.8 (five or more),
/// following `0.3 + 0.1 * min(enchant_count, 5)`.
pub fn enchant_glow(enchant_count: u8) -> EnchantGlowColor {
    let capped = enchant_count.min(5) as f32;
    EnchantGlowColor {
        r: 0.6,
        g: 0.2,
        b: 0.8,
        intensity: 0.3 + 0.1 * capped,
    }
}

/// Returns `true` when the item carries at least one enchantment.
pub fn has_enchant_glow(enchant_count: u8) -> bool {
    enchant_count > 0
}

/// Spawn 3-5 orange flame particles near a weapon's tip for Fire Aspect.
pub fn fire_aspect_particles(pos: Vec3) -> Vec<(Vec3, [f32; 3])> {
    let color: [f32; 3] = [1.0, 0.5, 0.0];
    let offsets: &[(f32, f32, f32)] = &[
        (0.0, 0.1, 0.0),
        (0.05, 0.15, -0.05),
        (-0.05, 0.12, 0.05),
        (0.08, 0.08, 0.0),
    ];

    offsets
        .iter()
        .map(|&(dx, dy, dz)| (pos + Vec3::new(dx, dy, dz), color))
        .collect()
}

/// Spawn 4 ice-blue particles at the player's feet for Frost Walker.
pub fn frost_walker_particles(foot_pos: Vec3) -> Vec<(Vec3, [f32; 3])> {
    let color: [f32; 3] = [0.5, 0.8, 1.0];
    let offsets: &[(f32, f32)] = &[
        (0.2, 0.0),
        (-0.2, 0.0),
        (0.0, 0.2),
        (0.0, -0.2),
    ];

    offsets
        .iter()
        .map(|&(dx, dz)| (foot_pos + Vec3::new(dx, 0.0, dz), color))
        .collect()
}

/// Spawn 2-3 red damage-flash particles around the attacker for Thorns.
pub fn thorns_visual(attacker_pos: Vec3) -> Vec<(Vec3, [f32; 3])> {
    let color: [f32; 3] = [0.8, 0.1, 0.1];
    let offsets: &[(f32, f32, f32)] = &[
        (0.1, 0.3, 0.0),
        (-0.1, 0.5, 0.1),
        (0.0, 0.4, -0.1),
    ];

    offsets
        .iter()
        .map(|&(dx, dy, dz)| (attacker_pos + Vec3::new(dx, dy, dz), color))
        .collect()
}

/// Spawn 6 white particles in a horizontal arc for a Sharpness sweep attack.
///
/// The arc is centered on `yaw` and spans roughly 120 degrees in front of the
/// origin at a fixed radius.
pub fn sharpness_sweep(origin: Vec3, yaw: f32) -> Vec<(Vec3, [f32; 3])> {
    let color: [f32; 3] = [0.9, 0.9, 0.9];
    let count = 6;
    let arc_span = std::f32::consts::FRAC_PI_3 * 2.0; // 120 degrees
    let half_arc = arc_span / 2.0;
    let radius = 1.5;

    (0..count)
        .map(|i| {
            let frac = i as f32 / (count - 1) as f32;
            let angle = yaw - half_arc + arc_span * frac;
            let (sin, cos) = angle.sin_cos();
            let pos = origin + Vec3::new(cos * radius, 0.5, sin * radius);
            (pos, color)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- enchant_glow tests ----

    #[test]
    fn glow_intensity_scales_with_enchant_count() {
        let glow1 = enchant_glow(1);
        let glow3 = enchant_glow(3);
        let glow5 = enchant_glow(5);

        assert!((glow1.intensity - 0.4).abs() < f32::EPSILON);
        assert!((glow3.intensity - 0.6).abs() < f32::EPSILON);
        assert!((glow5.intensity - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn glow_intensity_capped_at_five() {
        let glow5 = enchant_glow(5);
        let glow10 = enchant_glow(10);
        assert!((glow5.intensity - glow10.intensity).abs() < f32::EPSILON);
    }

    #[test]
    fn glow_color_is_purple() {
        let glow = enchant_glow(1);
        assert!((glow.r - 0.6).abs() < f32::EPSILON);
        assert!((glow.g - 0.2).abs() < f32::EPSILON);
        assert!((glow.b - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn glow_zero_enchants_has_base_intensity() {
        let glow = enchant_glow(0);
        assert!((glow.intensity - 0.3).abs() < f32::EPSILON);
    }

    // ---- has_enchant_glow tests ----

    #[test]
    fn no_glow_for_zero_enchants() {
        assert!(!has_enchant_glow(0));
    }

    #[test]
    fn glow_for_nonzero_enchants() {
        assert!(has_enchant_glow(1));
        assert!(has_enchant_glow(5));
        assert!(has_enchant_glow(255));
    }

    // ---- fire_aspect_particles tests ----

    #[test]
    fn fire_aspect_produces_three_to_five_particles() {
        let particles = fire_aspect_particles(Vec3::ZERO);
        assert!(
            (3..=5).contains(&particles.len()),
            "expected 3-5 fire particles, got {}",
            particles.len()
        );
    }

    #[test]
    fn fire_aspect_color_is_orange() {
        for (_, color) in fire_aspect_particles(Vec3::ZERO) {
            assert_eq!(color, [1.0, 0.5, 0.0]);
        }
    }

    #[test]
    fn fire_aspect_particles_near_position() {
        let pos = Vec3::new(5.0, 10.0, 3.0);
        for (p, _) in fire_aspect_particles(pos) {
            assert!((p.x - pos.x).abs() < 0.5);
            assert!((p.y - pos.y).abs() < 0.5);
            assert!((p.z - pos.z).abs() < 0.5);
        }
    }

    // ---- frost_walker_particles tests ----

    #[test]
    fn frost_walker_produces_four_particles() {
        let particles = frost_walker_particles(Vec3::ZERO);
        assert_eq!(particles.len(), 4);
    }

    #[test]
    fn frost_walker_color_is_ice_blue() {
        for (_, color) in frost_walker_particles(Vec3::ZERO) {
            assert_eq!(color, [0.5, 0.8, 1.0]);
        }
    }

    #[test]
    fn frost_walker_particles_at_foot_level() {
        let foot = Vec3::new(0.0, 64.0, 0.0);
        for (p, _) in frost_walker_particles(foot) {
            assert!(
                (p.y - foot.y).abs() < f32::EPSILON,
                "frost particles should be at foot level"
            );
        }
    }

    // ---- thorns_visual tests ----

    #[test]
    fn thorns_produces_two_to_three_particles() {
        let particles = thorns_visual(Vec3::ZERO);
        assert!(
            (2..=3).contains(&particles.len()),
            "expected 2-3 thorns particles, got {}",
            particles.len()
        );
    }

    #[test]
    fn thorns_color_is_red() {
        for (_, color) in thorns_visual(Vec3::ZERO) {
            assert_eq!(color, [0.8, 0.1, 0.1]);
        }
    }

    // ---- sharpness_sweep tests ----

    #[test]
    fn sharpness_produces_six_particles() {
        let particles = sharpness_sweep(Vec3::ZERO, 0.0);
        assert_eq!(particles.len(), 6);
    }

    #[test]
    fn sharpness_color_is_white() {
        for (_, color) in sharpness_sweep(Vec3::ZERO, 0.0) {
            assert_eq!(color, [0.9, 0.9, 0.9]);
        }
    }

    #[test]
    fn sharpness_particles_form_arc_at_radius() {
        let origin = Vec3::ZERO;
        let particles = sharpness_sweep(origin, 0.0);
        for (p, _) in &particles {
            let dist = Vec3::new(p.x, 0.0, p.z).length();
            assert!(
                (dist - 1.5).abs() < 0.1,
                "particle should be near radius 1.5, got {dist}"
            );
        }
    }

    #[test]
    fn sharpness_sweep_respects_yaw() {
        let origin = Vec3::ZERO;
        let p0 = sharpness_sweep(origin, 0.0);
        let p_half_pi = sharpness_sweep(origin, std::f32::consts::FRAC_PI_2);

        // The two sets of positions should differ when yaw differs.
        let differs = p0.iter().zip(p_half_pi.iter()).any(|((a, _), (b, _))| {
            (a.x - b.x).abs() > 0.01 || (a.z - b.z).abs() > 0.01
        });
        assert!(differs, "different yaw should produce different particle positions");
    }
}
