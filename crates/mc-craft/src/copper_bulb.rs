//! Copper bulb block logic.
//!
//! A copper bulb is a light-emitting block toggled by redstone pulses. Its
//! brightness decreases as it oxidizes through four stages. Waxing freezes
//! oxidation, and scraping can remove wax or reverse one oxidation level.

/// The four oxidation stages of a copper bulb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OxidationLevel {
    Clean,
    Exposed,
    Weathered,
    Oxidized,
}

impl OxidationLevel {
    /// Light level emitted when the bulb is lit at this oxidation stage.
    pub const fn brightness(self) -> u8 {
        match self {
            OxidationLevel::Clean => 15,
            OxidationLevel::Exposed => 12,
            OxidationLevel::Weathered => 8,
            OxidationLevel::Oxidized => 4,
        }
    }

    /// Returns the next oxidation stage, or `None` if already fully oxidized.
    pub const fn next(self) -> Option<OxidationLevel> {
        match self {
            OxidationLevel::Clean => Some(OxidationLevel::Exposed),
            OxidationLevel::Exposed => Some(OxidationLevel::Weathered),
            OxidationLevel::Weathered => Some(OxidationLevel::Oxidized),
            OxidationLevel::Oxidized => None,
        }
    }

    /// Returns the previous (less oxidized) stage, or `None` if already clean.
    pub const fn previous(self) -> Option<OxidationLevel> {
        match self {
            OxidationLevel::Clean => None,
            OxidationLevel::Exposed => Some(OxidationLevel::Clean),
            OxidationLevel::Weathered => Some(OxidationLevel::Exposed),
            OxidationLevel::Oxidized => Some(OxidationLevel::Weathered),
        }
    }
}

/// Complete state of a copper bulb block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopperBulbState {
    pub lit: bool,
    pub oxidation: OxidationLevel,
    pub waxed: bool,
}

impl CopperBulbState {
    /// Create a new copper bulb in its default unlit, clean, unwaxed state.
    pub const fn new() -> Self {
        Self {
            lit: false,
            oxidation: OxidationLevel::Clean,
            waxed: false,
        }
    }

    /// Toggle the bulb on/off (triggered by a redstone pulse).
    ///
    /// Returns a new state with `lit` flipped.
    pub const fn toggle(self) -> Self {
        Self {
            lit: !self.lit,
            oxidation: self.oxidation,
            waxed: self.waxed,
        }
    }

    /// The redstone signal strength emitted when the bulb is lit.
    ///
    /// Returns 15 when lit, 0 when off — copper bulbs act as redstone power
    /// sources when lit.
    pub const fn bulb_redstone_signal(self) -> u8 {
        if self.lit {
            15
        } else {
            0
        }
    }

    /// Advance oxidation by one stage. Has no effect if waxed or already
    /// fully oxidized. Returns the (possibly unchanged) new state.
    pub const fn oxidize(self) -> Self {
        if self.waxed {
            return self;
        }
        match self.oxidation.next() {
            Some(next) => Self {
                lit: self.lit,
                oxidation: next,
                waxed: self.waxed,
            },
            None => self,
        }
    }

    /// Apply honeycomb to wax the bulb, freezing its current oxidation level.
    /// Returns the new state with `waxed` set to `true`.
    pub const fn wax_bulb(self) -> Self {
        Self {
            lit: self.lit,
            oxidation: self.oxidation,
            waxed: true,
        }
    }

    /// Scrape wax off with an axe. Only effective if the bulb is waxed.
    /// Returns the new state with `waxed` set to `false`.
    pub const fn scrape_wax(self) -> Self {
        Self {
            lit: self.lit,
            oxidation: self.oxidation,
            waxed: false,
        }
    }

    /// Scrape oxidation with an axe, reverting one oxidation stage.
    ///
    /// In Minecraft, scraping a waxed block removes the wax first; a second
    /// scrape removes one oxidation level. This method only handles the
    /// oxidation scrape (use [`scrape_wax`](Self::scrape_wax) first if waxed).
    /// Has no effect if the bulb is waxed or already clean.
    pub const fn scrape_oxidation(self) -> Self {
        if self.waxed {
            return self;
        }
        match self.oxidation.previous() {
            Some(prev) => Self {
                lit: self.lit,
                oxidation: prev,
                waxed: self.waxed,
            },
            None => self,
        }
    }

