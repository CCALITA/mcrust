use glam::Vec3;

// ---------------------------------------------------------------------------
// Vehicle types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VehicleType {
    Minecart,
    Boat,
    ChestMinecart,
    HopperMinecart,
    TNTMinecart,
}

// ---------------------------------------------------------------------------
// Vehicle
// ---------------------------------------------------------------------------

/// A rideable vehicle entity (minecart or boat variant).
#[derive(Debug, Clone)]
pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub passenger: Option<u64>,
}

/// Maximum minecart speed in blocks per second.
const MINECART_MAX_SPEED: f32 = 8.0;

/// Friction coefficient for a minecart on rails.
const MINECART_RAIL_FRICTION: f32 = 0.005;

/// Friction coefficient for a minecart off rails.
const MINECART_OFF_RAIL_FRICTION: f32 = 0.5;

/// Friction coefficient for a boat in water.
const BOAT_WATER_FRICTION: f32 = 0.1;

/// Friction coefficient for a boat on land.
const BOAT_LAND_FRICTION: f32 = 0.9;

impl Vehicle {
    /// Create a new vehicle at the given position with zero velocity.
    pub fn new(vehicle_type: VehicleType, position: Vec3) -> Self {
        Self {
            vehicle_type,
            position,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            passenger: None,
        }
    }

    /// Attempt to mount an entity. Returns `false` if the seat is already occupied.
    pub fn mount(&mut self, entity_id: u64) -> bool {
        if self.passenger.is_some() {
            return false;
        }
        self.passenger = Some(entity_id);
        true
    }

    /// Dismount the current passenger, returning its entity id if one was riding.
    pub fn dismount(&mut self) -> Option<u64> {
        self.passenger.take()
    }
}

// ---------------------------------------------------------------------------
// Tick functions
// ---------------------------------------------------------------------------

/// Advance a minecart by one tick.
///
/// On a rail the minecart experiences very low friction (`0.005`) and its speed
/// is clamped to `MINECART_MAX_SPEED` (8 blocks/s). Off-rail friction is much
/// higher (`0.5`), bringing the minecart to a quick stop.
pub fn minecart_tick(vehicle: &mut Vehicle, on_rail: bool, dt: f32) {
    let friction = if on_rail {
        MINECART_RAIL_FRICTION
    } else {
        MINECART_OFF_RAIL_FRICTION
    };

    // Apply friction: scale velocity toward zero.
    let factor = (1.0 - friction).max(0.0);
    vehicle.velocity *= factor;

    // Clamp horizontal speed to max.
    let horizontal = Vec3::new(vehicle.velocity.x, 0.0, vehicle.velocity.z);
    let speed = horizontal.length();
    if speed > MINECART_MAX_SPEED {
        let scale = MINECART_MAX_SPEED / speed;
        vehicle.velocity.x *= scale;
        vehicle.velocity.z *= scale;
    }

    vehicle.position += vehicle.velocity * dt;
}

