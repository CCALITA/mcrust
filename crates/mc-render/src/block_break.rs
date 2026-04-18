#[derive(Debug, Default)]
pub struct BlockBreakOverlay {
    pub block_pos: Option<(i32, i32, i32)>,
    pub progress: f32,
    pub stage: Option<u8>,
    pub active: bool,
}

impl BlockBreakOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, pos: (i32, i32, i32), progress: f32) {
        self.block_pos = Some(pos);
        self.progress = progress;
        self.stage = Some((progress * 10.0).min(9.0) as u8);
        self.active = true;
    }

    pub fn clear(&mut self) {
        self.block_pos = None;
        self.progress = 0.0;
        self.stage = None;
        self.active = false;
    }
}
