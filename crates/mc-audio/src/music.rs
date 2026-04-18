use rand::Rng;

/// Identifies a background music track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MusicTrack {
    Calm1,
    Calm2,
    Calm3,
    Creative1,
    Creative2,
    Nether1,
    Nether2,
    End1,
    BossMusic,
}

/// Minimum duration of a music track in seconds.
const TRACK_DURATION_MIN: f32 = 180.0;
/// Maximum duration of a music track in seconds.
const TRACK_DURATION_MAX: f32 = 300.0;
/// Duration of the fade-out transition in seconds.
const FADE_OUT_DURATION: f32 = 3.0;
/// Minimum idle wait before the next track in seconds.
const IDLE_WAIT_MIN: f32 = 60.0;
/// Maximum idle wait before the next track in seconds.
const IDLE_WAIT_MAX: f32 = 300.0;

/// Dimension constants used for track selection.
pub const DIMENSION_OVERWORLD: u8 = 0;
pub const DIMENSION_NETHER: u8 = 1;
pub const DIMENSION_END: u8 = 2;

/// State machine for the background music player.
#[derive(Debug, Clone)]
pub enum MusicState {
    Idle { wait_timer: f32 },
    Playing { track: MusicTrack, elapsed: f32 },
    FadeOut { track: MusicTrack, fade_timer: f32 },
}

/// Actions emitted by the music player to be handled by an audio backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicAction {
    StartTrack(MusicTrack),
    StopTrack,
    FadeOut,
}

/// Background music player that manages track selection and state transitions.
///
/// Call [`MusicPlayer::tick`] each frame with the delta time and current
/// dimension. The returned [`MusicAction`], if any, tells the audio backend
/// what to do.
pub struct MusicPlayer {
    pub state: MusicState,
}

impl MusicPlayer {
    /// Creates a new music player starting in the idle state with a random
    /// wait timer.
    pub fn new() -> Self {
        let wait = rand::rng().random_range(IDLE_WAIT_MIN..=IDLE_WAIT_MAX);
        Self {
            state: MusicState::Idle { wait_timer: wait },
        }
    }

    /// Creates a music player with a specific initial state (useful for
    /// testing).
    pub fn with_state(state: MusicState) -> Self {
        Self { state }
    }

    /// Advances the music state machine by `dt` seconds.
    ///
    /// `dimension` selects which tracks are appropriate:
    /// - `0` = Overworld (Calm tracks)
    /// - `1` = Nether
    /// - `2` = End
    pub fn tick(&mut self, dt: f32, dimension: u8) -> Option<MusicAction> {
        match &mut self.state {
            MusicState::Idle { wait_timer } => {
                *wait_timer -= dt;
                if *wait_timer <= 0.0 {
                    let track = pick_track_for_dimension(dimension);
                    self.state = MusicState::Playing {
                        track,
                        elapsed: 0.0,
                    };
                    Some(MusicAction::StartTrack(track))
                } else {
                    None
                }
            }
            MusicState::Playing { track, elapsed } => {
                *elapsed += dt;
                let duration = track_duration(*track);
                if *elapsed >= duration {
                    let t = *track;
                    self.state = MusicState::FadeOut {
                        track: t,
                        fade_timer: 0.0,
                    };
                    Some(MusicAction::FadeOut)
                } else {
                    None
                }
            }
            MusicState::FadeOut { fade_timer, .. } => {
                *fade_timer += dt;
                if *fade_timer >= FADE_OUT_DURATION {
                    let wait = rand::rng().random_range(IDLE_WAIT_MIN..=IDLE_WAIT_MAX);
                    self.state = MusicState::Idle { wait_timer: wait };
                    Some(MusicAction::StopTrack)
                } else {
                    None
                }
            }
        }
    }
}

impl Default for MusicPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Selects a track appropriate for the given dimension.
fn pick_track_for_dimension(dimension: u8) -> MusicTrack {
    let mut rng = rand::rng();
    match dimension {
        DIMENSION_NETHER => {
            let tracks = [MusicTrack::Nether1, MusicTrack::Nether2];
            tracks[rng.random_range(0..tracks.len())]
        }
        DIMENSION_END => {
            let tracks = [MusicTrack::End1, MusicTrack::BossMusic];
            tracks[rng.random_range(0..tracks.len())]
        }
        // Overworld and any unknown dimension default to calm tracks.
        _ => {
            let tracks = [
                MusicTrack::Calm1,
                MusicTrack::Calm2,
                MusicTrack::Calm3,
                MusicTrack::Creative1,
                MusicTrack::Creative2,
            ];
            tracks[rng.random_range(0..tracks.len())]
        }
    }
}

