use serde::{Deserialize, Serialize};

/// Persistent game settings, serialized to/from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub render_distance: u32,
    pub fov: f32,
    pub mouse_sensitivity: f32,
    pub music_volume: f32,
    pub sound_volume: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub gui_scale: u8,
    pub max_fps: u32,
    pub username: String,
    pub seed: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            render_distance: 8,
            fov: 70.0,
            mouse_sensitivity: 0.003,
            music_volume: 0.5,
            sound_volume: 1.0,
            fullscreen: false,
            vsync: true,
            gui_scale: 2,
            max_fps: 60,
            username: "Player".to_string(),
            seed: 42,
        }
    }
}

impl GameSettings {
    /// Load settings from a TOML file.  Returns defaults when the file is
    /// missing or cannot be parsed.  Values are clamped to valid ranges
    /// after loading.
    pub fn load(path: &str) -> Self {
        let mut settings = match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<GameSettings>(&content) {
                Ok(s) => {
                    log::info!("Loaded settings from {path}");
                    s
                }
                Err(e) => {
                    log::warn!("Failed to parse {path}: {e} — using defaults");
                    Self::default()
                }
            },
            Err(_) => {
                log::info!("No settings file at {path} — using defaults");
                Self::default()
            }
        };
        settings.validate();
        settings
    }

    /// Serialize and write settings to a TOML file.
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(format!("TOML serialize error: {e}")))?;
        std::fs::write(path, content)
    }

    /// Clamp all values to their valid ranges.
    pub fn validate(&mut self) {
        self.render_distance = self.render_distance.clamp(2, 32);
        self.fov = self.fov.clamp(30.0, 110.0);
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(0.001, 0.01);
        self.music_volume = self.music_volume.clamp(0.0, 1.0);
        self.sound_volume = self.sound_volume.clamp(0.0, 1.0);
        self.gui_scale = self.gui_scale.clamp(1, 4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_are_valid() {
        let mut settings = GameSettings::default();
        let before = settings.clone();
        settings.validate();
        // All defaults should survive validation unchanged.
        assert_eq!(settings.render_distance, before.render_distance);
        assert_eq!(settings.fov, before.fov);
        assert_eq!(settings.mouse_sensitivity, before.mouse_sensitivity);
        assert_eq!(settings.music_volume, before.music_volume);
        assert_eq!(settings.sound_volume, before.sound_volume);
        assert_eq!(settings.gui_scale, before.gui_scale);
        assert_eq!(settings.fullscreen, before.fullscreen);
        assert_eq!(settings.vsync, before.vsync);
        assert_eq!(settings.max_fps, before.max_fps);
        assert_eq!(settings.username, before.username);
        assert_eq!(settings.seed, before.seed);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join("mcrust_test_settings");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_settings.toml");
        let path_str = path.to_str().unwrap();

        let original = GameSettings {
            render_distance: 16,
            fov: 90.0,
            mouse_sensitivity: 0.005,
            music_volume: 0.3,
            sound_volume: 0.8,
            fullscreen: true,
            vsync: false,
            gui_scale: 3,
            max_fps: 120,
            username: "TestPlayer".to_string(),
            seed: 12345,
        };
        original.save(path_str).unwrap();

        let loaded = GameSettings::load(path_str);
        assert_eq!(loaded.render_distance, original.render_distance);
        assert_eq!(loaded.fov, original.fov);
        assert_eq!(loaded.mouse_sensitivity, original.mouse_sensitivity);
        assert_eq!(loaded.music_volume, original.music_volume);
        assert_eq!(loaded.sound_volume, original.sound_volume);
        assert_eq!(loaded.fullscreen, original.fullscreen);
        assert_eq!(loaded.vsync, original.vsync);
        assert_eq!(loaded.gui_scale, original.gui_scale);
        assert_eq!(loaded.max_fps, original.max_fps);
        assert_eq!(loaded.username, original.username);
        assert_eq!(loaded.seed, original.seed);

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let settings = GameSettings::load("/tmp/mcrust_nonexistent_settings_file.toml");
        let defaults = GameSettings::default();
        assert_eq!(settings.render_distance, defaults.render_distance);
        assert_eq!(settings.fov, defaults.fov);
        assert_eq!(settings.seed, defaults.seed);
    }

    #[test]
    fn invalid_values_get_clamped() {
        let mut settings = GameSettings {
            render_distance: 100,
            fov: 200.0,
            mouse_sensitivity: 0.0,
            music_volume: -1.0,
            sound_volume: 5.0,
            gui_scale: 0,
            ..GameSettings::default()
        };
        settings.validate();
        assert_eq!(settings.render_distance, 32);
        assert_eq!(settings.fov, 110.0);
        assert_eq!(settings.mouse_sensitivity, 0.001);
        assert_eq!(settings.music_volume, 0.0);
        assert_eq!(settings.sound_volume, 1.0);
        assert_eq!(settings.gui_scale, 1);
    }
}
