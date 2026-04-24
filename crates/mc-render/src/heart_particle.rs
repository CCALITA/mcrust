//! Specialized heart particle effects for breeding and happy villager events.

/// A heart-shaped particle with position, aging, and color.
#[derive(Debug, Clone)]
pub struct HeartParticle {
    pub pos: [f32; 3],
    pub age: f32,
    pub lifetime: f32,
    pub color: [f32; 3],
}

/// Spawn red heart particles around a breeding entity.
///
/// Returns `count` hearts scattered around `pos` with a red tint.
pub fn spawn_breeding_hearts(pos: [f32; 3], count: u8) -> Vec<HeartParticle> {
    (0..count)
        .map(|i| {
            let offset = offset_for_index(pos, i);
            HeartParticle {
                pos: [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]],
                age: 0.0,
                lifetime: 1.5,
                color: [1.0, 0.0, 0.0],
            }
        })
        .collect()
}

/// Spawn green heart particles for a happy villager.
///
/// Returns a fixed set of 5 green hearts scattered around `pos`.
pub fn spawn_villager_happy(pos: [f32; 3]) -> Vec<HeartParticle> {
    (0..5)
        .map(|i| {
            let offset = offset_for_index(pos, i);
            HeartParticle {
                pos: [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]],
                age: 0.0,
                lifetime: 1.2,
                color: [0.0, 1.0, 0.0],
            }
        })
        .collect()
}

/// Advance a heart particle by `dt` seconds. Returns `false` when expired.
pub fn tick_heart(p: &mut HeartParticle, dt: f32) -> bool {
    p.age += dt;
    // Float upward over time
    p.pos[1] += 0.5 * dt;
    p.age < p.lifetime
}

/// Compute a pulsing size factor based on age and lifetime.
///
/// The heart grows quickly at spawn, oscillates with a sine pulse,
/// and shrinks toward the end of its lifetime.
pub fn heart_size(age: f32, lifetime: f32) -> f32 {
    if lifetime <= 0.0 {
        return 0.0;
    }
    let t = age / lifetime;
    if t >= 1.0 {
        return 0.0;
    }
    // Base scale: ramp up then fade out
    let base = if t < 0.1 {
        // Quick grow-in over first 10%
        t / 0.1
    } else if t > 0.8 {
        // Shrink over last 20%
        (1.0 - t) / 0.2
    } else {
        1.0
    };
    // Sine pulse: two full beats over the lifetime
    let pulse = 1.0 + 0.15 * (t * std::f32::consts::TAU * 2.0).sin();
    base * pulse
}

/// Deterministic offset from position and index, avoiding the `rand` crate.
fn offset_for_index(pos: [f32; 3], index: u8) -> [f32; 3] {
    let hash = hash_pos_index(pos, index);
    let x = ((hash & 0xFF) as f32 / 127.5 - 1.0) * 0.4;
    let y = (((hash >> 8) & 0xFF) as f32 / 255.0) * 0.6;
    let z = (((hash >> 16) & 0xFF) as f32 / 127.5 - 1.0) * 0.4;
    [x, y, z]
}

