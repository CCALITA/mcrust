/// Jukebox block and music disc playback system.
///
/// Models a jukebox that accepts music discs, tracks playback elapsed time,
/// and emits events when discs start, finish, or are ejected.

/// A music disc that can be inserted into a jukebox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscTrack {
    Thirteen,
    Cat,
    Blocks,
    Chirp,
    Far,
    Mall,
    Mellohi,
    Stal,
    Strad,
    Ward,
    Wait,
    Pigstep,
    Otherside,
    Five,
    Relic,
}

impl DiscTrack {
    /// Returns the display name of the disc.
    pub fn name(&self) -> &'static str {
        match self {
            DiscTrack::Thirteen => "13",
            DiscTrack::Cat => "cat",
            DiscTrack::Blocks => "blocks",
            DiscTrack::Chirp => "chirp",
            DiscTrack::Far => "far",
            DiscTrack::Mall => "mall",
            DiscTrack::Mellohi => "mellohi",
            DiscTrack::Stal => "stal",
            DiscTrack::Strad => "strad",
            DiscTrack::Ward => "ward",
            DiscTrack::Wait => "wait",
            DiscTrack::Pigstep => "Pigstep",
            DiscTrack::Otherside => "otherside",
            DiscTrack::Five => "5",
            DiscTrack::Relic => "Relic",
        }
    }

    /// Returns the author/artist of the disc.
    pub fn author(&self) -> &'static str {
        match self {
            DiscTrack::Thirteen
            | DiscTrack::Cat
            | DiscTrack::Blocks
            | DiscTrack::Chirp
            | DiscTrack::Far
            | DiscTrack::Mall
            | DiscTrack::Mellohi
            | DiscTrack::Stal
            | DiscTrack::Strad
            | DiscTrack::Ward
            | DiscTrack::Wait => "C418",
            DiscTrack::Pigstep | DiscTrack::Otherside => "Lena Raine",
            DiscTrack::Five => "Samuel Aberg",
            DiscTrack::Relic => "Aaron Cherof",
        }
    }
}

/// Returns the duration in seconds for the given disc track.
pub fn disc_duration(track: DiscTrack) -> f32 {
    match track {
        DiscTrack::Thirteen => 178.0,
        DiscTrack::Cat => 185.0,
        DiscTrack::Blocks => 345.0,
        DiscTrack::Chirp => 185.0,
        DiscTrack::Far => 174.0,
        DiscTrack::Mall => 197.0,
        DiscTrack::Mellohi => 96.0,
        DiscTrack::Stal => 150.0,
        DiscTrack::Strad => 188.0,
        DiscTrack::Ward => 251.0,
        DiscTrack::Wait => 238.0,
        DiscTrack::Pigstep => 149.0,
        DiscTrack::Otherside => 195.0,
        DiscTrack::Five => 178.0,
        DiscTrack::Relic => 218.0,
    }
}

/// Events emitted by a jukebox during interaction and playback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JukeboxEvent {
    /// A disc has started playing.
    Started(DiscTrack),
    /// The currently playing disc has finished its full duration.
    Finished,
    /// A disc was ejected from the jukebox.
    Ejected(DiscTrack),
}

/// State of a jukebox block in the world.
#[derive(Debug, Clone, PartialEq)]
pub struct JukeboxState {
    /// The disc currently playing, if any.
    pub playing: Option<DiscTrack>,
    /// Block position of the jukebox `(x, y, z)`.
    pub position: (i32, i32, i32),
    /// Elapsed playback time in seconds.
    pub elapsed: f32,
}

impl JukeboxState {
    /// Creates a new idle jukebox at the given block position.
    pub fn new(pos: (i32, i32, i32)) -> Self {
        Self {
            playing: None,
            position: pos,
            elapsed: 0.0,
        }
    }

    /// Inserts a disc into the jukebox.
    ///
    /// If a disc is already playing it is ejected first. The returned event
    /// is always `Started` for the newly inserted disc. Callers that need to
    /// know about the ejection should check [`is_playing`](Self::is_playing)
    /// before calling this method or listen for the `Ejected` event produced
    /// by [`eject`](Self::eject).
    pub fn insert_disc(&mut self, disc: DiscTrack) -> JukeboxEvent {
        if let Some(old) = self.playing.take() {
            // Silently eject the old disc -- callers may detect this via the
            // state change.  The spec says insert_disc ejects the current disc
            // if any and starts the new one.
            let _ = old;
        }
        self.playing = Some(disc);
        self.elapsed = 0.0;
        JukeboxEvent::Started(disc)
    }

    /// Ejects the current disc, if any.
    ///
    /// Returns `Some(JukeboxEvent::Ejected(disc))` when a disc was present,
    /// or `None` if the jukebox was already empty.
    pub fn eject(&mut self) -> Option<JukeboxEvent> {
        let disc = self.playing.take()?;
        self.elapsed = 0.0;
        Some(JukeboxEvent::Ejected(disc))
    }

    /// Advances playback by `dt` seconds.
    ///
    /// Returns `Some(JukeboxEvent::Finished)` when the disc's full duration
    /// has been reached, after which the jukebox becomes idle. Returns `None`
    /// if no disc is playing or the disc has not yet finished.
    pub fn tick(&mut self, dt: f32) -> Option<JukeboxEvent> {
        let disc = self.playing?;
        self.elapsed += dt;
        if self.elapsed >= disc_duration(disc) {
            self.playing = None;
            self.elapsed = 0.0;
            Some(JukeboxEvent::Finished)
        } else {
            None
        }
    }

