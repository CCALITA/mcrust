use std::fs;
use std::path::Path;

use mc_world::save::{WorldSave, load_world, save_world};

// ---------------------------------------------------------------------------
// SaveData — client-facing subset of WorldSave
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SaveData {
    pub player_pos: (f32, f32, f32),
    pub player_yaw: f32,
    pub player_pitch: f32,
    pub time_of_day: f32,
    pub seed: u64,
}

// ---------------------------------------------------------------------------
// SaveSystem — bridge between mc-client and mc-world save/load
// ---------------------------------------------------------------------------

pub struct SaveSystem {
    save_dir: String,
    auto_save_timer: f32,
    auto_save_interval: f32,
}

impl SaveSystem {
    pub fn new(save_dir: &str) -> Self {
        Self {
            save_dir: save_dir.to_string(),
            auto_save_timer: 0.0,
            auto_save_interval: 300.0,
        }
    }

    /// Advances the auto-save timer by `dt` seconds. Returns `true` when an
    /// auto-save is due (the timer resets automatically).
    pub fn tick(&mut self, dt: f32) -> bool {
        self.auto_save_timer += dt;
        if self.auto_save_timer >= self.auto_save_interval {
            self.auto_save_timer -= self.auto_save_interval;
            true
        } else {
            false
        }
    }

    /// Persists the current game state to `<save_dir>/world.bin`.
    pub fn save_game(
        &self,
        player_pos: (f32, f32, f32),
        player_yaw: f32,
        player_pitch: f32,
        time_of_day: f32,
        seed: u64,
    ) {
        let world_save = WorldSave {
            seed,
            player_pos: [player_pos.0, player_pos.1, player_pos.2],
            player_yaw,
            player_pitch,
            time_of_day,
        };

        let path = Path::new(&self.save_dir).join("world.bin");
        if let Err(e) = save_world(&path, &world_save) {
            log::error!("Failed to save game: {e}");
        }
    }

    /// Loads the game state from `<save_dir>/world.bin`, returning `None` if
    /// the file does not exist or cannot be read.
    pub fn load_game(&self) -> Option<SaveData> {
        let path = Path::new(&self.save_dir).join("world.bin");
        match load_world(&path) {
            Ok(ws) => Some(SaveData {
                player_pos: (ws.player_pos[0], ws.player_pos[1], ws.player_pos[2]),
                player_yaw: ws.player_yaw,
                player_pitch: ws.player_pitch,
                time_of_day: ws.time_of_day,
                seed: ws.seed,
            }),
            Err(e) => {
                log::warn!("Failed to load game: {e}");
                None
            }
        }
    }

    /// Returns `true` when a save file exists at `<save_dir>/world.bin`.
    pub fn save_exists(&self) -> bool {
        Path::new(&self.save_dir).join("world.bin").exists()
    }

    /// Deletes the save file at `<save_dir>/world.bin` if it exists.
    pub fn delete_save(&self) {
        let path = Path::new(&self.save_dir).join("world.bin");
        if path.exists()
            && let Err(e) = fs::remove_file(&path)
        {
            log::error!("Failed to delete save: {e}");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_save_dir(name: &str) -> String {
        let dir = std::env::temp_dir()
            .join(format!("mcrust_save_test_{name}"))
            .to_string_lossy()
            .to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn new_system_has_correct_interval() {
        let sys = SaveSystem::new("/tmp/test");
        assert_eq!(sys.auto_save_interval, 300.0);
        assert_eq!(sys.auto_save_timer, 0.0);
    }

    #[test]
    fn tick_returns_true_when_due() {
        let mut sys = SaveSystem::new("/tmp/test");
        assert!(!sys.tick(100.0));
        assert!(!sys.tick(100.0));
        assert!(sys.tick(100.0)); // 300 total
    }

    #[test]
    fn tick_resets_timer_after_trigger() {
        let mut sys = SaveSystem::new("/tmp/test");
        assert!(sys.tick(350.0)); // triggers at 300, 50 left over
        // Next trigger should be at 250 more seconds (300 - 50)
        assert!(!sys.tick(200.0));
        assert!(sys.tick(50.0)); // 250 + 50 = 300
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = temp_save_dir("round_trip");
        let sys = SaveSystem::new(&dir);

        sys.save_game((1.0, 64.0, -3.0), 90.0, -15.0, 6000.0, 42);

        let data = sys.load_game().expect("should load saved game");
        assert_eq!(data.player_pos, (1.0, 64.0, -3.0));
        assert_eq!(data.player_yaw, 90.0);
        assert_eq!(data.player_pitch, -15.0);
        assert_eq!(data.time_of_day, 6000.0);
        assert_eq!(data.seed, 42);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_game_returns_none_when_missing() {
        let dir = temp_save_dir("missing");
        let sys = SaveSystem::new(&dir);
        assert!(sys.load_game().is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_exists_reflects_file_presence() {
        let dir = temp_save_dir("exists");
        let sys = SaveSystem::new(&dir);

        assert!(!sys.save_exists());
        sys.save_game((0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 1);
        assert!(sys.save_exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_save_removes_file() {
        let dir = temp_save_dir("delete");
        let sys = SaveSystem::new(&dir);

        sys.save_game((0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 1);
        assert!(sys.save_exists());

        sys.delete_save();
        assert!(!sys.save_exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_save_is_no_op_when_missing() {
        let dir = temp_save_dir("delete_noop");
        let sys = SaveSystem::new(&dir);
        sys.delete_save(); // should not panic
        let _ = fs::remove_dir_all(&dir);
    }
}