/// Simple integer hash mixing position bits and an index for deterministic randomness.
fn hash_pos_index(pos: [f32; 3], index: u8) -> u32 {
    let xb = pos[0].to_bits();
    let yb = pos[1].to_bits();
    let zb = pos[2].to_bits();

    let mut h = xb.wrapping_mul(73_856_093)
        ^ yb.wrapping_mul(19_349_663)
        ^ zb.wrapping_mul(83_492_791)
        ^ (index as u32).wrapping_mul(49_979_693);

    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breeding_hearts_returns_correct_count() {
        let hearts = spawn_breeding_hearts([0.0, 64.0, 0.0], 7);
        assert_eq!(hearts.len(), 7);
    }

    #[test]
    fn breeding_hearts_are_red() {
        let hearts = spawn_breeding_hearts([1.0, 2.0, 3.0], 3);
        for h in &hearts {
            assert_eq!(h.color, [1.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn breeding_hearts_start_at_age_zero() {
        let hearts = spawn_breeding_hearts([0.0, 0.0, 0.0], 4);
        for h in &hearts {
            assert!((h.age).abs() < 1e-6);
        }
    }

    #[test]
    fn villager_happy_returns_five() {
        let hearts = spawn_villager_happy([5.0, 70.0, 5.0]);
        assert_eq!(hearts.len(), 5);
    }

    #[test]
    fn villager_happy_hearts_are_green() {
        let hearts = spawn_villager_happy([0.0, 0.0, 0.0]);
        for h in &hearts {
            assert_eq!(h.color, [0.0, 1.0, 0.0]);
        }
    }

    #[test]
    fn tick_heart_advances_age() {
        let mut h = HeartParticle {
            pos: [0.0, 0.0, 0.0],
            age: 0.0,
            lifetime: 2.0,
            color: [1.0, 0.0, 0.0],
        };
        let alive = tick_heart(&mut h, 0.5);
        assert!(alive);
        assert!((h.age - 0.5).abs() < 1e-6);
    }

    #[test]
    fn tick_heart_moves_upward() {
        let mut h = HeartParticle {
            pos: [0.0, 10.0, 0.0],
            age: 0.0,
            lifetime: 2.0,
            color: [1.0, 0.0, 0.0],
        };
        tick_heart(&mut h, 1.0);
        assert!(h.pos[1] > 10.0, "heart should float upward");
    }

    #[test]
    fn tick_heart_returns_false_when_expired() {
        let mut h = HeartParticle {
            pos: [0.0, 0.0, 0.0],
            age: 0.0,
            lifetime: 1.0,
            color: [1.0, 0.0, 0.0],
        };
        // Advance past lifetime
        let alive = tick_heart(&mut h, 1.5);
        assert!(!alive);
    }

    #[test]
    fn heart_size_is_zero_at_or_past_lifetime() {
        assert!((heart_size(1.0, 1.0)).abs() < 1e-6);
        assert!((heart_size(2.0, 1.0)).abs() < 1e-6);
    }

    #[test]
    fn heart_size_is_zero_with_zero_lifetime() {
        assert!((heart_size(0.5, 0.0)).abs() < 1e-6);
    }

    #[test]
    fn heart_size_grows_in_first_phase() {
        let early = heart_size(0.05, 1.0);
        let mid = heart_size(0.5, 1.0);
        assert!(
            early < mid,
            "heart should still be growing early: early={early}, mid={mid}"
        );
    }

    #[test]
    fn heart_size_shrinks_in_last_phase() {
        let at_80 = heart_size(0.5, 1.0);
        let at_95 = heart_size(0.95, 1.0);
        assert!(
            at_95 < at_80,
            "heart should shrink near end: at_80={at_80}, at_95={at_95}"
        );
    }

    #[test]
    fn heart_size_has_pulse() {
        // Sample several points in the middle phase and check they are not all equal
        let sizes: Vec<f32> = (20..80)
            .map(|i| heart_size(i as f32 / 100.0, 1.0))
            .collect();
        let min = sizes.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = sizes.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max - min > 0.01,
            "pulse should cause variation: min={min}, max={max}"
        );
    }

    #[test]
    fn zero_count_breeding_returns_empty() {
        let hearts = spawn_breeding_hearts([0.0, 0.0, 0.0], 0);
        assert!(hearts.is_empty());
    }

    #[test]
    fn offsets_are_deterministic() {
        let a = offset_for_index([1.0, 2.0, 3.0], 0);
        let b = offset_for_index([1.0, 2.0, 3.0], 0);
        assert_eq!(a, b);
    }

    #[test]
    fn offsets_differ_per_index() {
        let a = offset_for_index([1.0, 2.0, 3.0], 0);
        let b = offset_for_index([1.0, 2.0, 3.0], 1);
        let dist = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        assert!(dist > 0.001, "offsets should differ: a={a:?}, b={b:?}");
    }
}
