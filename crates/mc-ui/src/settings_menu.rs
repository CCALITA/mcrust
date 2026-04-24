//! Settings menu data model: render distance, FOV, audio, input, video, and
//! particle preferences with INI-style serialization.

/// Particle rendering preference, mirroring vanilla Minecraft options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleSetting {
    All,
    Decreased,
    Minimal,
}

impl ParticleSetting {
    fn as_key(self) -> &'static str {
        match self {
            ParticleSetting::All => "all",
            ParticleSetting::Decreased => "decreased",
            ParticleSetting::Minimal => "minimal",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key.trim().to_ascii_lowercase().as_str() {
            "all" => Some(ParticleSetting::All),
            "decreased" => Some(ParticleSetting::Decreased),
            "minimal" => Some(ParticleSetting::Minimal),
            _ => None,
        }
    }
}

/// Player-tunable settings persisted between sessions.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub render_distance: u8,
    pub fov: u8,
    pub music_volume: f32,
    pub sound_volume: f32,
    pub mouse_sensitivity: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub smooth_lighting: bool,
    pub particles: ParticleSetting,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 8,
            fov: 70,
            music_volume: 0.5,
            sound_volume: 1.0,
            mouse_sensitivity: 0.5,
            fullscreen: false,
            vsync: true,
            smooth_lighting: true,
            particles: ParticleSetting::All,
        }
    }
}

/// Clamp render distance to the supported chunk range [2, 32].
pub fn apply_render_distance(rd: u8) -> u8 {
    rd.clamp(2, 32)
}

/// Clamp FOV to the supported degree range [30, 110].
pub fn apply_fov(fov: u8) -> u8 {
    fov.clamp(30, 110)
}

/// Validate a `Settings` struct, returning a list of human-readable issues.
/// An empty vector means the settings are valid.
pub fn validate_settings(s: &Settings) -> Vec<String> {
    let mut issues = Vec::new();
    if s.render_distance < 2 || s.render_distance > 32 {
        issues.push(format!(
            "render_distance {} out of range [2, 32]",
            s.render_distance
        ));
    }
    if s.fov < 30 || s.fov > 110 {
        issues.push(format!("fov {} out of range [30, 110]", s.fov));
    }
    if !(0.0..=1.0).contains(&s.music_volume) {
        issues.push(format!(
            "music_volume {} out of range [0.0, 1.0]",
            s.music_volume
        ));
    }
    if !(0.0..=1.0).contains(&s.sound_volume) {
        issues.push(format!(
            "sound_volume {} out of range [0.0, 1.0]",
            s.sound_volume
        ));
    }
    if !(0.0..=1.0).contains(&s.mouse_sensitivity) {
        issues.push(format!(
            "mouse_sensitivity {} out of range [0.0, 1.0]",
            s.mouse_sensitivity
        ));
    }
    issues
}

/// Serialize settings into a deterministic INI-like `key=value\n` blob.
pub fn serialize_settings(s: &Settings) -> String {
    let mut out = String::new();
    out.push_str(&format!("render_distance={}\n", s.render_distance));
    out.push_str(&format!("fov={}\n", s.fov));
    out.push_str(&format!("music_volume={}\n", s.music_volume));
    out.push_str(&format!("sound_volume={}\n", s.sound_volume));
    out.push_str(&format!("mouse_sensitivity={}\n", s.mouse_sensitivity));
    out.push_str(&format!("fullscreen={}\n", s.fullscreen));
    out.push_str(&format!("vsync={}\n", s.vsync));
    out.push_str(&format!("smooth_lighting={}\n", s.smooth_lighting));
    out.push_str(&format!("particles={}\n", s.particles.as_key()));
    out
}

