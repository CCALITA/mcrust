/// Music disc that can be played in a jukebox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MusicDisc {
    Disc13,
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
    Otherside,
    Pigstep,
}

impl MusicDisc {
    /// Returns the display name of the disc.
    pub fn name(&self) -> &'static str {
        match self {
            MusicDisc::Disc13 => "13",
            MusicDisc::Cat => "cat",
            MusicDisc::Blocks => "blocks",
            MusicDisc::Chirp => "chirp",
            MusicDisc::Far => "far",
            MusicDisc::Mall => "mall",
            MusicDisc::Mellohi => "mellohi",
            MusicDisc::Stal => "stal",
            MusicDisc::Strad => "strad",
            MusicDisc::Ward => "ward",
            MusicDisc::Wait => "wait",
            MusicDisc::Otherside => "otherside",
            MusicDisc::Pigstep => "Pigstep",
        }
    }

    /// Returns the author/artist of the disc.
    pub fn author(&self) -> &'static str {
        match self {
            MusicDisc::Disc13 => "C418",
            MusicDisc::Cat => "C418",
            MusicDisc::Blocks => "C418",
            MusicDisc::Chirp => "C418",
            MusicDisc::Far => "C418",
            MusicDisc::Mall => "C418",
            MusicDisc::Mellohi => "C418",
            MusicDisc::Stal => "C418",
            MusicDisc::Strad => "C418",
            MusicDisc::Ward => "C418",
            MusicDisc::Wait => "C418",
            MusicDisc::Otherside => "Lena Raine",
            MusicDisc::Pigstep => "Lena Raine",
        }
    }

    /// Returns the duration of the disc in seconds.
    pub fn duration_secs(&self) -> f32 {
        match self {
            MusicDisc::Disc13 => 178.0,
            MusicDisc::Cat => 185.0,
            MusicDisc::Blocks => 345.0,
            MusicDisc::Chirp => 185.0,
            MusicDisc::Far => 174.0,
            MusicDisc::Mall => 197.0,
            MusicDisc::Mellohi => 96.0,
            MusicDisc::Stal => 150.0,
            MusicDisc::Strad => 188.0,
            MusicDisc::Ward => 251.0,
            MusicDisc::Wait => 238.0,
            MusicDisc::Otherside => 195.0,
            MusicDisc::Pigstep => 149.0,
        }
    }
}

/// Events emitted by the disc player each tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiscEvent {
    /// No state change this tick.
    None,
    /// A disc started playing.
    Started(MusicDisc),
    /// The currently playing disc has finished.
    Finished,
}

/// Jukebox-style disc player that tracks playback of a single music disc.
#[derive(Debug, Clone)]
pub struct DiscPlayer {
    /// The currently playing disc and its elapsed time, or `None` if idle.
    playing: Option<(MusicDisc, f32)>,
}

impl DiscPlayer {
    /// Creates a new disc player with nothing playing.
    pub fn new() -> Self {
        Self { playing: None }
    }

    /// Starts playing the given disc from the beginning.
    ///
    /// If a disc is already playing it is replaced.
    pub fn play(&mut self, disc: MusicDisc) {
        self.playing = Some((disc, 0.0));
    }

    /// Stops the currently playing disc, if any.
    pub fn stop(&mut self) {
        self.playing = None;
    }

    /// Returns `true` if a disc is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playing.is_some()
    }

    /// Returns the currently playing disc, if any.
    pub fn current_disc(&self) -> Option<MusicDisc> {
        self.playing.map(|(disc, _)| disc)
    }

    /// Advances playback by `dt` seconds and returns the resulting event.
    ///
    /// Returns [`DiscEvent::Finished`] when the disc's duration is exceeded,
    /// after which the player becomes idle.
    pub fn tick(&mut self, dt: f32) -> DiscEvent {
        let Some((disc, elapsed)) = &mut self.playing else {
            return DiscEvent::None;
        };
        *elapsed += dt;
        if *elapsed >= disc.duration_secs() {
            self.playing = None;
            DiscEvent::Finished
        } else {
            DiscEvent::None
        }
    }
}

