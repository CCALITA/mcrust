// ---------------------------------------------------------------------------
// Panda — genetics, personality, state, and breeding
// ---------------------------------------------------------------------------

/// Simple hash-based pseudo-random number derived from `seed` and `salt`.
fn pseudo_random(seed: u64, salt: u32) -> u64 {
    let mut h = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(salt as u64);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

// ---------------------------------------------------------------------------
// Personality
// ---------------------------------------------------------------------------

/// The seven panda personality variants from Minecraft.
///
/// Normal, Lazy, Worried, and Playful are *dominant* — they show when the
/// main gene matches.  Aggressive, Weak, and Brown are *recessive* — they
/// only display when **both** main and hidden genes match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PandaPersonality {
    Normal,
    Lazy,
    Worried,
    Playful,
    Aggressive,
    Weak,
    Brown,
}

impl PandaPersonality {
    /// Total number of personality variants (used for gene selection).
    const COUNT: u64 = 7;

    /// Returns `true` for dominant personalities (display when main gene alone
    /// matches).
    pub fn is_dominant(self) -> bool {
        matches!(
            self,
            Self::Normal | Self::Lazy | Self::Worried | Self::Playful
        )
    }

    /// Returns `true` for recessive personalities (display only when both
    /// main and hidden genes match).
    pub fn is_recessive(self) -> bool {
        !self.is_dominant()
    }