/// Parse INI-like settings text. Unknown keys are ignored, and missing or
/// malformed values fall back to `Settings::default()` per-field.
pub fn parse_settings(text: &str) -> Settings {
    let mut s = Settings::default();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "render_distance" => {
                if let Ok(v) = value.parse::<u8>() {
                    s.render_distance = v;
                }
            }
            "fov" => {
                if let Ok(v) = value.parse::<u8>() {
                    s.fov = v;
                }
            }
            "music_volume" => {
                if let Ok(v) = value.parse::<f32>() {
                    s.music_volume = v;
                }
            }
            "sound_volume" => {
                if let Ok(v) = value.parse::<f32>() {
                    s.sound_volume = v;
                }
            }
            "mouse_sensitivity" => {
                if let Ok(v) = value.parse::<f32>() {
                    s.mouse_sensitivity = v;
                }
            }
            "fullscreen" => {
                if let Ok(v) = value.parse::<bool>() {
                    s.fullscreen = v;
                }
            }
            "vsync" => {
                if let Ok(v) = value.parse::<bool>() {
                    s.vsync = v;
                }
            }
            "smooth_lighting" => {
                if let Ok(v) = value.parse::<bool>() {
                    s.smooth_lighting = v;
                }
            }
            "particles" => {
                if let Some(p) = ParticleSetting::from_key(value) {
                    s.particles = p;
                }
            }
            _ => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let s = Settings::default();
        assert_eq!(s.render_distance, 8);
        assert_eq!(s.fov, 70);
        assert_eq!(s.music_volume, 0.5);
        assert_eq!(s.sound_volume, 1.0);
        assert_eq!(s.mouse_sensitivity, 0.5);
        assert!(!s.fullscreen);
        assert!(s.vsync);
        assert!(s.smooth_lighting);
        assert_eq!(s.particles, ParticleSetting::All);
    }

    #[test]
    fn render_distance_clamps_low_and_high() {
        assert_eq!(apply_render_distance(0), 2);
        assert_eq!(apply_render_distance(1), 2);
        assert_eq!(apply_render_distance(8), 8);
        assert_eq!(apply_render_distance(32), 32);
        assert_eq!(apply_render_distance(64), 32);
    }

    #[test]
    fn fov_clamps_low_and_high() {
        assert_eq!(apply_fov(0), 30);
        assert_eq!(apply_fov(29), 30);
        assert_eq!(apply_fov(70), 70);
        assert_eq!(apply_fov(110), 110);
        assert_eq!(apply_fov(200), 110);
    }

    #[test]
    fn validate_clean_settings_yields_no_issues() {
        let s = Settings::default();
        assert!(validate_settings(&s).is_empty());
    }

    #[test]
    fn validate_flags_out_of_range_fields() {
        let s = Settings {
            render_distance: 1,
            fov: 200,
            music_volume: -0.1,
            sound_volume: 2.0,
            mouse_sensitivity: 1.5,
            ..Settings::default()
        };
        let issues = validate_settings(&s);
        assert_eq!(issues.len(), 5);
        assert!(issues.iter().any(|i| i.contains("render_distance")));
        assert!(issues.iter().any(|i| i.contains("fov")));
        assert!(issues.iter().any(|i| i.contains("music_volume")));
        assert!(issues.iter().any(|i| i.contains("sound_volume")));
        assert!(issues.iter().any(|i| i.contains("mouse_sensitivity")));
    }

    #[test]
    fn serialize_round_trips_through_parse() {
        let original = Settings {
            render_distance: 16,
            fov: 90,
            music_volume: 0.25,
            sound_volume: 0.75,
            mouse_sensitivity: 0.6,
            fullscreen: true,
            vsync: false,
            smooth_lighting: false,
            particles: ParticleSetting::Minimal,
        };
        let text = serialize_settings(&original);
        let parsed = parse_settings(&text);
        assert_eq!(parsed, original);
    }

    #[test]
    fn serialize_emits_expected_keys() {
        let text = serialize_settings(&Settings::default());
        assert!(text.contains("render_distance=8\n"));
        assert!(text.contains("fov=70\n"));
        assert!(text.contains("particles=all\n"));
        assert!(text.contains("vsync=true\n"));
        assert!(text.contains("fullscreen=false\n"));
    }

    #[test]
    fn parse_ignores_blank_lines_and_comments() {
        let text = "\n# comment\n; also comment\nrender_distance=12\n";
        let s = parse_settings(text);
        assert_eq!(s.render_distance, 12);
        assert_eq!(s.fov, Settings::default().fov);
    }

    #[test]
    fn parse_ignores_unknown_keys_and_bad_values() {
        let text = "unknown=foo\nfov=not_a_number\nfullscreen=true\n";
        let s = parse_settings(text);
        assert_eq!(s.fov, Settings::default().fov);
        assert!(s.fullscreen);
    }

    #[test]
    fn parse_particles_variants() {
        assert_eq!(
            parse_settings("particles=decreased\n").particles,
            ParticleSetting::Decreased
        );
        assert_eq!(
            parse_settings("particles=Minimal\n").particles,
            ParticleSetting::Minimal
        );
        assert_eq!(
            parse_settings("particles=garbage\n").particles,
            ParticleSetting::All
        );
    }
}
