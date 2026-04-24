//! Animated texture frame system.
//!
//! Drives multi-frame texture animations (water, lava, fire, portal, etc.)
//! by tracking elapsed time and reporting the current frame plus an
//! interpolation factor between consecutive frames for smooth blending.

/// A frame-by-frame animated texture.
///
/// `frames` holds the texture IDs for each frame in playback order.
/// `frame_duration` is the number of seconds each frame is shown.
/// `current_time` is the elapsed time within the full animation cycle.
#[derive(Debug, Clone)]
pub struct AnimatedTexture {
    pub frames: Vec<u32>,
    pub frame_duration: f32,
    pub current_time: f32,
}

impl AnimatedTexture {
    /// Create a new animated texture from the given frame IDs and per-frame duration.
    pub fn new(frames: Vec<u32>, frame_duration: f32) -> Self {
        Self {
            frames,
            frame_duration,
            current_time: 0.0,
        }
    }

    /// Total cycle duration in seconds.
    fn cycle_duration(&self) -> f32 {
        self.frame_duration * self.frames.len() as f32
    }

    /// Advance the animation by `dt` seconds and return the current frame's texture ID.
    ///
    /// Time wraps around the total cycle. Returns 0 when there are no frames.
    pub fn tick(&mut self, dt: f32) -> u32 {
        if self.frames.is_empty() || self.frame_duration <= 0.0 {
            return 0;
        }
        let cycle = self.cycle_duration();
        self.current_time = (self.current_time + dt).rem_euclid(cycle);
        let idx = (self.current_time / self.frame_duration) as usize % self.frames.len();
        self.frames[idx]
    }

    /// Return `(prev_frame, next_frame, lerp_factor)` for smooth interpolation.
    ///
    /// `lerp_factor` is in `[0.0, 1.0)`, representing progress from prev to next.
    pub fn interpolated_frame(&self) -> (u32, u32, f32) {
        if self.frames.is_empty() || self.frame_duration <= 0.0 {
            return (0, 0, 0.0);
        }
        let len = self.frames.len();
        let position = self.current_time / self.frame_duration;
        let prev_idx = (position as usize) % len;
        let next_idx = (prev_idx + 1) % len;
        let lerp = position - position.floor();
        (self.frames[prev_idx], self.frames[next_idx], lerp)
    }
}

/// Build an animation whose frames are `base_id..base_id + count`.
fn sequential(base_id: u32, count: u32, frame_duration: f32) -> AnimatedTexture {
    let frames: Vec<u32> = (0..count).map(|i| base_id + i).collect();
    AnimatedTexture::new(frames, frame_duration)
}

/// Water: 32 frames, 0.05s each.
pub fn water_animation(base_id: u32) -> AnimatedTexture {
    sequential(base_id, 32, 0.05)
}

/// Lava: 40 frames, 0.075s each.
pub fn lava_animation(base_id: u32) -> AnimatedTexture {
    sequential(base_id, 40, 0.075)
}

/// Fire: 32 frames, 0.04s each.
pub fn fire_animation(base_id: u32) -> AnimatedTexture {
    sequential(base_id, 32, 0.04)
}

/// Nether portal: 32 frames, 0.05s each.
pub fn portal_animation(base_id: u32) -> AnimatedTexture {
    sequential(base_id, 32, 0.05)
}

/// Magma block: 8 frames, 0.1s each.
pub fn magma_animation(base_id: u32) -> AnimatedTexture {
    sequential(base_id, 8, 0.1)
}

/// Prismarine: 24 frames, 0.083s each.
pub fn prismarine_animation(base_id: u32) -> AnimatedTexture {
    sequential(base_id, 24, 0.083)
}