    /// Returns `true` if a disc is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playing.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_eject_cycle() {
        let mut jukebox = JukeboxState::new((10, 64, -20));

        // Insert a disc.
        let event = jukebox.insert_disc(DiscTrack::Cat);
        assert_eq!(event, JukeboxEvent::Started(DiscTrack::Cat));
        assert!(jukebox.is_playing());
        assert_eq!(jukebox.playing, Some(DiscTrack::Cat));

        // Eject the disc.
        let event = jukebox.eject();
        assert_eq!(event, Some(JukeboxEvent::Ejected(DiscTrack::Cat)));
        assert!(!jukebox.is_playing());
        assert_eq!(jukebox.playing, None);
    }

    #[test]
    fn eject_when_empty_returns_none() {
        let mut jukebox = JukeboxState::new((0, 0, 0));
        assert_eq!(jukebox.eject(), None);
    }

    #[test]
    fn tick_to_completion() {
        let mut jukebox = JukeboxState::new((0, 65, 0));
        jukebox.insert_disc(DiscTrack::Mellohi);

        // Mellohi is 96s -- tick partially.
        let event = jukebox.tick(50.0);
        assert_eq!(event, None);
        assert!(jukebox.is_playing());

        // Tick past the remaining duration.
        let event = jukebox.tick(50.0);
        assert_eq!(event, Some(JukeboxEvent::Finished));
        assert!(!jukebox.is_playing());
    }

    #[test]
    fn tick_when_idle_returns_none() {
        let mut jukebox = JukeboxState::new((0, 0, 0));
        assert_eq!(jukebox.tick(1.0), None);
    }

    #[test]
    fn double_insert_ejects_first() {
        let mut jukebox = JukeboxState::new((5, 60, 5));
        jukebox.insert_disc(DiscTrack::Cat);
        assert_eq!(jukebox.playing, Some(DiscTrack::Cat));

        // Insert a second disc -- the first should be replaced.
        let event = jukebox.insert_disc(DiscTrack::Stal);
        assert_eq!(event, JukeboxEvent::Started(DiscTrack::Stal));
        assert_eq!(jukebox.playing, Some(DiscTrack::Stal));
        // Elapsed is reset.
        assert!((jukebox.elapsed - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn double_insert_resets_elapsed() {
        let mut jukebox = JukeboxState::new((0, 0, 0));
        jukebox.insert_disc(DiscTrack::Blocks);
        jukebox.tick(100.0);
        assert!(jukebox.elapsed > 0.0);

        jukebox.insert_disc(DiscTrack::Far);
        assert!((jukebox.elapsed - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn duration_correctness() {
        assert!((disc_duration(DiscTrack::Thirteen) - 178.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Cat) - 185.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Blocks) - 345.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Chirp) - 185.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Far) - 174.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Mall) - 197.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Mellohi) - 96.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Stal) - 150.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Strad) - 188.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Ward) - 251.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Wait) - 238.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Pigstep) - 149.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Otherside) - 195.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Five) - 178.0).abs() < f32::EPSILON);
        assert!((disc_duration(DiscTrack::Relic) - 218.0).abs() < f32::EPSILON);
    }

    #[test]
    fn all_tracks_have_valid_metadata() {
        let tracks = [
            DiscTrack::Thirteen,
            DiscTrack::Cat,
            DiscTrack::Blocks,
            DiscTrack::Chirp,
            DiscTrack::Far,
            DiscTrack::Mall,
            DiscTrack::Mellohi,
            DiscTrack::Stal,
            DiscTrack::Strad,
            DiscTrack::Ward,
            DiscTrack::Wait,
            DiscTrack::Pigstep,
            DiscTrack::Otherside,
            DiscTrack::Five,
            DiscTrack::Relic,
        ];
        assert_eq!(tracks.len(), 15);
        for track in &tracks {
            assert!(!track.name().is_empty(), "{track:?} has empty name");
            assert!(!track.author().is_empty(), "{track:?} has empty author");
            let dur = disc_duration(*track);
            assert!(
                dur >= 60.0 && dur <= 400.0,
                "{track:?} duration {dur} out of range"
            );
        }
    }

    #[test]
    fn new_jukebox_is_idle() {
        let jukebox = JukeboxState::new((1, 2, 3));
        assert!(!jukebox.is_playing());
        assert_eq!(jukebox.playing, None);
        assert_eq!(jukebox.position, (1, 2, 3));
        assert!((jukebox.elapsed - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_boundary_exact_duration() {
        let mut jukebox = JukeboxState::new((0, 0, 0));
        jukebox.insert_disc(DiscTrack::Mellohi);

        // Tick right up to but not past the boundary.
        let event = jukebox.tick(95.9);
        assert_eq!(event, None);
        assert!(jukebox.is_playing());

        // Tick to exactly the boundary (96.0).
        let event = jukebox.tick(0.1);
        assert_eq!(event, Some(JukeboxEvent::Finished));
        assert!(!jukebox.is_playing());
    }

    #[test]
    fn eject_resets_elapsed() {
        let mut jukebox = JukeboxState::new((0, 0, 0));
        jukebox.insert_disc(DiscTrack::Ward);
        jukebox.tick(100.0);
        assert!(jukebox.elapsed > 0.0);

        jukebox.eject();
        assert!((jukebox.elapsed - 0.0).abs() < f32::EPSILON);
    }
}
