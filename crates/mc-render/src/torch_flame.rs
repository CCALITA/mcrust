//! Torch flame animation: flicker, brightness, color, and smoke particles.

/// Represents a torch flame with position, type, and flicker state.
pub struct TorchFlame {
    pub pos: [f32; 3],
    pub soul: bool,
    pub flicker_seed: u64,
}

/// Returns a tiny xy jitter offset (±0.02) for flame animation.
pub fn flame_offset(time: f32, seed: u64) -> [f32; 2] {
    let s = seed as f32;
    let x = (time * 3.7 + s * 0.13).sin() * 0.02;
    let y = (time * 4.3 + s * 0.17).cos() * 0.02;
    [x, y]
}

/// Returns flame brightness oscillating between 0.85 and 1.0.
pub fn flame_brightness(time: f32, seed: u64) -> f32 {
    let s = seed as f32;
    let t = (time * 5.0 + s * 0.11).sin();
    0.925 + t * 0.075
}

/// Returns the color for a soul torch (blue tint).
pub fn soul_torch_color() -> [f32; 3] {
    [0.3, 0.7, 1.0]
}

/// Returns the color for a regular torch (warm orange).
pub fn regular_torch_color() -> [f32; 3] {
    [1.0, 0.7, 0.3]
}

/// Returns a position slightly above the torch for smoke particles.
pub fn smoke_particle_above_torch(pos: [f32; 3]) -> [f32; 3] {
    [pos[0], pos[1] + 0.15, pos[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flame_offset_within_bounds() {
        for t in 0..100 {
            let time = t as f32 * 0.1;
            let [x, y] = flame_offset(time, 42);
            assert!(x.abs() <= 0.02, "x={x} out of range");
            assert!(y.abs() <= 0.02, "y={y} out of range");
        }
    }

    #[test]
    fn flame_brightness_within_range() {
        for t in 0..100 {
            let time = t as f32 * 0.1;
            let b = flame_brightness(time, 99);
            assert!(b >= 0.85, "brightness {b} below 0.85");
            assert!(b <= 1.0, "brightness {b} above 1.0");
        }
    }

    #[test]
    fn soul_torch_color_is_blue() {
        let c = soul_torch_color();
        assert_eq!(c, [0.3, 0.7, 1.0]);
    }

    #[test]
    fn regular_torch_color_is_orange() {
        let c = regular_torch_color();
        assert_eq!(c, [1.0, 0.7, 0.3]);
    }

    #[test]
    fn smoke_particle_is_above_torch() {
        let pos = [1.0, 2.0, 3.0];
        let smoke = smoke_particle_above_torch(pos);
        assert_eq!(smoke[0], pos[0]);
        assert!(smoke[1] > pos[1]);
        assert_eq!(smoke[2], pos[2]);
    }

    #[test]
    fn different_seeds_produce_different_offsets() {
        let a = flame_offset(1.0, 1);
        let b = flame_offset(1.0, 9999);
        assert_ne!(a, b);
    }
}
