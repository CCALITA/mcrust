use glam::Vec3;
use mc_audio::ambient::{AmbientConditions, AmbientSound, select_ambient};
use mc_audio::footsteps::{footstep_for_block, footstep_interval, footstep_volume, FootstepSound};
use mc_audio::{MusicAction, MusicPlayer, SoundEvent, SoundId, SoundQueue};

/// Bridge between game events and the audio subsystem.
///
/// Translates high-level game actions (block break, player hurt, etc.) into
/// [`SoundEvent`]s queued for the audio backend, and drives the background
/// music player.
pub struct GameSoundSystem {
    sounds: SoundQueue,
    music: MusicPlayer,
    active_ambient: Vec<(AmbientSound, f32)>,
    footstep_timer: f32,
}

impl GameSoundSystem {
    /// Creates a new sound system with an empty queue and a fresh music player.
    pub fn new() -> Self {
        Self {
            sounds: SoundQueue::new(),
            music: MusicPlayer::new(),
            active_ambient: Vec::new(),
            footstep_timer: 0.0,
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

    /// Updates the active ambient sound set based on environmental conditions.
    ///
    /// Evaluates the player's dimension, depth, cave status, weather, and time of
    /// day to select appropriate ambient loops with associated volume levels.
    pub fn update_ambient(
        &mut self,
        dimension: u8,
        y: i32,
        is_cave: bool,
        weather: u8,
        time_of_day: f32,
    ) {
        let conditions = AmbientConditions {
            dimension,
            y,
            is_underwater: false,
            is_cave,
            weather,
            time_of_day,
        };
        self.active_ambient = select_ambient(&conditions);
    }

    /// Returns the currently active ambient sounds with their volume levels.
    pub fn active_ambient(&self) -> &[(AmbientSound, f32)] {
        &self.active_ambient
    }

    /// Advances the footstep timer and queues a step sound when the interval elapses.
    ///
    /// Only ticks when the player is on the ground and moving above the minimum
    /// speed threshold (0.1). The sound emitted depends on the block below the
    /// player and the volume scales with movement speed and sneak state.
    pub fn tick_footsteps(
        &mut self,
        speed: f32,
        on_ground: bool,
        is_sprinting: bool,
        is_sneaking: bool,
        block_below: u16,
        dt: f32,
    ) {
        if !on_ground || speed <= 0.1 {
            self.footstep_timer = 0.0;
            return;
        }

        self.footstep_timer += dt;
        let interval = footstep_interval(speed, is_sprinting);

        if self.footstep_timer >= interval {
            self.footstep_timer = 0.0;
            let volume = footstep_volume(speed, is_sneaking);
            let step_sound = footstep_for_block(block_below);
            let sound_id = footstep_sound_to_id(step_sound);
            self.sounds.play_with(sound_id, Vec3::ZERO, volume, 1.0);
        }
    }
}

/// Maps a [`FootstepSound`] category to the corresponding [`SoundId`] variant.
fn footstep_sound_to_id(sound: FootstepSound) -> SoundId {
    match sound {
        FootstepSound::Grass => SoundId::StepGrass,
        FootstepSound::Stone => SoundId::StepStone,
        FootstepSound::Wood => SoundId::StepWood,
        FootstepSound::Sand | FootstepSound::Powder => SoundId::StepSand,
        FootstepSound::Gravel => SoundId::StepGravel,
        FootstepSound::Snow => SoundId::StepSnow,
        FootstepSound::Wool | FootstepSound::Honey => SoundId::BlockStep,
        FootstepSound::Glass | FootstepSound::Metal | FootstepSound::Wet => SoundId::BlockStep,
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

    // --- Ambient sound tests ---

    #[test]
    fn update_ambient_stores_rain_sound_during_rain() {
        let mut sys = GameSoundSystem::new();
        sys.update_ambient(0, 64, false, 1, 0.5);

        let ambient = sys.active_ambient();
        assert!(!ambient.is_empty());
        assert!(ambient.iter().any(|(s, _)| *s == mc_audio::ambient::AmbientSound::Rain));
    }

    #[test]
    fn update_ambient_stores_nether_loop_in_nether() {
        let mut sys = GameSoundSystem::new();
        sys.update_ambient(1, 64, false, 0, 0.5);

        let ambient = sys.active_ambient();
        assert_eq!(ambient.len(), 1);
        assert_eq!(ambient[0].0, mc_audio::ambient::AmbientSound::NetherLoop);
    }

    #[test]
    fn update_ambient_clear_day_is_empty() {
        let mut sys = GameSoundSystem::new();
        sys.update_ambient(0, 64, false, 0, 0.5);

        assert!(sys.active_ambient().is_empty());
    }

    #[test]
    fn update_ambient_cave_underground() {
        let mut sys = GameSoundSystem::new();
        sys.update_ambient(0, 20, true, 0, 0.5);

        let ambient = sys.active_ambient();
        assert!(ambient.iter().any(|(s, _)| *s == mc_audio::ambient::AmbientSound::CaveAmbience));
    }

    // --- Footstep tests ---

    #[test]
    fn tick_footsteps_no_sound_when_airborne() {
        let mut sys = GameSoundSystem::new();
        sys.tick_footsteps(5.0, false, false, false, 1, 1.0);
        assert_eq!(sys.pending_sounds(), 0);
    }

    #[test]
    fn tick_footsteps_no_sound_when_stationary() {
        let mut sys = GameSoundSystem::new();
        sys.tick_footsteps(0.05, true, false, false, 1, 1.0);
        assert_eq!(sys.pending_sounds(), 0);
    }

    #[test]
    fn tick_footsteps_queues_sound_after_interval() {
        let mut sys = GameSoundSystem::new();
        // Walking speed 4.0, interval is 0.5s
        sys.tick_footsteps(4.0, true, false, false, 1, 0.5);
        assert_eq!(sys.pending_sounds(), 1);
    }

    #[test]
    fn tick_footsteps_no_sound_before_interval() {
        let mut sys = GameSoundSystem::new();
        // Walking speed 4.0, interval is 0.5s, only 0.3s elapsed
        sys.tick_footsteps(4.0, true, false, false, 1, 0.3);
        assert_eq!(sys.pending_sounds(), 0);
    }

    #[test]
    fn tick_footsteps_resets_timer_when_not_moving() {
        let mut sys = GameSoundSystem::new();
        // Accumulate some time
        sys.tick_footsteps(4.0, true, false, false, 1, 0.3);
        // Stop moving
        sys.tick_footsteps(0.0, true, false, false, 1, 0.1);
        // Start again - should need full interval again
        sys.tick_footsteps(4.0, true, false, false, 1, 0.3);
        assert_eq!(sys.pending_sounds(), 0);
    }

    #[test]
    fn tick_footsteps_sprinting_uses_shorter_interval() {
        let mut sys = GameSoundSystem::new();
        // Sprinting interval is 0.35s
        sys.tick_footsteps(6.0, true, true, false, 1, 0.35);
        assert_eq!(sys.pending_sounds(), 1);
    }

    #[test]
    fn footstep_sound_mapping_grass() {
        assert_eq!(footstep_sound_to_id(FootstepSound::Grass), SoundId::StepGrass);
    }

    #[test]
    fn footstep_sound_mapping_stone() {
        assert_eq!(footstep_sound_to_id(FootstepSound::Stone), SoundId::StepStone);
    }

    #[test]
    fn footstep_sound_mapping_wood() {
        assert_eq!(footstep_sound_to_id(FootstepSound::Wood), SoundId::StepWood);
    }
}
