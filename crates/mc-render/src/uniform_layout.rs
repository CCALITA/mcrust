//! Uniform buffer layout manager for organizing GPU uniform bindings.

/// A single uniform buffer slot in a layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniformSlot {
    pub binding: u32,
    pub size: u64,
    pub name: &'static str,
}

/// A collection of uniform buffer slots forming a complete layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniformLayout {
    pub slots: Vec<UniformSlot>,
}

/// Layout for terrain rendering: camera and sky uniforms.
pub fn terrain_layout() -> UniformLayout {
    UniformLayout {
        slots: vec![
            UniformSlot { binding: 0, size: 64, name: "camera" },
            UniformSlot { binding: 2, size: 32, name: "sky" },
        ],
    }
}

/// Layout for terrain rendering with fog: adds fog uniform to terrain layout.
pub fn terrain_with_fog_layout() -> UniformLayout {
    let mut slots = terrain_layout().slots;
    slots.push(UniformSlot { binding: 3, size: 32, name: "fog" });
    UniformLayout { slots }
}

/// Layout for overlay rendering: screen size uniform only.
pub fn overlay_layout() -> UniformLayout {
    UniformLayout {
        slots: vec![
            UniformSlot { binding: 0, size: 8, name: "screen_size" },
        ],
    }
}

/// Layout for water rendering: camera, sky, and water uniforms.
pub fn water_layout() -> UniformLayout {
    UniformLayout {
        slots: vec![
            UniformSlot { binding: 0, size: 64, name: "camera" },
            UniformSlot { binding: 1, size: 32, name: "sky" },
            UniformSlot { binding: 2, size: 16, name: "water" },
        ],
    }
}

/// Compute total bytes needed for all uniform buffers in a layout.
pub fn total_uniform_bytes(layout: &UniformLayout) -> u64 {
    layout.slots.iter().map(|s| s.size).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_layout() {
        let layout = terrain_layout();
        assert_eq!(layout.slots.len(), 2);
        assert_eq!(layout.slots[0].name, "camera");
        assert_eq!(layout.slots[0].binding, 0);
        assert_eq!(layout.slots[0].size, 64);
        assert_eq!(layout.slots[1].name, "sky");
        assert_eq!(layout.slots[1].binding, 2);
        assert_eq!(layout.slots[1].size, 32);
    }

    #[test]
    fn test_terrain_with_fog_layout() {
        let layout = terrain_with_fog_layout();
        assert_eq!(layout.slots.len(), 3);
        assert_eq!(layout.slots[2].name, "fog");
        assert_eq!(layout.slots[2].binding, 3);
        assert_eq!(layout.slots[2].size, 32);
    }

    #[test]
    fn test_overlay_layout() {
        let layout = overlay_layout();
        assert_eq!(layout.slots.len(), 1);
        assert_eq!(layout.slots[0].name, "screen_size");
        assert_eq!(layout.slots[0].size, 8);
    }

    #[test]
    fn test_water_layout() {
        let layout = water_layout();
        assert_eq!(layout.slots.len(), 3);
        assert_eq!(layout.slots[0].name, "camera");
        assert_eq!(layout.slots[1].name, "sky");
        assert_eq!(layout.slots[1].binding, 1);
        assert_eq!(layout.slots[2].name, "water");
        assert_eq!(layout.slots[2].size, 16);
    }

    #[test]
    fn test_total_uniform_bytes() {
        assert_eq!(total_uniform_bytes(&terrain_layout()), 96);
        assert_eq!(total_uniform_bytes(&terrain_with_fog_layout()), 128);
        assert_eq!(total_uniform_bytes(&overlay_layout()), 8);
        assert_eq!(total_uniform_bytes(&water_layout()), 112);
    }
}