    /// Map an index `0..7` to a personality variant.
    fn from_index(index: u64) -> Self {
        match index % Self::COUNT {
            0 => Self::Normal,
            1 => Self::Lazy,
            2 => Self::Worried,
            3 => Self::Playful,
            4 => Self::Aggressive,
            5 => Self::Weak,
            6 => Self::Brown,
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Genes
// ---------------------------------------------------------------------------

/// A panda's genetic makeup consisting of a main gene and a hidden gene.
///
/// The displayed personality follows Minecraft rules:
/// - If the main gene is *dominant*, that personality is displayed.
/// - If the main gene is *recessive*, it only displays when the hidden gene
///   matches; otherwise the panda appears Normal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PandaGenes {
    pub main: PandaPersonality,
    pub hidden: PandaPersonality,
}

impl PandaGenes {
    pub fn new(main: PandaPersonality, hidden: PandaPersonality) -> Self {
        Self { main, hidden }
    }
}

/// Determine the displayed personality from a set of genes.
///
/// Dominant main genes always show.  Recessive main genes only show when the
/// hidden gene is identical; otherwise the panda appears Normal.
pub fn personality(genes: &PandaGenes) -> PandaPersonality {
    if genes.main.is_dominant() {
        return genes.main;
    }
    // Recessive — must match hidden gene to display.
    if genes.main == genes.hidden {
        genes.main
    } else {
        PandaPersonality::Normal
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Runtime state for a panda entity.
#[derive(Debug, Clone)]
pub struct PandaState {
    pub genes: PandaGenes,
    pub sitting: bool,
    pub sneeze_timer: f32,
}

impl PandaState {
    /// Sneeze cooldown duration in seconds for baby pandas.
    const SNEEZE_INTERVAL: f32 = 8.0;

    pub fn new(genes: PandaGenes) -> Self {
        Self {
            genes,
            sitting: false,
            sneeze_timer: 0.0,
        }
    }

    /// Tick the sneeze timer by `dt` seconds.
    ///
    /// Returns `true` when the sneeze timer has elapsed (baby panda sneezes).
    pub fn tick_sneeze(&mut self, dt: f32) -> bool {
        self.sneeze_timer += dt;
        if self.sneeze_timer >= Self::SNEEZE_INTERVAL {
            self.sneeze_timer -= Self::SNEEZE_INTERVAL;
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Breeding
// ---------------------------------------------------------------------------

/// Breed two pandas and produce offspring genes.
///
/// Each gene (main and hidden) of the offspring is independently chosen:
/// the child has a 50/50 chance of inheriting each gene from either parent.
/// After selection there is a 1-in-32 chance per gene of a random mutation.
///
/// Uses deterministic hashing from `seed` so the result is reproducible.
pub fn breed(parent1: &PandaGenes, parent2: &PandaGenes, seed: u64) -> PandaGenes {
    let pick_main = pseudo_random(seed, 0);
    let pick_hidden = pseudo_random(seed, 1);
    let mutate_main = pseudo_random(seed, 2);
    let mutate_hidden = pseudo_random(seed, 3);

    let main = if mutate_main % 32 == 0 {
        // 1-in-32 random mutation
        PandaPersonality::from_index(pseudo_random(seed, 4))
    } else if pick_main % 2 == 0 {
        parent1.main
    } else {
        parent2.main
    };

    let hidden = if mutate_hidden % 32 == 0 {
        PandaPersonality::from_index(pseudo_random(seed, 5))
    } else if pick_hidden % 2 == 0 {
        parent1.hidden
    } else {
        parent2.hidden
    };

    PandaGenes { main, hidden }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Personality display ---------------------------------------------------

    #[test]
    fn dominant_main_gene_always_displays() {
        let genes = PandaGenes::new(PandaPersonality::Lazy, PandaPersonality::Aggressive);
        assert_eq!(personality(&genes), PandaPersonality::Lazy);
    }

    #[test]
    fn recessive_main_gene_displays_when_hidden_matches() {
        let genes = PandaGenes::new(PandaPersonality::Brown, PandaPersonality::Brown);
        assert_eq!(personality(&genes), PandaPersonality::Brown);
    }

    #[test]
    fn recessive_main_gene_falls_back_to_normal_when_hidden_differs() {
        let genes = PandaGenes::new(PandaPersonality::Aggressive, PandaPersonality::Lazy);
        assert_eq!(personality(&genes), PandaPersonality::Normal);
    }

    #[test]
    fn all_dominant_variants_display_regardless_of_hidden() {
        for dominant in [
            PandaPersonality::Normal,
            PandaPersonality::Lazy,
            PandaPersonality::Worried,
            PandaPersonality::Playful,
        ] {
            for hidden in [
                PandaPersonality::Aggressive,
                PandaPersonality::Weak,
                PandaPersonality::Brown,
            ] {
                assert_eq!(
                    personality(&PandaGenes::new(dominant, hidden)),
                    dominant,
                    "dominant {dominant:?} should display with hidden {hidden:?}"
                );
            }
        }
    }

    #[test]
    fn recessive_weak_displays_only_when_both_match() {
        assert_eq!(
            personality(&PandaGenes::new(PandaPersonality::Weak, PandaPersonality::Weak)),
            PandaPersonality::Weak,
        );
        assert_eq!(
            personality(&PandaGenes::new(PandaPersonality::Weak, PandaPersonality::Normal)),
            PandaPersonality::Normal,
        );
    }

    // -- Breeding -------------------------------------------------------------

    #[test]
    fn breed_is_deterministic() {
        let p1 = PandaGenes::new(PandaPersonality::Lazy, PandaPersonality::Worried);
        let p2 = PandaGenes::new(PandaPersonality::Playful, PandaPersonality::Brown);
        let a = breed(&p1, &p2, 12345);
        let b = breed(&p1, &p2, 12345);
        assert_eq!(a, b, "same parents + seed must produce identical offspring");
    }

    #[test]
    fn breed_different_seeds_can_differ() {
        let p1 = PandaGenes::new(PandaPersonality::Lazy, PandaPersonality::Normal);
        let p2 = PandaGenes::new(PandaPersonality::Playful, PandaPersonality::Normal);
        // With enough seeds at least one should differ
        let results: Vec<_> = (0..20).map(|s| breed(&p1, &p2, s)).collect();
        let all_same = results.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same, "different seeds should eventually produce different offspring");
    }

    #[test]
    fn breed_inherits_from_parents_without_mutation() {
        let p1 = PandaGenes::new(PandaPersonality::Lazy, PandaPersonality::Worried);
        let p2 = PandaGenes::new(PandaPersonality::Playful, PandaPersonality::Brown);

        // Test many seeds; non-mutated offspring must inherit a parental gene
        for seed in 100..200 {
            let child = breed(&p1, &p2, seed);
            let mutate_main = pseudo_random(seed, 2);
            let mutate_hidden = pseudo_random(seed, 3);

            if mutate_main % 32 != 0 {
                assert!(
                    child.main == p1.main || child.main == p2.main,
                    "seed {seed}: main gene {:?} not from either parent ({:?}, {:?})",
                    child.main,
                    p1.main,
                    p2.main,
                );
            }
            if mutate_hidden % 32 != 0 {
                assert!(
                    child.hidden == p1.hidden || child.hidden == p2.hidden,
                    "seed {seed}: hidden gene {:?} not from either parent ({:?}, {:?})",
                    child.hidden,
                    p1.hidden,
                    p2.hidden,
                );
            }
        }
    }

    // -- PandaState -----------------------------------------------------------

    #[test]
    fn sneeze_timer_fires_at_interval() {
        let genes = PandaGenes::new(PandaPersonality::Normal, PandaPersonality::Normal);
        let mut state = PandaState::new(genes);

        // Not yet at 8 seconds
        assert!(!state.tick_sneeze(7.0));
        // Cross the 8-second boundary
        assert!(state.tick_sneeze(1.5));
        // Timer should have wrapped around (0.5s remaining after reset)
        assert!(state.sneeze_timer < 1.0);
    }

    #[test]
    fn panda_state_initial_values() {
        let genes = PandaGenes::new(PandaPersonality::Playful, PandaPersonality::Weak);
        let state = PandaState::new(genes);
        assert!(!state.sitting);
        assert!((state.sneeze_timer).abs() < f32::EPSILON);
        assert_eq!(state.genes.main, PandaPersonality::Playful);
    }

    // -- Personality classification -------------------------------------------

    #[test]
    fn dominant_recessive_classification() {
        assert!(PandaPersonality::Normal.is_dominant());
        assert!(PandaPersonality::Lazy.is_dominant());
        assert!(PandaPersonality::Worried.is_dominant());
        assert!(PandaPersonality::Playful.is_dominant());

        assert!(PandaPersonality::Aggressive.is_recessive());
        assert!(PandaPersonality::Weak.is_recessive());
        assert!(PandaPersonality::Brown.is_recessive());
    }
}