    /// Current light level: the oxidation-dependent brightness when lit, 0 when off.
    pub const fn light_level(self) -> u8 {
        if self.lit {
            self.oxidation.brightness()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── OxidationLevel ──────────────────────────────────────────────────

    #[test]
    fn brightness_decreases_with_oxidation() {
        assert_eq!(OxidationLevel::Clean.brightness(), 15);
        assert_eq!(OxidationLevel::Exposed.brightness(), 12);
        assert_eq!(OxidationLevel::Weathered.brightness(), 8);
        assert_eq!(OxidationLevel::Oxidized.brightness(), 4);
    }

    #[test]
    fn next_oxidation_progresses_correctly() {
        assert_eq!(OxidationLevel::Clean.next(), Some(OxidationLevel::Exposed));
        assert_eq!(OxidationLevel::Exposed.next(), Some(OxidationLevel::Weathered));
        assert_eq!(OxidationLevel::Weathered.next(), Some(OxidationLevel::Oxidized));
        assert_eq!(OxidationLevel::Oxidized.next(), None);
    }

    #[test]
    fn previous_oxidation_reverts_correctly() {
        assert_eq!(OxidationLevel::Oxidized.previous(), Some(OxidationLevel::Weathered));
        assert_eq!(OxidationLevel::Weathered.previous(), Some(OxidationLevel::Exposed));
        assert_eq!(OxidationLevel::Exposed.previous(), Some(OxidationLevel::Clean));
        assert_eq!(OxidationLevel::Clean.previous(), None);
    }

    // ── CopperBulbState — construction ──────────────────────────────────

    #[test]
    fn new_bulb_is_unlit_clean_unwaxed() {
        let bulb = CopperBulbState::new();
        assert!(!bulb.lit);
        assert_eq!(bulb.oxidation, OxidationLevel::Clean);
        assert!(!bulb.waxed);
    }

    // ── toggle ──────────────────────────────────────────────────────────

    #[test]
    fn toggle_turns_bulb_on() {
        let bulb = CopperBulbState::new().toggle();
        assert!(bulb.lit);
    }

    #[test]
    fn toggle_twice_returns_to_off() {
        let bulb = CopperBulbState::new().toggle().toggle();
        assert!(!bulb.lit);
    }

    #[test]
    fn toggle_preserves_oxidation_and_wax() {
        let bulb = CopperBulbState {
            lit: false,
            oxidation: OxidationLevel::Weathered,
            waxed: true,
        };
        let toggled = bulb.toggle();
        assert!(toggled.lit);
        assert_eq!(toggled.oxidation, OxidationLevel::Weathered);
        assert!(toggled.waxed);
    }

    // ── bulb_redstone_signal ────────────────────────────────────────────

    #[test]
    fn redstone_signal_is_fifteen_when_lit() {
        let bulb = CopperBulbState::new().toggle();
        assert_eq!(bulb.bulb_redstone_signal(), 15);
    }

    #[test]
    fn redstone_signal_is_zero_when_off() {
        let bulb = CopperBulbState::new();
        assert_eq!(bulb.bulb_redstone_signal(), 0);
    }

    // ── oxidize ─────────────────────────────────────────────────────────

    #[test]
    fn oxidize_advances_one_stage() {
        let bulb = CopperBulbState::new().oxidize();
        assert_eq!(bulb.oxidation, OxidationLevel::Exposed);
    }

    #[test]
    fn oxidize_stops_at_fully_oxidized() {
        let bulb = CopperBulbState {
            lit: false,
            oxidation: OxidationLevel::Oxidized,
            waxed: false,
        };
        let same = bulb.oxidize();
        assert_eq!(same.oxidation, OxidationLevel::Oxidized);
    }

    #[test]
    fn oxidize_blocked_when_waxed() {
        let bulb = CopperBulbState::new().wax_bulb();
        let same = bulb.oxidize();
        assert_eq!(same.oxidation, OxidationLevel::Clean);
    }

    #[test]
    fn full_oxidation_chain() {
        let bulb = CopperBulbState::new()
            .oxidize()
            .oxidize()
            .oxidize();
        assert_eq!(bulb.oxidation, OxidationLevel::Oxidized);
        // One more does nothing
        assert_eq!(bulb.oxidize().oxidation, OxidationLevel::Oxidized);
    }

    // ── wax_bulb ────────────────────────────────────────────────────────

    #[test]
    fn wax_bulb_sets_waxed() {
        let bulb = CopperBulbState::new().wax_bulb();
        assert!(bulb.waxed);
    }

    #[test]
    fn wax_preserves_lit_and_oxidation() {
        let bulb = CopperBulbState {
            lit: true,
            oxidation: OxidationLevel::Exposed,
            waxed: false,
        }
        .wax_bulb();
        assert!(bulb.lit);
        assert_eq!(bulb.oxidation, OxidationLevel::Exposed);
        assert!(bulb.waxed);
    }

    // ── scrape_wax ──────────────────────────────────────────────────────

    #[test]
    fn scrape_wax_removes_wax() {
        let bulb = CopperBulbState::new().wax_bulb().scrape_wax();
        assert!(!bulb.waxed);
    }

    #[test]
    fn scrape_wax_preserves_oxidation() {
        let bulb = CopperBulbState {
            lit: false,
            oxidation: OxidationLevel::Weathered,
            waxed: true,
        }
        .scrape_wax();
        assert_eq!(bulb.oxidation, OxidationLevel::Weathered);
        assert!(!bulb.waxed);
    }

    // ── scrape_oxidation ────────────────────────────────────────────────

    #[test]
    fn scrape_oxidation_reverts_one_stage() {
        let bulb = CopperBulbState {
            lit: false,
            oxidation: OxidationLevel::Weathered,
            waxed: false,
        }
        .scrape_oxidation();
        assert_eq!(bulb.oxidation, OxidationLevel::Exposed);
    }

    #[test]
    fn scrape_oxidation_stops_at_clean() {
        let bulb = CopperBulbState::new().scrape_oxidation();
        assert_eq!(bulb.oxidation, OxidationLevel::Clean);
    }

    #[test]
    fn scrape_oxidation_blocked_when_waxed() {
        let bulb = CopperBulbState {
            lit: false,
            oxidation: OxidationLevel::Oxidized,
            waxed: true,
        }
        .scrape_oxidation();
        assert_eq!(bulb.oxidation, OxidationLevel::Oxidized);
    }

    // ── light_level ─────────────────────────────────────────────────────

    #[test]
    fn light_level_matches_brightness_when_lit() {
        for oxidation in [
            OxidationLevel::Clean,
            OxidationLevel::Exposed,
            OxidationLevel::Weathered,
            OxidationLevel::Oxidized,
        ] {
            let bulb = CopperBulbState {
                lit: true,
                oxidation,
                waxed: false,
            };
            assert_eq!(bulb.light_level(), oxidation.brightness());
        }
    }

    #[test]
    fn light_level_is_zero_when_off() {
        for oxidation in [
            OxidationLevel::Clean,
            OxidationLevel::Exposed,
            OxidationLevel::Weathered,
            OxidationLevel::Oxidized,
        ] {
            let bulb = CopperBulbState {
                lit: false,
                oxidation,
                waxed: false,
            };
            assert_eq!(bulb.light_level(), 0);
        }
    }

    // ── integration: wax-then-scrape round-trip ─────────────────────────

    #[test]
    fn wax_scrape_oxidize_round_trip() {
        let bulb = CopperBulbState::new()
            .oxidize()       // Exposed
            .wax_bulb()      // freeze at Exposed
            .oxidize()       // no-op (waxed)
            .scrape_wax()    // remove wax, still Exposed
            .oxidize();      // Weathered
        assert_eq!(bulb.oxidation, OxidationLevel::Weathered);
        assert!(!bulb.waxed);
    }
}
