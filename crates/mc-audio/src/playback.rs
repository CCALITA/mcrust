use crate::sound::SoundEvent;

/// Abstraction over an audio output device.
///
/// Implementations must be safe to share across threads (`Send + Sync`).
/// Two built-in implementations are provided:
///
/// * [`StubAudioBackend`] -- logs every call via the `log` crate (useful for
///   development / integration testing).
/// * [`NullAudioBackend`] -- does nothing (useful for headless / CI runs).
pub trait AudioBackend: Send + Sync {
    /// Play a one-shot sound effect identified by `sound_id`.
    fn play_sound(&self, sound_id: u8, volume: f32, pitch: f32);

    /// Begin (or switch to) a looping music track.
    fn play_music(&self, track_id: u8, volume: f32);

    /// Stop any currently playing music.
    fn stop_music(&self);

    /// Set the master volume that scales all other volumes.
    fn set_master_volume(&self, volume: f32);
}

// ---------------------------------------------------------------------------
// Stub backend -- logs to `log` crate
// ---------------------------------------------------------------------------

/// A backend that logs every call at the `debug` level.
///
/// No actual audio is produced. Useful during development when no real audio
/// system is available but you want to observe what the engine requests.
#[derive(Debug, Default)]
pub struct StubAudioBackend;

impl AudioBackend for StubAudioBackend {
    fn play_sound(&self, sound_id: u8, volume: f32, pitch: f32) {
        log::debug!(
            "StubAudioBackend::play_sound(sound_id={sound_id}, volume={volume:.2}, pitch={pitch:.2})"
        );
    }

    fn play_music(&self, track_id: u8, volume: f32) {
        log::debug!("StubAudioBackend::play_music(track_id={track_id}, volume={volume:.2})");
    }

    fn stop_music(&self) {
        log::debug!("StubAudioBackend::stop_music()");
    }

    fn set_master_volume(&self, volume: f32) {
        log::debug!("StubAudioBackend::set_master_volume(volume={volume:.2})");
    }
}

// ---------------------------------------------------------------------------
// Null backend -- no-op
// ---------------------------------------------------------------------------

/// A backend that silently discards every request.
///
/// Use this for headless servers, benchmarks, or CI environments where audio
/// output is irrelevant.
#[derive(Debug, Default)]
pub struct NullAudioBackend;

impl AudioBackend for NullAudioBackend {
    fn play_sound(&self, _sound_id: u8, _volume: f32, _pitch: f32) {}
    fn play_music(&self, _track_id: u8, _volume: f32) {}
    fn stop_music(&self) {}
    fn set_master_volume(&self, _volume: f32) {}
}

// ---------------------------------------------------------------------------
// AudioManager
// ---------------------------------------------------------------------------

/// Central manager that dispatches sound and music requests through an
/// [`AudioBackend`].
///
/// Volume levels are kept in the `0.0..=1.0` range and clamped on set.
pub struct AudioManager {
    backend: Box<dyn AudioBackend>,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
}

impl AudioManager {
    /// Creates a new `AudioManager` with all volumes at `1.0`.
    pub fn new(backend: Box<dyn AudioBackend>) -> Self {
        let manager = Self {
            backend,
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
        };
        manager.backend.set_master_volume(1.0);
        manager
    }

