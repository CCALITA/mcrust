//! End gateway block: teleportation, beam rendering, and cooldown logic.

/// An end gateway block that teleports entities between locations.
#[derive(Debug, Clone, PartialEq)]
pub struct EndGateway {
    pub pos: (i32, i32, i32),
    pub target: (i32, i32, i32),
    pub age: u64,
    pub exact_teleport: bool,
}

impl EndGateway {
    /// Creates a new end gateway at `pos` targeting `target`.
    pub fn new(pos: (i32, i32, i32), target: (i32, i32, i32)) -> Self {
        Self {
            pos,
            target,
            age: 0,
            exact_teleport: false,
        }
    }
}

/// Computes the teleport destination for an entity entering the gateway.
///
/// If `exact_teleport` is set, the entity lands exactly at the target.
/// Otherwise, the entity's offset from the gateway centre is preserved.
pub fn gateway_teleport_destination(gw: &EndGateway, entity_pos: [f32; 3]) -> [f32; 3] {
    if gw.exact_teleport {
        [
            gw.target.0 as f32 + 0.5,
            gw.target.1 as f32,
            gw.target.2 as f32 + 0.5,
        ]
    } else {
        let offset = [
            entity_pos[0] - (gw.pos.0 as f32 + 0.5),
            entity_pos[1] - gw.pos.1 as f32,
            entity_pos[2] - (gw.pos.2 as f32 + 0.5),
        ];
        [
            gw.target.0 as f32 + 0.5 + offset[0],
            gw.target.1 as f32 + offset[1],
            gw.target.2 as f32 + 0.5 + offset[2],
        ]
    }
}

/// Returns the beam colour based on gateway age.
///
/// Young gateways (age < 200) emit a magenta beam; older ones emit yellow.
pub fn gateway_beam_color(age: u64) -> [f32; 3] {
    if age < 200 {
        [0.6, 0.0, 0.6] // magenta
    } else {
        [1.0, 1.0, 0.0] // yellow
    }
}

/// Returns the gateway cooldown in ticks (40 ticks = 2 seconds).
pub fn gateway_cooldown() -> u32 {
    40
}

/// Returns the target position for the first end gateway generated when
/// the ender dragon is defeated: the centre End island at (0, 75, 0).
pub fn first_gateway_target() -> (i32, i32, i32) {
    (0, 75, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gateway_defaults() {
        let gw = EndGateway::new((100, 50, 100), (0, 75, 0));
        assert_eq!(gw.pos, (100, 50, 100));
        assert_eq!(gw.target, (0, 75, 0));
        assert_eq!(gw.age, 0);
        assert!(!gw.exact_teleport);
    }

    #[test]
    fn exact_teleport_lands_at_target_centre() {
        let gw = EndGateway {
            pos: (100, 50, 100),
            target: (0, 75, 0),
            age: 0,
            exact_teleport: true,
        };
        let dest = gateway_teleport_destination(&gw, [100.5, 50.0, 100.5]);
        assert!((dest[0] - 0.5).abs() < 1e-5);
        assert!((dest[1] - 75.0).abs() < 1e-5);
        assert!((dest[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn relative_teleport_preserves_offset() {
        let gw = EndGateway::new((100, 50, 100), (0, 75, 0));
        // Entity is 1 block east and 2 blocks above gateway centre
        let dest = gateway_teleport_destination(&gw, [101.5, 52.0, 100.5]);
        assert!((dest[0] - 1.5).abs() < 1e-5);
        assert!((dest[1] - 77.0).abs() < 1e-5);
        assert!((dest[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn beam_color_young_gateway() {
        let color = gateway_beam_color(0);
        assert_eq!(color, [0.6, 0.0, 0.6]);
    }

    #[test]
    fn beam_color_old_gateway() {
        let color = gateway_beam_color(200);
        assert_eq!(color, [1.0, 1.0, 0.0]);
    }

    #[test]
    fn cooldown_is_40_ticks() {
        assert_eq!(gateway_cooldown(), 40);
    }

    #[test]
    fn first_target_is_centre_island() {
        assert_eq!(first_gateway_target(), (0, 75, 0));
    }
}