/// Advance a boat by one tick.
///
/// In water the boat floats (vertical velocity decays toward zero) and
/// experiences moderate friction (`0.1`). On land the friction is very high
/// (`0.9`) so the boat quickly decelerates.
pub fn boat_tick(vehicle: &mut Vehicle, in_water: bool, dt: f32) {
    let friction = if in_water {
        BOAT_WATER_FRICTION
    } else {
        BOAT_LAND_FRICTION
    };

    // In water, float: push vertical velocity toward zero.
    if in_water {
        vehicle.velocity.y *= 0.5;
    }

    let factor = (1.0 - friction).max(0.0);
    vehicle.velocity *= factor;

    vehicle.position += vehicle.velocity * dt;
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

/// Apply player input to a vehicle, accelerating in the facing direction.
///
/// `forward` is the throttle along the vehicle's yaw and `strafe` is
/// lateral movement. Both are typically in `[-1.0, 1.0]`.
pub fn apply_input(vehicle: &mut Vehicle, forward: f32, strafe: f32, dt: f32) {
    let (sin_yaw, cos_yaw) = vehicle.yaw.sin_cos();

    // Forward direction is along +Z rotated by yaw.
    let forward_dir = Vec3::new(-sin_yaw, 0.0, cos_yaw);
    let strafe_dir = Vec3::new(cos_yaw, 0.0, sin_yaw);

    let acceleration = (forward_dir * forward + strafe_dir * strafe) * dt;
    vehicle.velocity += acceleration;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Mount / Dismount ---------------------------------------------------

    #[test]
    fn mount_succeeds_when_empty() {
        let mut v = Vehicle::new(VehicleType::Minecart, Vec3::ZERO);
        assert!(v.mount(1));
        assert_eq!(v.passenger, Some(1));
    }

    #[test]
    fn mount_fails_when_occupied() {
        let mut v = Vehicle::new(VehicleType::Boat, Vec3::ZERO);
        assert!(v.mount(1));
        assert!(!v.mount(2));
        assert_eq!(v.passenger, Some(1));
    }

    #[test]
    fn dismount_returns_passenger() {
        let mut v = Vehicle::new(VehicleType::Minecart, Vec3::ZERO);
        v.mount(42);
        assert_eq!(v.dismount(), Some(42));
        assert_eq!(v.passenger, None);
    }

    #[test]
    fn dismount_returns_none_when_empty() {
        let mut v = Vehicle::new(VehicleType::Boat, Vec3::ZERO);
        assert_eq!(v.dismount(), None);
    }

    // -- Minecart rail friction ---------------------------------------------

    #[test]
    fn rail_reduces_friction_compared_to_off_rail() {
        let mut on_rail = Vehicle::new(VehicleType::Minecart, Vec3::ZERO);
        on_rail.velocity = Vec3::new(5.0, 0.0, 0.0);

        let mut off_rail = on_rail.clone();

        minecart_tick(&mut on_rail, true, 1.0);
        minecart_tick(&mut off_rail, false, 1.0);

        // On-rail vehicle should retain more speed.
        assert!(
            on_rail.velocity.x.abs() > off_rail.velocity.x.abs(),
            "on_rail speed {} should exceed off_rail speed {}",
            on_rail.velocity.x.abs(),
            off_rail.velocity.x.abs(),
        );
    }

    // -- Minecart speed limit -----------------------------------------------

    #[test]
    fn minecart_speed_clamped_to_max() {
        let mut v = Vehicle::new(VehicleType::Minecart, Vec3::ZERO);
        v.velocity = Vec3::new(20.0, 0.0, 0.0);

        minecart_tick(&mut v, true, 0.05);

        let speed = Vec3::new(v.velocity.x, 0.0, v.velocity.z).length();
        assert!(
            speed <= MINECART_MAX_SPEED + f32::EPSILON,
            "speed {} should not exceed {}",
            speed,
            MINECART_MAX_SPEED,
        );
    }

    // -- Boat floating ------------------------------------------------------

    #[test]
    fn boat_floats_in_water() {
        let mut v = Vehicle::new(VehicleType::Boat, Vec3::new(0.0, 5.0, 0.0));
        v.velocity = Vec3::new(0.0, -2.0, 0.0);

        boat_tick(&mut v, true, 0.05);

        // Vertical velocity should be damped toward zero.
        assert!(
            v.velocity.y.abs() < 2.0,
            "vertical velocity {} should be damped",
            v.velocity.y,
        );
    }

    #[test]
    fn boat_on_land_decelerates_fast() {
        let mut water = Vehicle::new(VehicleType::Boat, Vec3::ZERO);
        water.velocity = Vec3::new(5.0, 0.0, 0.0);

        let mut land = water.clone();

        boat_tick(&mut water, true, 1.0);
        boat_tick(&mut land, false, 1.0);

        assert!(
            land.velocity.x.abs() < water.velocity.x.abs(),
            "land speed {} should be less than water speed {}",
            land.velocity.x.abs(),
            water.velocity.x.abs(),
        );
    }

    // -- Input acceleration -------------------------------------------------

    #[test]
    fn input_accelerates_in_facing_direction() {
        let mut v = Vehicle::new(VehicleType::Boat, Vec3::ZERO);
        v.yaw = 0.0; // facing +Z

        apply_input(&mut v, 1.0, 0.0, 1.0);

        assert!(
            v.velocity.z > 0.0,
            "forward input at yaw=0 should accelerate along +Z, got {}",
            v.velocity.z,
        );
    }

    #[test]
    fn input_with_strafe_adds_lateral_velocity() {
        let mut v = Vehicle::new(VehicleType::Minecart, Vec3::ZERO);
        v.yaw = 0.0;

        apply_input(&mut v, 0.0, 1.0, 1.0);

        // Strafe at yaw=0 should add velocity along +X.
        assert!(
            v.velocity.x > 0.0,
            "strafe input at yaw=0 should add +X velocity, got {}",
            v.velocity.x,
        );
    }

    // -- Vehicle construction -----------------------------------------------

    #[test]
    fn new_vehicle_has_zero_velocity_and_no_passenger() {
        let v = Vehicle::new(VehicleType::ChestMinecart, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.velocity, Vec3::ZERO);
        assert_eq!(v.passenger, None);
        assert_eq!(v.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.vehicle_type, VehicleType::ChestMinecart);
    }

    #[test]
    fn all_vehicle_types_constructible() {
        for vtype in [
            VehicleType::Minecart,
            VehicleType::Boat,
            VehicleType::ChestMinecart,
            VehicleType::HopperMinecart,
            VehicleType::TNTMinecart,
        ] {
            let v = Vehicle::new(vtype, Vec3::ZERO);
            assert_eq!(v.vehicle_type, vtype);
        }
    }
}
