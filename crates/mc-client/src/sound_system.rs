use glam::Vec3;
use mc_audio::{MusicAction, MusicPlayer, SoundEvent, SoundId, SoundQueue};

/// Bridge between game events and the audio subsystem.
///
/// Translates high-level game actions (block break, player hurt, etc.) into
/// [`SoundEvent`]s queued for the audio backend, and drives the background
/// music player.
pub struct GameSoundSystem {
    sounds: SoundQueue,
    music: MusicPlayer,
}

impl GameSoundSystem {
    /// Creates a new sound system with an empty queue and a fresh music player.
    pub fn new() -> Self {
        Self {
            sounds: SoundQueue::new(),
            music: MusicPlayer::new(),
        }
    }

    /// Queues a block-break sound at the given world position.
    pub fn on_block_break(&mut self, pos: Vec3) {
        self.sounds.play(SoundId::BlockBreak, pos);
    }

    /// Queues a block-place sound at the given world position.
    pub fn on_block_place(&mut self, pos: Vec3) {
        self.sounds.play(SoundId::BlockPlace, pos);
    }

    /// Queues a player-hurt sound at the given world position.
    pub fn on_player_hurt(&mut self, pos: Vec3) {
        self.sounds.play(SoundId::Hurt, pos);
    }

    /// Queues a jump/step sound at the given world position.
    ///
    /// Currently uses `StepGrass` as a placeholder until surface-aware step
    /// sounds are implemented.
    pub fn on_player_jump(&mut self, pos: Vec3) {
        self.sounds.play(SoundId::StepGrass, pos);
    }

    /// Queues an eating sound at the given world position.
    pub fn on_eat(&mut self, pos: Vec3) {
        self.sounds.play(SoundId::Eat, pos);
    }

    /// Queues an explosion sound at the given world position.
    pub fn on_explosion(&mut self, pos: Vec3) {
        self.sounds.play(SoundId::Explosion, pos);
    }

    /// Queues a level-up sound at the given world position.
    pub fn on_level_up(&mut self, pos: Vec3) {
        self.sounds.play(SoundId::LevelUp, pos);
    }

    /// Advances the background music player by `dt` seconds.
    ///
    /// `dimension` selects which track pool to draw from:
    /// - `0` = Overworld
    /// - `1` = Nether
    /// - `2` = End
    ///
    /// Returns a [`MusicAction`] when the music state machine transitions.
    pub fn tick_music(&mut self, dt: f32, dimension: u8) -> Option<MusicAction> {
        self.music.tick(dt, dimension)
    }

    /// Drains all pending sound events for the audio backend to consume.
    pub fn drain_sound_events(&mut self) -> Vec<SoundEvent> {
        self.sounds.drain()
    }

    /// Returns the number of sounds currently queued.
    pub fn pending_sounds(&self) -> usize {
        self.sounds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_system_has_no_pending_sounds() {
        let sys = GameSoundSystem::new();
        assert_eq!(sys.pending_sounds(), 0);
    }

    #[test]
    fn on_block_break_queues_block_break_sound() {
        let mut sys = GameSoundSystem::new();
        sys.on_block_break(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(sys.pending_sounds(), 1);
        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sound_id, SoundId::BlockBreak);
        assert_eq!(events[0].position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn on_block_place_queues_block_place_sound() {
        let mut sys = GameSoundSystem::new();
        sys.on_block_place(Vec3::ZERO);

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sound_id, SoundId::BlockPlace);
    }

    #[test]
    fn on_player_hurt_queues_hurt_sound() {
        let mut sys = GameSoundSystem::new();
        sys.on_player_hurt(Vec3::ONE);

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sound_id, SoundId::Hurt);
    }

    #[test]
    fn on_player_jump_queues_step_grass_placeholder() {
        let mut sys = GameSoundSystem::new();
        sys.on_player_jump(Vec3::new(5.0, 10.0, 15.0));

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sound_id, SoundId::StepGrass);
    }

    #[test]
    fn on_eat_queues_eat_sound() {
        let mut sys = GameSoundSystem::new();
        sys.on_eat(Vec3::ZERO);

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sound_id, SoundId::Eat);
    }

    #[test]
    fn on_explosion_queues_explosion_sound() {
        let mut sys = GameSoundSystem::new();
        sys.on_explosion(Vec3::new(-10.0, 64.0, 30.0));

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sound_id, SoundId::Explosion);
    }

    #[test]
    fn on_level_up_queues_level_up_sound() {
        let mut sys = GameSoundSystem::new();
        sys.on_level_up(Vec3::ZERO);

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sound_id, SoundId::LevelUp);
    }

    #[test]
    fn drain_clears_pending_sounds() {
        let mut sys = GameSoundSystem::new();
        sys.on_block_break(Vec3::ZERO);
        sys.on_block_place(Vec3::ONE);
        assert_eq!(sys.pending_sounds(), 2);

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 2);
        assert_eq!(sys.pending_sounds(), 0);

        let second = sys.drain_sound_events();
        assert!(second.is_empty());
    }

    #[test]
    fn multiple_events_queued_in_order() {
        let mut sys = GameSoundSystem::new();
        sys.on_block_break(Vec3::ZERO);
        sys.on_player_hurt(Vec3::ONE);
        sys.on_explosion(Vec3::new(2.0, 3.0, 4.0));

        let events = sys.drain_sound_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sound_id, SoundId::BlockBreak);
        assert_eq!(events[1].sound_id, SoundId::Hurt);
        assert_eq!(events[2].sound_id, SoundId::Explosion);
    }
}
