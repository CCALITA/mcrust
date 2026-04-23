//! Title screen layout, splash text, and main menu button positioning.
//!
//! Provides deterministic splash selection from a fixed message list, an
//! oscillating scale factor for the bouncing splash text, and a layout helper
//! that computes screen-space rectangles for the logo, splash text, and the
//! "Singleplayer" / "Quit" buttons.

use std::f32::consts::TAU;

/// Version label rendered in the bottom-left corner of the title screen.
pub const VERSION_STRING: &str = "MCRust 0.1.0";

/// Rotating splash text shown in yellow next to the logo.
pub const SPLASH_MESSAGES: &[&str] = &[
    "Also try Terraria!",
    "Singleplayer!",
    "100% Rust!",
    "Now with wgpu!",
    "Chunk-tastic!",
    "Mine! Craft!",
    "Built with Claude Code!",
    "Open source!",
    "Not affiliated with Mojang!",
    "Procedurally generated!",
    "64-bit!",
    "Smooth lighting!",
    "3D!",
    "Community driven!",
    "Explore the depths!",
    "Diamond level!",
    "Watch out for creepers!",
    "WASD to move!",
    "F3 for debug!",
    "Survival mode!",
    "It's a feature!",
    "Cross-platform!",
    "Blazingly fast!",
    "Memory safe!",
    "No garbage collector!",
    "Borrow checked!",
    "Zero cost abstractions!",
    "Fearless concurrency!",
    "Hello World!",
    "42",
];

/// Pick a splash message deterministically from `seed`.
///
/// The same seed always returns the same splash. Different seeds spread evenly
/// across [`SPLASH_MESSAGES`].
pub fn select_splash(seed: u64) -> &'static str {
    let index = (seed % SPLASH_MESSAGES.len() as u64) as usize;
    SPLASH_MESSAGES[index]
}

/// Bouncing scale factor for the splash text.
///
/// Returns a value in `[0.9, 1.1]` driven by a 1-second sinusoid.
pub fn splash_scale(time: f32) -> f32 {
    1.0 + 0.1 * (time * TAU).sin()
}

/// Screen-space rectangles for every visible element on the title screen.
///
/// All rectangles are `(x, y, width, height)` in pixels with origin at the
/// top-left of the screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitleScreenLayout {
    pub logo_rect: (f32, f32, f32, f32),
    pub splash_pos: (f32, f32),
    pub singleplayer_btn: (f32, f32, f32, f32),
    pub quit_btn: (f32, f32, f32, f32),
}

/// Compute the title-screen layout for a screen of size `(screen_w, screen_h)`.
///
/// The logo sits in the upper third, splash text overlays its top-right corner,
/// and two stacked buttons (Singleplayer above Quit) are centered horizontally
/// in the lower half.
pub fn layout_title_screen(screen_w: f32, screen_h: f32) -> TitleScreenLayout {
    let logo_w = screen_w * 0.5;
    let logo_h = logo_w * 0.2;
    let logo_x = (screen_w - logo_w) / 2.0;
    let logo_y = screen_h * 0.15;

    let splash_pos = (logo_x + logo_w * 0.85, logo_y + logo_h * 0.5);

    let btn_w = 300.0;
    let btn_h = 40.0;
    let btn_x = (screen_w - btn_w) / 2.0;
    let singleplayer_y = screen_h * 0.55;
    let quit_y = singleplayer_y + btn_h + 10.0;

    TitleScreenLayout {
        logo_rect: (logo_x, logo_y, logo_w, logo_h),
        splash_pos,
        singleplayer_btn: (btn_x, singleplayer_y, btn_w, btn_h),
        quit_btn: (btn_x, quit_y, btn_w, btn_h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splash_count_is_thirty() {
        assert_eq!(SPLASH_MESSAGES.len(), 30);
    }

    #[test]
    fn splash_messages_are_non_empty() {
        for msg in SPLASH_MESSAGES {
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn select_splash_is_deterministic() {
        for seed in 0u64..200 {
            assert_eq!(select_splash(seed), select_splash(seed));
        }
    }

    #[test]
    fn select_splash_wraps_with_modulo() {
        let n = SPLASH_MESSAGES.len() as u64;
        for seed in 0u64..n {
            assert_eq!(select_splash(seed), select_splash(seed + n));
            assert_eq!(select_splash(seed), select_splash(seed + n * 7));
        }
    }

    #[test]
    fn select_splash_zero_seed() {
        assert_eq!(select_splash(0), "Also try Terraria!");
    }

    #[test]
    fn splash_scale_within_range() {
        for i in 0..1000 {
            let t = i as f32 * 0.013;
            let s = splash_scale(t);
            assert!(s >= 0.9 - 1e-5, "scale {s} below 0.9 at t={t}");
            assert!(s <= 1.1 + 1e-5, "scale {s} above 1.1 at t={t}");
        }
    }

    #[test]
    fn splash_scale_baseline_at_zero() {
        assert!((splash_scale(0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn splash_scale_period_one_second() {
        // A 1-second period means scale(t) == scale(t + 1.0) for all t.
        for i in 0..50 {
            let t = i as f32 * 0.05;
            assert!((splash_scale(t) - splash_scale(t + 1.0)).abs() < 1e-4);
        }
    }

    #[test]
    fn layout_logo_is_horizontally_centered() {
        let layout = layout_title_screen(1280.0, 720.0);
        let (x, _y, w, _h) = layout.logo_rect;
        let expected_x = (1280.0 - w) / 2.0;
        assert!((x - expected_x).abs() < 1e-3);
    }

    #[test]
    fn layout_logo_in_upper_half() {
        let layout = layout_title_screen(1280.0, 720.0);
        let (_x, y, _w, h) = layout.logo_rect;
        assert!(y < 720.0 / 2.0);
        assert!(y + h < 720.0 / 2.0);
    }

    #[test]
    fn layout_buttons_are_centered_and_stacked() {
        let layout = layout_title_screen(1280.0, 720.0);
        let (sx, sy, sw, sh) = layout.singleplayer_btn;
        let (qx, qy, qw, qh) = layout.quit_btn;
        assert!((sx - qx).abs() < 1e-5);
        assert!((sw - qw).abs() < 1e-5);
        assert!((sh - qh).abs() < 1e-5);
        assert!(qy > sy + sh, "quit button must sit below singleplayer");
        let expected_x = (1280.0 - sw) / 2.0;
        assert!((sx - expected_x).abs() < 1e-3);
    }

    #[test]
    fn layout_buttons_in_lower_half() {
        let layout = layout_title_screen(1280.0, 720.0);
        assert!(layout.singleplayer_btn.1 > 720.0 / 2.0);
        assert!(layout.quit_btn.1 > 720.0 / 2.0);
    }

    #[test]
    fn layout_splash_near_logo_top_right() {
        let layout = layout_title_screen(1280.0, 720.0);
        let (lx, ly, lw, lh) = layout.logo_rect;
        let (sx, sy) = layout.splash_pos;
        assert!(sx > lx + lw * 0.5);
        assert!(sy >= ly && sy <= ly + lh);
    }

    #[test]
    fn layout_scales_with_screen_size() {
        let small = layout_title_screen(800.0, 600.0);
        let large = layout_title_screen(1920.0, 1080.0);
        assert!(large.logo_rect.2 > small.logo_rect.2);
    }

    #[test]
    fn version_string_is_expected() {
        assert_eq!(VERSION_STRING, "MCRust 0.1.0");
    }
}
