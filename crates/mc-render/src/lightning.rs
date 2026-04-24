//! Lightning bolt visual effect: zigzag path generation, lifetime ticking,
//! and brightness/color curves for the flash animation.

/// Default lightning bolt lifetime in seconds.
pub const LIGHTNING_DEFAULT_LIFETIME: f32 = 0.5;

/// Minimum number of segments in a generated bolt (inclusive).
const MIN_SEGMENTS: usize = 12;
/// Maximum number of segments in a generated bolt (inclusive).
const MAX_SEGMENTS: usize = 20;
/// Jitter magnitude in blocks applied perpendicular to the main direction.
const JITTER_MAGNITUDE: f32 = 0.6;

/// A single lightning bolt: a polyline of world-space points with a lifetime.
#[derive(Debug, Clone, PartialEq)]
pub struct LightningBolt {
    /// Ordered points forming the zigzag path from start to end.
    pub segments: Vec<[f32; 3]>,
    /// Time elapsed since the bolt was spawned (seconds).
    pub elapsed: f32,
    /// Total lifetime of the bolt (seconds).
    pub lifetime: f32,
}

/// Simple splitmix64-style hash for seed-based deterministic jitter.
fn hash_u64(seed: u64, index: u64) -> u64 {
    let mut x = seed.wrapping_add(index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Maps a u64 hash to a float in `[-1.0, 1.0]`.
fn hash_to_unit(seed: u64, index: u64) -> f32 {
    let h = hash_u64(seed, index);
    // Use top 24 bits for float mantissa precision
    let normalized = (h >> 40) as f32 / ((1u64 << 24) as f32);
    normalized * 2.0 - 1.0
}

/// Generates a zigzag lightning bolt path from `start` to `end`.
///
/// The number of segments is seeded-deterministic in the range
/// `[MIN_SEGMENTS, MAX_SEGMENTS]`. Intermediate points are jittered along two
/// perpendicular axes orthogonal to the main direction.
pub fn generate_lightning_bolt(start: [f32; 3], end: [f32; 3], seed: u64) -> LightningBolt {
    let segment_count = MIN_SEGMENTS
        + (hash_u64(seed, 0) as usize % (MAX_SEGMENTS - MIN_SEGMENTS + 1));

    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let dz = end[2] - start[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();

    // Build two perpendicular axes to the main direction (dx, dy, dz).
    let (perp_a, perp_b) = if len > 1e-6 {
        let main = [dx / len, dy / len, dz / len];
        // Pick a reference vector not parallel to `main`.
        let reference = if main[1].abs() < 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        // perp_a = normalize(main x reference)
        let a = [
            main[1] * reference[2] - main[2] * reference[1],
            main[2] * reference[0] - main[0] * reference[2],
            main[0] * reference[1] - main[1] * reference[0],
        ];
        let a_len = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-6);
        let a_n = [a[0] / a_len, a[1] / a_len, a[2] / a_len];
        // perp_b = main x perp_a
        let b = [
            main[1] * a_n[2] - main[2] * a_n[1],
            main[2] * a_n[0] - main[0] * a_n[2],
            main[0] * a_n[1] - main[1] * a_n[0],
        ];
        (a_n, b)
    } else {
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0])
    };

    let mut segments = Vec::with_capacity(segment_count + 1);
    segments.push(start);

    // Intermediate points: 1..segment_count (endpoints fixed).
    for i in 1..segment_count {
        let t = i as f32 / segment_count as f32;
        let base = [
            start[0] + dx * t,
            start[1] + dy * t,
            start[2] + dz * t,
        ];
        let ja = hash_to_unit(seed, (i as u64) * 2 + 1) * JITTER_MAGNITUDE;
        let jb = hash_to_unit(seed, (i as u64) * 2 + 2) * JITTER_MAGNITUDE;
        segments.push([
            base[0] + perp_a[0] * ja + perp_b[0] * jb,
            base[1] + perp_a[1] * ja + perp_b[1] * jb,
            base[2] + perp_a[2] * ja + perp_b[2] * jb,
        ]);
    }

    segments.push(end);

    LightningBolt {
        segments,
        elapsed: 0.0,
        lifetime: LIGHTNING_DEFAULT_LIFETIME,
    }
}

impl LightningBolt {
    /// Advances the bolt by `dt` seconds. Returns `false` when the bolt has expired.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        self.elapsed < self.lifetime
    }
}

/// Brightness curve: `1.0` at `elapsed == 0`, linearly fading to `0.0` at `elapsed == lifetime`.
pub fn lightning_brightness(elapsed: f32, lifetime: f32) -> f32 {
    if lifetime <= 0.0 {
        return 0.0;
    }
    let t = (elapsed / lifetime).clamp(0.0, 1.0);
    1.0 - t
}

