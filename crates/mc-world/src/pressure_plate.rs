/// Pressure plate types and signal weight calculations.
///
/// Wood and stone plates produce binary signals, while iron (heavy) and gold
/// (light) weighted plates output an analogue signal proportional to the number
/// of entities or items on them.

/// The four pressure plate variants found in Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressurePlateType {
    Wood,
    Stone,
    Iron,
    Gold,
}

/// Maximum entity/item count for the iron (heavy) weighted pressure plate.
const IRON_MAX: u16 = 150;

/// Maximum entity/item count for the gold (light) weighted pressure plate.
const GOLD_MAX: u16 = 15;

/// Compute the redstone signal strength for a weighted pressure plate.
///
/// The signal scales linearly from 1 to 15 as `count` goes from 1 to `max`.
/// Returns 0 when `count` is 0 and caps at 15 when `count >= max`.
pub fn weighted_signal(count: u16, max: u16) -> u8 {
    if count == 0 || max == 0 {
        return 0;
    }
    let clamped = count.min(max) as u32;
    let signal = (clamped * 15 + max as u32 - 1) / max as u32;
    (signal as u8).min(15).max(1)
}

/// Calculate the redstone signal emitted by a pressure plate.
///
/// * **Wood** — activates with any entity or item; always outputs 15.
/// * **Stone** — activates only with mobs (`entity_count`); ignores items.
/// * **Iron** (heavy weighted) — scales with total entity + item count up to 150.
/// * **Gold** (light weighted) — scales with total entity + item count up to 15.
pub fn plate_activation(plate: PressurePlateType, entity_count: u8, item_count: u16) -> u8 {
    match plate {
        PressurePlateType::Wood => {
            if entity_count > 0 || item_count > 0 {
                15
            } else {
                0
            }
        }
        PressurePlateType::Stone => {
            if entity_count > 0 {
                15
            } else {
                0
            }
        }
        PressurePlateType::Iron => {
            let total = entity_count as u16 + item_count;
            weighted_signal(total, IRON_MAX)
        }
        PressurePlateType::Gold => {
            let total = entity_count as u16 + item_count;
            weighted_signal(total, GOLD_MAX)
        }
    }
}

/// Tick delay before a pressure plate deactivates after all entities leave.
///
/// Wooden plates have a longer cooldown (20 ticks / 1 second) while stone and
/// weighted plates deactivate after 10 ticks (0.5 seconds).
pub fn plate_deactivation_delay(plate: PressurePlateType) -> u32 {
    match plate {
        PressurePlateType::Wood => 20,
        PressurePlateType::Stone | PressurePlateType::Iron | PressurePlateType::Gold => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wood_activates_with_entity() {
        assert_eq!(plate_activation(PressurePlateType::Wood, 1, 0), 15);
    }

    #[test]
    fn wood_activates_with_item() {
        assert_eq!(plate_activation(PressurePlateType::Wood, 0, 1), 15);
    }

    #[test]
    fn wood_inactive_when_empty() {
        assert_eq!(plate_activation(PressurePlateType::Wood, 0, 0), 0);
    }

    #[test]
    fn stone_activates_with_entity_only() {
        assert_eq!(plate_activation(PressurePlateType::Stone, 1, 0), 15);
        assert_eq!(plate_activation(PressurePlateType::Stone, 0, 100), 0);
    }

    #[test]
    fn stone_inactive_when_empty() {
        assert_eq!(plate_activation(PressurePlateType::Stone, 0, 0), 0);
    }

    #[test]
    fn iron_weighted_scales() {
        assert_eq!(plate_activation(PressurePlateType::Iron, 0, 0), 0);
        assert_eq!(plate_activation(PressurePlateType::Iron, 1, 0), 1);
        assert_eq!(plate_activation(PressurePlateType::Iron, 0, 150), 15);
        assert_eq!(plate_activation(PressurePlateType::Iron, 0, 200), 15);
    }

    #[test]
    fn gold_weighted_scales() {
        assert_eq!(plate_activation(PressurePlateType::Gold, 0, 0), 0);
        assert_eq!(plate_activation(PressurePlateType::Gold, 1, 0), 1);
        assert_eq!(plate_activation(PressurePlateType::Gold, 0, 15), 15);
        assert_eq!(plate_activation(PressurePlateType::Gold, 0, 100), 15);
    }

    #[test]
    fn weighted_signal_zero_count() {
        assert_eq!(weighted_signal(0, 150), 0);
    }

    #[test]
    fn weighted_signal_zero_max() {
        assert_eq!(weighted_signal(5, 0), 0);
    }

    #[test]
    fn weighted_signal_exact_max() {
        assert_eq!(weighted_signal(150, 150), 15);
        assert_eq!(weighted_signal(15, 15), 15);
    }

    #[test]
    fn weighted_signal_one() {
        assert_eq!(weighted_signal(1, 150), 1);
        assert_eq!(weighted_signal(1, 15), 1);
    }

    #[test]
    fn weighted_signal_midpoint() {
        // 75/150 = 0.5 → ceil(7.5) = 8
        assert_eq!(weighted_signal(75, 150), 8);
    }

    #[test]
    fn deactivation_delay_wood() {
        assert_eq!(plate_deactivation_delay(PressurePlateType::Wood), 20);
    }

    #[test]
    fn deactivation_delay_stone() {
        assert_eq!(plate_deactivation_delay(PressurePlateType::Stone), 10);
    }

    #[test]
    fn deactivation_delay_iron() {
        assert_eq!(plate_deactivation_delay(PressurePlateType::Iron), 10);
    }

    #[test]
    fn deactivation_delay_gold() {
        assert_eq!(plate_deactivation_delay(PressurePlateType::Gold), 10);
    }
}