/// Seagrass: 8 frames, 0.0625s each.
pub fn seagrass_animation(base_id: u32) -> AnimatedTexture {
    sequential(base_id, 8, 0.0625)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_returns_first_frame_initially() {
        let mut anim = AnimatedTexture::new(vec![10, 11, 12, 13], 0.1);
        assert_eq!(anim.tick(0.0), 10);
    }

    #[test]
    fn tick_advances_through_frames() {
        let mut anim = AnimatedTexture::new(vec![10, 11, 12, 13], 0.1);
        assert_eq!(anim.tick(0.1), 11);
        assert_eq!(anim.tick(0.1), 12);
        assert_eq!(anim.tick(0.1), 13);
    }

    #[test]
    fn tick_wraps_around_cycle() {
        let mut anim = AnimatedTexture::new(vec![10, 11, 12, 13], 0.1);
        // total cycle = 0.4s; advancing 0.45s should wrap to 0.05s -> frame 0
        assert_eq!(anim.tick(0.45), 10);
        // advancing further past wrap
        assert_eq!(anim.tick(0.4), 10);
    }

    #[test]
    fn tick_empty_frames_returns_zero() {
        let mut anim = AnimatedTexture::new(vec![], 0.1);
        assert_eq!(anim.tick(0.5), 0);
    }

    #[test]
    fn interpolation_factor_between_frames() {
        let mut anim = AnimatedTexture::new(vec![100, 101, 102, 103], 0.1);
        anim.tick(0.05); // halfway into frame 0
        let (prev, next, lerp) = anim.interpolated_frame();
        assert_eq!(prev, 100);
        assert_eq!(next, 101);
        assert!((lerp - 0.5).abs() < 1e-5, "lerp was {lerp}");
    }

    #[test]
    fn interpolation_wraps_to_first_frame() {
        let mut anim = AnimatedTexture::new(vec![100, 101, 102, 103], 0.1);
        // jump near the end of last frame
        anim.tick(0.375); // position 3.75 -> prev 3, next 0, lerp 0.75
        let (prev, next, lerp) = anim.interpolated_frame();
        assert_eq!(prev, 103);
        assert_eq!(next, 100);
        assert!((lerp - 0.75).abs() < 1e-5, "lerp was {lerp}");
    }

    #[test]
    fn interpolation_factor_in_range() {
        let mut anim = AnimatedTexture::new(vec![1, 2, 3], 0.2);
        for step in 0..50 {
            anim.tick(0.013 * step as f32);
            let (_, _, lerp) = anim.interpolated_frame();
            assert!((0.0..1.0).contains(&lerp), "lerp out of range: {lerp}");
        }
    }

    #[test]
    fn water_animation_has_32_frames() {
        let anim = water_animation(1000);
        assert_eq!(anim.frames.len(), 32);
        assert_eq!(anim.frame_duration, 0.05);
        assert_eq!(anim.frames[0], 1000);
        assert_eq!(anim.frames[31], 1031);
    }

    #[test]
    fn lava_animation_has_40_frames() {
        let anim = lava_animation(2000);
        assert_eq!(anim.frames.len(), 40);
        assert_eq!(anim.frame_duration, 0.075);
        assert_eq!(anim.frames[0], 2000);
        assert_eq!(anim.frames[39], 2039);
    }

    #[test]
    fn fire_animation_has_32_frames() {
        let anim = fire_animation(3000);
        assert_eq!(anim.frames.len(), 32);
        assert_eq!(anim.frame_duration, 0.04);
    }

    #[test]
    fn portal_animation_has_32_frames() {
        let anim = portal_animation(4000);
        assert_eq!(anim.frames.len(), 32);
        assert_eq!(anim.frame_duration, 0.05);
    }

    #[test]
    fn magma_animation_has_8_frames() {
        let anim = magma_animation(5000);
        assert_eq!(anim.frames.len(), 8);
        assert_eq!(anim.frame_duration, 0.1);
    }

    #[test]
    fn prismarine_animation_has_24_frames() {
        let anim = prismarine_animation(6000);
        assert_eq!(anim.frames.len(), 24);
        assert_eq!(anim.frame_duration, 0.083);
    }

    #[test]
    fn seagrass_animation_has_8_frames() {
        let anim = seagrass_animation(7000);
        assert_eq!(anim.frames.len(), 8);
        assert_eq!(anim.frame_duration, 0.0625);
    }
}