/// Returns a deterministic duration for a given track (in seconds).
///
/// Durations are spread across the 180-300s range.
fn track_duration(track: MusicTrack) -> f32 {
    match track {
        MusicTrack::Calm1 => 210.0,
        MusicTrack::Calm2 => 240.0,
        MusicTrack::Calm3 => 195.0,
        MusicTrack::Creative1 => 270.0,
        MusicTrack::Creative2 => 255.0,
        MusicTrack::Nether1 => TRACK_DURATION_MIN,
        MusicTrack::Nether2 => 225.0,
        MusicTrack::End1 => 285.0,
        MusicTrack::BossMusic => TRACK_DURATION_MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_transitions_to_playing_after_timer() {
        let mut player = MusicPlayer::with_state(MusicState::Idle { wait_timer: 1.0 });

        // Not yet expired.
        let action = player.tick(0.5, DIMENSION_OVERWORLD);
        assert!(action.is_none());
        assert!(matches!(player.state, MusicState::Idle { .. }));

        // Timer expires.
        let action = player.tick(0.6, DIMENSION_OVERWORLD);
        assert!(matches!(action, Some(MusicAction::StartTrack(_))));
        assert!(matches!(player.state, MusicState::Playing { .. }));
    }

    #[test]
    fn playing_transitions_to_fadeout_after_duration() {
        let track = MusicTrack::Calm1;
        let duration = track_duration(track);
        let mut player = MusicPlayer::with_state(MusicState::Playing {
            track,
            elapsed: duration - 0.5,
        });

        // Not yet expired.
        let action = player.tick(0.3, DIMENSION_OVERWORLD);
        assert!(action.is_none());

        // Duration reached.
        let action = player.tick(0.3, DIMENSION_OVERWORLD);
        assert_eq!(action, Some(MusicAction::FadeOut));
        assert!(matches!(player.state, MusicState::FadeOut { .. }));
    }

    #[test]
    fn fadeout_transitions_to_idle_after_3_seconds() {
        let mut player = MusicPlayer::with_state(MusicState::FadeOut {
            track: MusicTrack::Calm2,
            fade_timer: 0.0,
        });

        // Partially through fade.
        let action = player.tick(2.0, DIMENSION_OVERWORLD);
        assert!(action.is_none());

        // Fade completes.
        let action = player.tick(1.5, DIMENSION_OVERWORLD);
        assert_eq!(action, Some(MusicAction::StopTrack));
        assert!(matches!(player.state, MusicState::Idle { .. }));
    }

    #[test]
    fn full_lifecycle_idle_playing_fadeout_idle() {
        let mut player = MusicPlayer::with_state(MusicState::Idle { wait_timer: 0.1 });

        // Idle -> Playing.
        let action = player.tick(0.2, DIMENSION_OVERWORLD);
        assert!(matches!(action, Some(MusicAction::StartTrack(_))));
        let track = match player.state {
            MusicState::Playing { track, .. } => track,
            _ => panic!("expected Playing state"),
        };

        // Playing -> FadeOut.
        let dur = track_duration(track);
        let action = player.tick(dur + 1.0, DIMENSION_OVERWORLD);
        assert_eq!(action, Some(MusicAction::FadeOut));

        // FadeOut -> Idle.
        let action = player.tick(FADE_OUT_DURATION + 0.1, DIMENSION_OVERWORLD);
        assert_eq!(action, Some(MusicAction::StopTrack));
        assert!(matches!(player.state, MusicState::Idle { .. }));
    }

    #[test]
    fn nether_dimension_picks_nether_tracks() {
        for _ in 0..20 {
            let track = pick_track_for_dimension(DIMENSION_NETHER);
            assert!(
                matches!(track, MusicTrack::Nether1 | MusicTrack::Nether2),
                "expected nether track, got {track:?}"
            );
        }
    }

    #[test]
    fn end_dimension_picks_end_tracks() {
        for _ in 0..20 {
            let track = pick_track_for_dimension(DIMENSION_END);
            assert!(
                matches!(track, MusicTrack::End1 | MusicTrack::BossMusic),
                "expected end track, got {track:?}"
            );
        }
    }

    #[test]
    fn overworld_dimension_picks_calm_or_creative_tracks() {
        for _ in 0..20 {
            let track = pick_track_for_dimension(DIMENSION_OVERWORLD);
            assert!(
                matches!(
                    track,
                    MusicTrack::Calm1
                        | MusicTrack::Calm2
                        | MusicTrack::Calm3
                        | MusicTrack::Creative1
                        | MusicTrack::Creative2
                ),
                "expected overworld track, got {track:?}"
            );
        }
    }

    #[test]
    fn track_durations_are_within_valid_range() {
        let tracks = [
            MusicTrack::Calm1,
            MusicTrack::Calm2,
            MusicTrack::Calm3,
            MusicTrack::Creative1,
            MusicTrack::Creative2,
            MusicTrack::Nether1,
            MusicTrack::Nether2,
            MusicTrack::End1,
            MusicTrack::BossMusic,
        ];
        for track in &tracks {
            let dur = track_duration(*track);
            assert!(
                dur >= TRACK_DURATION_MIN && dur <= TRACK_DURATION_MAX,
                "{track:?} duration {dur} out of range [{TRACK_DURATION_MIN}, {TRACK_DURATION_MAX}]"
            );
        }
    }

    #[test]
    fn music_track_has_9_variants() {
        let tracks = [
            MusicTrack::Calm1,
            MusicTrack::Calm2,
            MusicTrack::Calm3,
            MusicTrack::Creative1,
            MusicTrack::Creative2,
            MusicTrack::Nether1,
            MusicTrack::Nether2,
            MusicTrack::End1,
            MusicTrack::BossMusic,
        ];
        assert_eq!(tracks.len(), 9);
    }
}
