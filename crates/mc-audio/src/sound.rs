use glam::Vec3;

/// Unique identifier for each sound effect in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundId {
    BlockPlace,
    BlockBreak,
    BlockStep,
    StepGrass,
    StepStone,
    StepWood,
    StepSand,
    StepGravel,
    StepSnow,
    Splash,
    Swim,
    Explosion,
    BowShoot,
    ArrowHit,
    Hurt,
    Death,
    Eat,
    Burp,
    LevelUp,
    Experience,
    Anvil,
    ChestOpen,
    ChestClose,
    DoorOpen,
    DoorClose,
    ButtonClick,
    ZombieGrowl,
    SkeletonRattle,
    CreeperHiss,
    SpiderHiss,
}

/// Category that a sound belongs to, used for per-category volume control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Master,
    Music,
    Weather,
    Blocks,
    Hostile,
    Friendly,
    Players,
    Ambient,
}

/// Static properties associated with a particular sound.
#[derive(Debug, Clone, Copy)]
pub struct SoundProperties {
    pub name: &'static str,
    pub default_volume: f32,
    pub default_pitch: f32,
    pub category: SoundCategory,
}

impl SoundId {
    /// Returns the static properties for this sound.
    pub fn properties(self) -> SoundProperties {
        match self {
            SoundId::BlockPlace => SoundProperties {
                name: "block.place",
                default_volume: 1.0,
                default_pitch: 0.8,
                category: SoundCategory::Blocks,
            },
            SoundId::BlockBreak => SoundProperties {
                name: "block.break",
                default_volume: 1.0,
                default_pitch: 0.8,
                category: SoundCategory::Blocks,
            },
            SoundId::BlockStep => SoundProperties {
                name: "block.step",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::StepGrass => SoundProperties {
                name: "step.grass",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::StepStone => SoundProperties {
                name: "step.stone",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::StepWood => SoundProperties {
                name: "step.wood",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::StepSand => SoundProperties {
                name: "step.sand",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::StepGravel => SoundProperties {
                name: "step.gravel",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::StepSnow => SoundProperties {
                name: "step.snow",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::Splash => SoundProperties {
                name: "liquid.splash",
                default_volume: 0.5,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Swim => SoundProperties {
                name: "liquid.swim",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Explosion => SoundProperties {
                name: "random.explode",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::BowShoot => SoundProperties {
                name: "random.bow",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::ArrowHit => SoundProperties {
                name: "random.arrow_hit",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Hurt => SoundProperties {
                name: "game.player.hurt",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Death => SoundProperties {
                name: "game.player.death",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Eat => SoundProperties {
                name: "random.eat",
                default_volume: 0.5,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Burp => SoundProperties {
                name: "random.burp",
                default_volume: 0.5,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::LevelUp => SoundProperties {
                name: "random.levelup",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Experience => SoundProperties {
                name: "random.orb",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Players,
            },
            SoundId::Anvil => SoundProperties {
                name: "random.anvil_use",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::ChestOpen => SoundProperties {
                name: "random.chestopen",
                default_volume: 0.5,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::ChestClose => SoundProperties {
                name: "random.chestclosed",
                default_volume: 0.5,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::DoorOpen => SoundProperties {
                name: "random.door_open",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::DoorClose => SoundProperties {
                name: "random.door_close",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::ButtonClick => SoundProperties {
                name: "random.click",
                default_volume: 0.3,
                default_pitch: 1.0,
                category: SoundCategory::Blocks,
            },
            SoundId::ZombieGrowl => SoundProperties {
                name: "mob.zombie.say",
                default_volume: 1.0,
                default_pitch: 0.8,
                category: SoundCategory::Hostile,
            },
            SoundId::SkeletonRattle => SoundProperties {
                name: "mob.skeleton.say",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Hostile,
            },
            SoundId::CreeperHiss => SoundProperties {
                name: "mob.creeper.say",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Hostile,
            },
            SoundId::SpiderHiss => SoundProperties {
                name: "mob.spider.say",
                default_volume: 1.0,
                default_pitch: 1.0,
                category: SoundCategory::Hostile,
            },
        }
    }
}

/// A sound event queued for playback at a specific world position.
#[derive(Debug, Clone)]
pub struct SoundEvent {
    pub sound_id: SoundId,
    pub position: Vec3,
    pub volume: f32,
    pub pitch: f32,
    pub category: SoundCategory,
}

/// Queue that accumulates sound events during a game tick, to be drained by
/// the audio renderer.
#[derive(Debug, Default)]
pub struct SoundQueue {
    events: Vec<SoundEvent>,
}

impl SoundQueue {
    /// Creates an empty sound queue.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Queues a sound with its default volume and pitch.
    pub fn play(&mut self, sound_id: SoundId, position: Vec3) {
        let props = sound_id.properties();
        self.play_with(sound_id, position, props.default_volume, props.default_pitch);
    }

    /// Queues a sound with explicit volume and pitch.
    pub fn play_with(
        &mut self,
        sound_id: SoundId,
        position: Vec3,
        volume: f32,
        pitch: f32,
    ) {
        let category = sound_id.properties().category;
        self.events.push(SoundEvent {
            sound_id,
            position,
            volume,
            pitch,
            category,
        });
    }

    /// Takes all queued events, leaving the queue empty.
    pub fn drain(&mut self) -> Vec<SoundEvent> {
        std::mem::take(&mut self.events)
    }

    /// Discards all queued events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Returns the number of currently queued events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `true` if the queue contains no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Attenuates volume based on distance from the listener.
///
/// Uses a linear falloff: beyond 16 blocks the sound is inaudible.
pub fn volume_at_distance(distance: f32, base_volume: f32) -> f32 {
    (base_volume * (1.0 - distance / 16.0)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_id_has_30_variants() {
        // Ensure all 30 variants exist and properties are accessible.
        let ids = [
            SoundId::BlockPlace,
            SoundId::BlockBreak,
            SoundId::BlockStep,
            SoundId::StepGrass,
            SoundId::StepStone,
            SoundId::StepWood,
            SoundId::StepSand,
            SoundId::StepGravel,
            SoundId::StepSnow,
            SoundId::Splash,
            SoundId::Swim,
            SoundId::Explosion,
            SoundId::BowShoot,
            SoundId::ArrowHit,
            SoundId::Hurt,
            SoundId::Death,
            SoundId::Eat,
            SoundId::Burp,
            SoundId::LevelUp,
            SoundId::Experience,
            SoundId::Anvil,
            SoundId::ChestOpen,
            SoundId::ChestClose,
            SoundId::DoorOpen,
            SoundId::DoorClose,
            SoundId::ButtonClick,
            SoundId::ZombieGrowl,
            SoundId::SkeletonRattle,
            SoundId::CreeperHiss,
            SoundId::SpiderHiss,
        ];
        assert_eq!(ids.len(), 30);
        for id in &ids {
            let props = id.properties();
            assert!(!props.name.is_empty());
            assert!(props.default_volume > 0.0);
            assert!(props.default_pitch > 0.0);
        }
    }

    #[test]
    fn queue_play_adds_event_with_defaults() {
        let mut queue = SoundQueue::new();
        queue.play(SoundId::BlockPlace, Vec3::new(1.0, 2.0, 3.0));

        let events = queue.drain();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.sound_id, SoundId::BlockPlace);
        assert_eq!(event.position, Vec3::new(1.0, 2.0, 3.0));

        let props = SoundId::BlockPlace.properties();
        assert!((event.volume - props.default_volume).abs() < f32::EPSILON);
        assert!((event.pitch - props.default_pitch).abs() < f32::EPSILON);
        assert_eq!(event.category, SoundCategory::Blocks);
    }

    #[test]
    fn queue_play_with_custom_volume_pitch() {
        let mut queue = SoundQueue::new();
        queue.play_with(SoundId::Explosion, Vec3::ZERO, 0.75, 1.5);

        let events = queue.drain();
        assert_eq!(events.len(), 1);
        assert!((events[0].volume - 0.75).abs() < f32::EPSILON);
        assert!((events[0].pitch - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn drain_empties_queue() {
        let mut queue = SoundQueue::new();
        queue.play(SoundId::Hurt, Vec3::ZERO);
        queue.play(SoundId::Death, Vec3::ONE);

        let first = queue.drain();
        assert_eq!(first.len(), 2);

        let second = queue.drain();
        assert!(second.is_empty());
    }

    #[test]
    fn clear_removes_all_events() {
        let mut queue = SoundQueue::new();
        queue.play(SoundId::Eat, Vec3::ZERO);
        queue.play(SoundId::Burp, Vec3::ZERO);
        queue.clear();

        let events = queue.drain();
        assert!(events.is_empty());
    }

    #[test]
    fn volume_at_distance_zero() {
        let vol = volume_at_distance(0.0, 1.0);
        assert!((vol - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn volume_at_distance_half() {
        let vol = volume_at_distance(8.0, 1.0);
        assert!((vol - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn volume_at_distance_full_range() {
        let vol = volume_at_distance(16.0, 1.0);
        assert!((vol - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn volume_at_distance_beyond_range_clamps_to_zero() {
        let vol = volume_at_distance(20.0, 1.0);
        assert!((vol - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn volume_at_distance_with_base_volume() {
        let vol = volume_at_distance(4.0, 0.8);
        // 0.8 * (1.0 - 4/16) = 0.8 * 0.75 = 0.6
        assert!((vol - 0.6).abs() < f32::EPSILON);
    }
}