impl Default for DiscPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a Minecraft item ID to its corresponding music disc.
///
/// Item IDs follow the standard Minecraft mapping (2256-2268).
pub fn disc_from_item_id(id: u16) -> Option<MusicDisc> {
    match id {
        2256 => Some(MusicDisc::Disc13),
        2257 => Some(MusicDisc::Cat),
        2258 => Some(MusicDisc::Blocks),
        2259 => Some(MusicDisc::Chirp),
        2260 => Some(MusicDisc::Far),
        2261 => Some(MusicDisc::Mall),
        2262 => Some(MusicDisc::Mellohi),
        2263 => Some(MusicDisc::Stal),
        2264 => Some(MusicDisc::Strad),
        2265 => Some(MusicDisc::Ward),
        2266 => Some(MusicDisc::Wait),
        2267 => Some(MusicDisc::Otherside),
        2268 => Some(MusicDisc::Pigstep),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_and_stop() {
        let mut player = DiscPlayer::new();
        assert!(!player.is_playing());
        assert_eq!(player.current_disc(), None);

        player.play(MusicDisc::Cat);
        assert!(player.is_playing());
        assert_eq!(player.current_disc(), Some(MusicDisc::Cat));

        player.stop();
        assert!(!player.is_playing());
        assert_eq!(player.current_disc(), None);
    }

    #[test]
    fn play_replaces_current_disc() {
        let mut player = DiscPlayer::new();
        player.play(MusicDisc::Cat);
        player.play(MusicDisc::Stal);
        assert_eq!(player.current_disc(), Some(MusicDisc::Stal));
    }

    #[test]
    fn tick_advances_time() {
        let mut player = DiscPlayer::new();
        player.play(MusicDisc::Mellohi);

        // Mellohi is 96s; tick 50s should not finish.
        let event = player.tick(50.0);
        assert_eq!(event, DiscEvent::None);
        assert!(player.is_playing());
    }

    #[test]
    fn tick_finishes_when_duration_exceeded() {
        let mut player = DiscPlayer::new();
        player.play(MusicDisc::Mellohi);

        // Mellohi is 96s; tick past the duration.
        let event = player.tick(100.0);
        assert_eq!(event, DiscEvent::Finished);
        assert!(!player.is_playing());
    }

    #[test]
    fn tick_returns_none_when_idle() {
        let mut player = DiscPlayer::new();
        let event = player.tick(1.0);
        assert_eq!(event, DiscEvent::None);
    }

    #[test]
    fn duration_finish_boundary() {
        let mut player = DiscPlayer::new();
        player.play(MusicDisc::Mellohi);

        // Tick right up to the boundary.
        let event = player.tick(95.9);
        assert_eq!(event, DiscEvent::None);
        assert!(player.is_playing());

        // Tick past the boundary.
        let event = player.tick(0.2);
        assert_eq!(event, DiscEvent::Finished);
        assert!(!player.is_playing());
    }

    #[test]
    fn disc_names_are_non_empty() {
        let discs = [
            MusicDisc::Disc13,
            MusicDisc::Cat,
            MusicDisc::Blocks,
            MusicDisc::Chirp,
            MusicDisc::Far,
            MusicDisc::Mall,
            MusicDisc::Mellohi,
            MusicDisc::Stal,
            MusicDisc::Strad,
            MusicDisc::Ward,
            MusicDisc::Wait,
            MusicDisc::Otherside,
            MusicDisc::Pigstep,
        ];
        assert_eq!(discs.len(), 13);
        for disc in &discs {
            assert!(!disc.name().is_empty(), "{disc:?} has empty name");
            assert!(!disc.author().is_empty(), "{disc:?} has empty author");
            assert!(
                disc.duration_secs() >= 60.0 && disc.duration_secs() <= 400.0,
                "{disc:?} duration {} out of range",
                disc.duration_secs()
            );
        }
    }

    #[test]
    fn item_id_mapping() {
        assert_eq!(disc_from_item_id(2256), Some(MusicDisc::Disc13));
        assert_eq!(disc_from_item_id(2257), Some(MusicDisc::Cat));
        assert_eq!(disc_from_item_id(2268), Some(MusicDisc::Pigstep));
        assert_eq!(disc_from_item_id(0), None);
        assert_eq!(disc_from_item_id(9999), None);
    }

    #[test]
    fn item_id_maps_all_13_discs() {
        let mut count = 0;
        for id in 2256..=2268 {
            assert!(
                disc_from_item_id(id).is_some(),
                "item id {id} should map to a disc"
            );
            count += 1;
        }
        assert_eq!(count, 13);
    }

    #[test]
    fn play_emits_started_on_next_concept() {
        // Verify that after calling play, current_disc reflects the new disc.
        let mut player = DiscPlayer::new();
        player.play(MusicDisc::Ward);
        assert_eq!(player.current_disc(), Some(MusicDisc::Ward));
    }

    #[test]
    fn stop_when_not_playing_is_noop() {
        let mut player = DiscPlayer::new();
        player.stop();
        assert!(!player.is_playing());
    }

    #[test]
    fn default_creates_idle_player() {
        let player = DiscPlayer::default();
        assert!(!player.is_playing());
        assert_eq!(player.current_disc(), None);
    }
}
