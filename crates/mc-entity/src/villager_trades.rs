// ---------------------------------------------------------------------------
// Default trade tables for each villager profession and level
// ---------------------------------------------------------------------------

use crate::villager::{TradeOffer, VillagerProfession};

/// Shorthand: single-input trade.
fn trade(input: (u16, u8), output: (u16, u8), max_uses: u8, xp: u8) -> TradeOffer {
    TradeOffer { input1: input, input2: None, output, max_uses, current_uses: 0, xp_reward: xp }
}

/// Shorthand: two-input trade.
fn trade2(
    input1: (u16, u8),
    input2: (u16, u8),
    output: (u16, u8),
    max_uses: u8,
    xp: u8,
) -> TradeOffer {
    TradeOffer { input1, input2: Some(input2), output, max_uses, current_uses: 0, xp_reward: xp }
}

/// Returns the trades unlocked for a `profession` at the given `level`.
///
/// Each level typically provides 2-3 new offers. Item IDs follow the
/// conventions in `mc-core`.
pub fn default_trades(profession: VillagerProfession, level: u8) -> Vec<TradeOffer> {
    match (profession, level) {
        // -- Farmer ----------------------------------------------------------
        (VillagerProfession::Farmer, 1) => vec![
            trade((1, 20), (2, 1), 16, 2),   // 20 wheat -> 1 emerald
            trade((2, 1), (3, 6), 16, 1),    // 1 emerald -> 6 bread
        ],
        (VillagerProfession::Farmer, 2) => vec![
            trade((4, 22), (2, 1), 16, 5),   // 22 potatoes -> 1 emerald
            trade((5, 22), (2, 1), 16, 5),   // 22 carrots -> 1 emerald
        ],

        // -- Librarian -------------------------------------------------------
        (VillagerProfession::Librarian, 1) => vec![
            trade((10, 24), (2, 1), 16, 2),              // 24 paper -> 1 emerald
            trade2((2, 1), (11, 1), (12, 1), 12, 2),     // 1 emerald + 1 book -> 1 enchanted book
        ],
        (VillagerProfession::Librarian, 2) => vec![
            trade((2, 1), (13, 4), 16, 5),   // 1 emerald -> 4 glass
            trade((11, 4), (2, 1), 12, 5),   // 4 books -> 1 emerald
        ],

        // -- Cleric ----------------------------------------------------------
        (VillagerProfession::Cleric, 1) => vec![
            trade((20, 32), (2, 1), 16, 2),  // 32 rotten flesh -> 1 emerald
            trade((2, 1), (21, 2), 16, 1),   // 1 emerald -> 2 redstone
        ],
        (VillagerProfession::Cleric, 2) => vec![
            trade((22, 3), (2, 1), 12, 5),   // 3 gold ingots -> 1 emerald
            trade((2, 1), (23, 1), 12, 5),   // 1 emerald -> 1 lapis lazuli
        ],

        // -- Armorer ---------------------------------------------------------
        (VillagerProfession::Armorer, 1) => vec![
            trade((30, 15), (2, 1), 16, 2),  // 15 coal -> 1 emerald
            trade((31, 7), (2, 1), 12, 2),   // 7 iron ingots -> 1 emerald
        ],
        (VillagerProfession::Armorer, 2) => vec![
            trade((2, 3), (32, 1), 12, 5),   // 3 emeralds -> 1 iron chestplate
            trade((2, 1), (33, 1), 12, 5),   // 1 emerald -> 1 iron helmet
        ],

        // -- Weaponsmith -----------------------------------------------------
        (VillagerProfession::Weaponsmith, 1) => vec![
            trade((30, 15), (2, 1), 16, 2),  // 15 coal -> 1 emerald
            trade((2, 3), (40, 1), 12, 2),   // 3 emeralds -> 1 iron axe
        ],
        (VillagerProfession::Weaponsmith, 2) => vec![
            trade((31, 4), (2, 1), 12, 5),   // 4 iron ingots -> 1 emerald
            trade((2, 2), (41, 1), 12, 5),   // 2 emeralds -> 1 iron sword
        ],

        // -- Toolsmith -------------------------------------------------------
        (VillagerProfession::Toolsmith, 1) => vec![
            trade((30, 15), (2, 1), 16, 2),  // 15 coal -> 1 emerald
            trade((2, 1), (50, 1), 12, 1),   // 1 emerald -> 1 stone axe
        ],
        (VillagerProfession::Toolsmith, 2) => vec![
            trade((31, 4), (2, 1), 12, 5),   // 4 iron ingots -> 1 emerald
            trade((2, 3), (51, 1), 12, 5),   // 3 emeralds -> 1 iron pickaxe
        ],

        // -- Butcher ---------------------------------------------------------
        (VillagerProfession::Butcher, 1) => vec![
            trade((60, 14), (2, 1), 16, 2),  // 14 raw chicken -> 1 emerald
            trade((61, 7), (2, 1), 16, 2),   // 7 raw porkchop -> 1 emerald
            trade((2, 1), (62, 5), 16, 1),   // 1 emerald -> 5 cooked porkchop
        ],
        (VillagerProfession::Butcher, 2) => vec![
            trade((30, 15), (2, 1), 16, 5),  // 15 coal -> 1 emerald
            trade((2, 1), (63, 8), 16, 5),   // 1 emerald -> 8 cooked chicken
        ],

        // -- Leatherworker ---------------------------------------------------
        (VillagerProfession::Leatherworker, 1) => vec![
            trade((70, 6), (2, 1), 16, 2),   // 6 leather -> 1 emerald
            trade((2, 3), (71, 1), 12, 2),   // 3 emeralds -> 1 leather pants
        ],
        (VillagerProfession::Leatherworker, 2) => vec![
            trade((70, 4), (2, 1), 16, 5),   // 4 leather -> 1 emerald
            trade((2, 7), (72, 1), 12, 5),   // 7 emeralds -> 1 leather tunic
        ],

        // -- Fletcher --------------------------------------------------------
        (VillagerProfession::Fletcher, 1) => vec![
            trade((80, 32), (2, 1), 16, 2),  // 32 sticks -> 1 emerald
            trade((2, 1), (81, 16), 12, 1),  // 1 emerald -> 16 arrows
        ],
        (VillagerProfession::Fletcher, 2) => vec![
            trade((82, 26), (2, 1), 12, 5),  // 26 flint -> 1 emerald
            trade((2, 2), (83, 1), 12, 5),   // 2 emeralds -> 1 bow
        ],

        // -- Cartographer ----------------------------------------------------
        (VillagerProfession::Cartographer, 1) => vec![
            trade((10, 24), (2, 1), 16, 2),  // 24 paper -> 1 emerald
            trade((2, 7), (90, 1), 12, 2),   // 7 emeralds -> 1 empty map
        ],
        (VillagerProfession::Cartographer, 2) => vec![
            trade((91, 11), (2, 1), 16, 5),              // 11 glass panes -> 1 emerald
            trade2((2, 13), (92, 1), (93, 1), 12, 5),    // 13 emeralds + 1 compass -> 1 ocean map
        ],

        // -- Mason -----------------------------------------------------------
        (VillagerProfession::Mason, 1) => vec![
            trade((100, 10), (2, 1), 16, 2),  // 10 clay balls -> 1 emerald
            trade((2, 1), (101, 10), 16, 1),  // 1 emerald -> 10 bricks
        ],
        (VillagerProfession::Mason, 2) => vec![
            trade((102, 20), (2, 1), 16, 5),  // 20 stone -> 1 emerald
            trade((2, 1), (103, 4), 16, 5),   // 1 emerald -> 4 chiseled stone bricks
        ],

        // -- Shepherd --------------------------------------------------------
        (VillagerProfession::Shepherd, 1) => vec![
            trade((110, 18), (2, 1), 16, 2),  // 18 white wool -> 1 emerald
            trade((2, 2), (111, 1), 12, 1),   // 2 emeralds -> 1 shears
        ],
        (VillagerProfession::Shepherd, 2) => vec![
            trade((112, 12), (2, 1), 16, 5),  // 12 black dye -> 1 emerald
            trade((2, 1), (113, 3), 16, 5),   // 1 emerald -> 3 colored wool
        ],

        // -- Nitwit (no trades at any level) ---------------------------------
        (VillagerProfession::Nitwit, _) => vec![],

        // -- Fallback for levels not yet defined -----------------------------
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_professions_have_level_1_trades() {
        let professions = [
            VillagerProfession::Farmer,
            VillagerProfession::Librarian,
            VillagerProfession::Cleric,
            VillagerProfession::Armorer,
            VillagerProfession::Weaponsmith,
            VillagerProfession::Toolsmith,
            VillagerProfession::Butcher,
            VillagerProfession::Leatherworker,
            VillagerProfession::Fletcher,
            VillagerProfession::Cartographer,
            VillagerProfession::Mason,
            VillagerProfession::Shepherd,
        ];
        for profession in &professions {
            let trades = default_trades(*profession, 1);
            assert!(
                trades.len() >= 2,
                "{profession:?} should have at least 2 L1 trades, got {}",
                trades.len()
            );
        }
    }

    #[test]
    fn nitwit_has_no_trades() {
        let trades = default_trades(VillagerProfession::Nitwit, 1);
        assert!(trades.is_empty());
    }
}