    /// Returns the current master volume.
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Returns the current music volume.
    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    /// Returns the current sound-effects volume.
    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }

    /// Sets the master volume (clamped to `0.0..=1.0`) and notifies the
    /// backend.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        self.backend.set_master_volume(self.master_volume);
    }

    /// Sets the music volume (clamped to `0.0..=1.0`).
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Sets the sound-effects volume (clamped to `0.0..=1.0`).
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Plays a music track through the backend, scaled by `master * music`
    /// volume.
    pub fn play_music(&self, track_id: u8) {
        let effective = self.master_volume * self.music_volume;
        self.backend.play_music(track_id, effective);
    }

    /// Stops music playback.
    pub fn stop_music(&self) {
        self.backend.stop_music();
    }

    /// Drains a list of [`SoundEvent`]s and forwards each to the backend,
    /// scaling volumes by `master * sfx`.
    pub fn process_sound_queue(&self, events: Vec<SoundEvent>) {
        let volume_scale = self.master_volume * self.sfx_volume;
        for event in &events {
            let effective_volume = event.volume * volume_scale;
            self.backend.play_sound(
                event.sound_id as u8,
                effective_volume,
                event.pitch,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sound::{SoundCategory, SoundId};
    use glam::Vec3;

    /// Helper: build an `AudioManager` backed by [`NullAudioBackend`].
    fn null_manager() -> AudioManager {
        AudioManager::new(Box::new(NullAudioBackend))
    }

    // -- construction -------------------------------------------------------

    #[test]
    fn new_manager_has_default_volumes() {
        let mgr = null_manager();
        assert!((mgr.master_volume() - 1.0).abs() < f32::EPSILON);
        assert!((mgr.music_volume() - 1.0).abs() < f32::EPSILON);
        assert!((mgr.sfx_volume() - 1.0).abs() < f32::EPSILON);
    }

    // -- volume clamping ----------------------------------------------------

    #[test]
    fn set_master_volume_clamps_to_unit_range() {
        let mut mgr = null_manager();

        mgr.set_master_volume(1.5);
        assert!((mgr.master_volume() - 1.0).abs() < f32::EPSILON);

        mgr.set_master_volume(-0.3);
        assert!((mgr.master_volume() - 0.0).abs() < f32::EPSILON);

        mgr.set_master_volume(0.7);
        assert!((mgr.master_volume() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn set_music_volume_clamps_to_unit_range() {
        let mut mgr = null_manager();

        mgr.set_music_volume(2.0);
        assert!((mgr.music_volume() - 1.0).abs() < f32::EPSILON);

        mgr.set_music_volume(-1.0);
        assert!((mgr.music_volume() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_sfx_volume_clamps_to_unit_range() {
        let mut mgr = null_manager();

        mgr.set_sfx_volume(5.0);
        assert!((mgr.sfx_volume() - 1.0).abs() < f32::EPSILON);

        mgr.set_sfx_volume(-0.1);
        assert!((mgr.sfx_volume() - 0.0).abs() < f32::EPSILON);
    }

    // -- process_sound_queue ------------------------------------------------

    #[test]
    fn process_sound_queue_handles_empty_vec() {
        let mgr = null_manager();
        // Should not panic.
        mgr.process_sound_queue(Vec::new());
    }

    #[test]
    fn process_sound_queue_forwards_events() {
        let mgr = null_manager();
        let events = vec![
            SoundEvent {
                sound_id: SoundId::BlockPlace,
                position: Vec3::ZERO,
                volume: 1.0,
                pitch: 0.8,
                category: SoundCategory::Blocks,
            },
            SoundEvent {
                sound_id: SoundId::Explosion,
                position: Vec3::new(5.0, 0.0, 5.0),
                volume: 0.5,
                pitch: 1.2,
                category: SoundCategory::Blocks,
            },
        ];
        // With NullAudioBackend this simply must not panic.
        mgr.process_sound_queue(events);
    }

    // -- music controls -----------------------------------------------------

    #[test]
    fn play_and_stop_music_do_not_panic() {
        let mgr = null_manager();
        mgr.play_music(1);
        mgr.stop_music();
    }

    // -- backend trait object safety ----------------------------------------

    #[test]
    fn null_backend_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NullAudioBackend>();
    }

    #[test]
    fn stub_backend_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StubAudioBackend>();
    }

    // -- stub backend logs (smoke test) -------------------------------------

    #[test]
    fn stub_backend_methods_do_not_panic() {
        let backend = StubAudioBackend;
        backend.play_sound(0, 1.0, 1.0);
        backend.play_music(1, 0.8);
        backend.stop_music();
        backend.set_master_volume(0.5);
    }
}