/// Color shift from bright white `[1.0, 1.0, 1.0]` to blue-white `[0.7, 0.7, 1.0]`.
///
/// Interpolation parameter is `elapsed / LIGHTNING_DEFAULT_LIFETIME`, clamped to `[0, 1]`.
pub fn lightning_color(elapsed: f32) -> [f32; 3] {
    let t = (elapsed / LIGHTNING_DEFAULT_LIFETIME).clamp(0.0, 1.0);
    let r = 1.0 + (0.7 - 1.0) * t;
    let g = 1.0 + (0.7 - 1.0) * t;
    let b = 1.0;
    [r, g, b]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_count_in_range() {
        for seed in 0..64u64 {
            let bolt = generate_lightning_bolt([0.0, 0.0, 0.0], [0.0, 10.0, 0.0], seed);
            // segments has segment_count + 1 points (endpoints + intermediates)
            let count = bolt.segments.len();
            assert!(
                count >= MIN_SEGMENTS + 1 && count <= MAX_SEGMENTS + 1,
                "segment count {count} out of range for seed {seed}"
            );
        }
    }

    #[test]
    fn endpoints_preserved() {
        let start = [1.0, 2.0, 3.0];
        let end = [4.0, 20.0, 5.0];
        let bolt = generate_lightning_bolt(start, end, 42);
        assert_eq!(bolt.segments.first().copied(), Some(start));
        assert_eq!(bolt.segments.last().copied(), Some(end));
    }

    #[test]
    fn deterministic_for_same_seed() {
        let a = generate_lightning_bolt([0.0, 0.0, 0.0], [0.0, 10.0, 0.0], 123);
        let b = generate_lightning_bolt([0.0, 0.0, 0.0], [0.0, 10.0, 0.0], 123);
        assert_eq!(a.segments, b.segments);
    }

    #[test]
    fn different_seeds_produce_different_paths() {
        let a = generate_lightning_bolt([0.0, 0.0, 0.0], [0.0, 10.0, 0.0], 1);
        let b = generate_lightning_bolt([0.0, 0.0, 0.0], [0.0, 10.0, 0.0], 2);
        assert_ne!(a.segments, b.segments);
    }

    #[test]
    fn tick_returns_false_when_done() {
        let mut bolt = generate_lightning_bolt([0.0, 0.0, 0.0], [0.0, 10.0, 0.0], 0);
        assert!(bolt.tick(0.1));
        assert!(bolt.tick(0.1));
        // After elapsed >= lifetime (0.5), tick returns false.
        bolt.tick(0.5);
        assert!(!bolt.tick(0.01));
    }

    #[test]
    fn brightness_curve_starts_at_one_ends_at_zero() {
        assert!((lightning_brightness(0.0, 0.5) - 1.0).abs() < 1e-6);
        assert!(lightning_brightness(0.5, 0.5).abs() < 1e-6);
        // Monotonically decreasing
        let mid = lightning_brightness(0.25, 0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn brightness_handles_zero_lifetime() {
        assert_eq!(lightning_brightness(0.1, 0.0), 0.0);
    }

    #[test]
    fn color_starts_white_ends_blue_white() {
        let start = lightning_color(0.0);
        assert!((start[0] - 1.0).abs() < 1e-6);
        assert!((start[1] - 1.0).abs() < 1e-6);
        assert!((start[2] - 1.0).abs() < 1e-6);

        let end = lightning_color(LIGHTNING_DEFAULT_LIFETIME);
        assert!((end[0] - 0.7).abs() < 1e-6);
        assert!((end[1] - 0.7).abs() < 1e-6);
        assert!((end[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color_shifts_toward_blue() {
        let mid = lightning_color(LIGHTNING_DEFAULT_LIFETIME * 0.5);
        // Red and green have dropped below 1.0, blue stays at 1.0
        assert!(mid[0] < 1.0 && mid[0] > 0.7);
        assert!(mid[1] < 1.0 && mid[1] > 0.7);
        assert!((mid[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_lifetime_is_half_second() {
        assert_eq!(LIGHTNING_DEFAULT_LIFETIME, 0.5);
    }

    #[test]
    fn jitter_stays_bounded() {
        // Intermediate points should be within JITTER_MAGNITUDE * sqrt(2) of the line.
        let start = [0.0, 0.0, 0.0];
        let end = [0.0, 100.0, 0.0];
        let bolt = generate_lightning_bolt(start, end, 7);
        let max_offset = JITTER_MAGNITUDE * std::f32::consts::SQRT_2 + 1e-4;
        for p in &bolt.segments[1..bolt.segments.len() - 1] {
            // Distance from Y axis
            let dist = (p[0] * p[0] + p[2] * p[2]).sqrt();
            assert!(dist <= max_offset, "jitter {dist} exceeds bound {max_offset}");
        }
    }
}
