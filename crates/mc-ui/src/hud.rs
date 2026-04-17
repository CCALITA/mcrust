use crate::widget::{Color, Rect, UiContext};

#[cfg(test)]
use crate::widget::DrawCommand;

// ---------------------------------------------------------------------------
// Layout constants — all proportional to normalized screen space (0.0..1.0)
// ---------------------------------------------------------------------------

/// Width of one hotbar slot as a fraction of screen width.
const SLOT_SIZE: f32 = 0.04;
/// Total hotbar width: 9 slots.
const HOTBAR_WIDTH: f32 = SLOT_SIZE * 9.0;
/// Vertical position of the hotbar (bottom of screen, with a small margin).
const HOTBAR_Y: f32 = 0.92;
/// Size of a single heart / hunger / armor icon.
const ICON_SIZE: f32 = 0.018;
/// Spacing between icons.
const ICON_SPACING: f32 = 0.02;
/// Height of the XP bar.
const XP_BAR_HEIGHT: f32 = 0.008;
/// Width of the crosshair arms.
const CROSSHAIR_ARM_LENGTH: f32 = 0.012;
/// Thickness of the crosshair.
const CROSSHAIR_THICKNESS: f32 = 0.003;

/// Background tint for hotbar slots.
const SLOT_BG_COLOR: Color = Color::new(0.2, 0.2, 0.2, 0.7);
/// Highlight tint for the selected hotbar slot.
const SLOT_SELECTED_COLOR: Color = Color::new(0.8, 0.8, 0.8, 0.8);
/// Heart color (red).
const HEART_COLOR: Color = Color::new(0.85, 0.1, 0.1, 1.0);
/// Empty heart outline color.
const HEART_EMPTY_COLOR: Color = Color::new(0.3, 0.0, 0.0, 0.6);
/// Half-heart color (slightly dimmer).
const HEART_HALF_COLOR: Color = Color::new(0.85, 0.1, 0.1, 0.5);
/// Hunger color (brown / drumstick).
const HUNGER_COLOR: Color = Color::new(0.75, 0.55, 0.2, 1.0);
/// Empty hunger outline.
const HUNGER_EMPTY_COLOR: Color = Color::new(0.35, 0.25, 0.1, 0.6);
/// Armor color (light blue).
const ARMOR_COLOR: Color = Color::new(0.6, 0.7, 0.85, 1.0);
/// XP bar filled color.
const XP_BAR_COLOR: Color = Color::new(0.3, 0.85, 0.1, 1.0);
/// XP bar background.
const XP_BAR_BG_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);
/// Crosshair color.
const CROSSHAIR_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.8);
/// Debug overlay text color.
const DEBUG_TEXT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.9);

// ---------------------------------------------------------------------------
// HudState
// ---------------------------------------------------------------------------

/// Snapshot of all HUD-visible player data for a single frame.
#[derive(Debug, Clone)]
pub struct HudState {
    /// Current health (0.0..=20.0, displayed as 10 hearts).
    pub health: f32,
    /// Maximum health (usually 20.0).
    pub max_health: f32,
    /// Current hunger (0..=20, displayed as 10 drumsticks).
    pub hunger: u32,
    /// Current armor (0..=20, displayed as 10 armor icons).
    pub armor: u32,
    /// Experience level.
    pub xp_level: u32,
    /// Experience bar progress (0.0..=1.0).
    pub xp_progress: f32,
    /// Currently selected hotbar slot (0..=8).
    pub selected_slot: usize,
    /// Whether the debug overlay (F3 screen) is visible.
    pub show_debug: bool,
    /// Player world position for the debug screen.
    pub player_pos: (f32, f32, f32),
    /// Frames per second for the debug screen.
    pub fps: u32,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            hunger: 20,
            armor: 0,
            xp_level: 0,
            xp_progress: 0.0,
            selected_slot: 0,
            show_debug: false,
            player_pos: (0.0, 0.0, 0.0),
            fps: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// HudRenderer
// ---------------------------------------------------------------------------

/// Stateless renderer that converts `HudState` into draw commands via `UiContext`.
pub struct HudRenderer;

impl HudRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Generate all HUD draw commands for a single frame.
    pub fn render(&self, state: &HudState, ctx: &mut UiContext) {
        self.render_crosshair(ctx);
        self.render_hotbar(state, ctx);
        self.render_xp_bar(state, ctx);
        self.render_health(state, ctx);
        self.render_hunger(state, ctx);
        if state.armor > 0 {
            self.render_armor(state, ctx);
        }
        if state.show_debug {
            self.render_debug(state, ctx);
        }
    }

    // -- Crosshair --------------------------------------------------------

    fn render_crosshair(&self, ctx: &mut UiContext) {
        let cx = 0.5;
        let cy = 0.5;

        // Horizontal bar
        ctx.draw_quad(
            Rect::new(
                cx - CROSSHAIR_ARM_LENGTH,
                cy - CROSSHAIR_THICKNESS / 2.0,
                CROSSHAIR_ARM_LENGTH * 2.0,
                CROSSHAIR_THICKNESS,
            ),
            CROSSHAIR_COLOR,
        );
        // Vertical bar
        ctx.draw_quad(
            Rect::new(
                cx - CROSSHAIR_THICKNESS / 2.0,
                cy - CROSSHAIR_ARM_LENGTH,
                CROSSHAIR_THICKNESS,
                CROSSHAIR_ARM_LENGTH * 2.0,
            ),
            CROSSHAIR_COLOR,
        );
    }

    // -- Hotbar -----------------------------------------------------------

    fn render_hotbar(&self, state: &HudState, ctx: &mut UiContext) {
        let start_x = 0.5 - HOTBAR_WIDTH / 2.0;

        for i in 0..9 {
            let x = start_x + (i as f32) * SLOT_SIZE;
            let color = if i == state.selected_slot {
                SLOT_SELECTED_COLOR
            } else {
                SLOT_BG_COLOR
            };
            ctx.draw_quad(Rect::new(x, HOTBAR_Y, SLOT_SIZE, SLOT_SIZE), color);
        }
    }

    // -- XP bar -----------------------------------------------------------

    fn render_xp_bar(&self, state: &HudState, ctx: &mut UiContext) {
        let bar_y = HOTBAR_Y - XP_BAR_HEIGHT - 0.005;
        let start_x = 0.5 - HOTBAR_WIDTH / 2.0;

        // Background
        ctx.draw_quad(
            Rect::new(start_x, bar_y, HOTBAR_WIDTH, XP_BAR_HEIGHT),
            XP_BAR_BG_COLOR,
        );

        // Filled portion
        let filled_width = HOTBAR_WIDTH * state.xp_progress.clamp(0.0, 1.0);
        if filled_width > 0.0 {
            ctx.draw_quad(
                Rect::new(start_x, bar_y, filled_width, XP_BAR_HEIGHT),
                XP_BAR_COLOR,
            );
        }

        // Level number centered above the bar
        let level_text = state.xp_level.to_string();
        ctx.draw_text(0.5, bar_y - 0.02, &level_text, XP_BAR_COLOR, 0.8);
    }

    // -- Health (hearts) --------------------------------------------------

    fn render_health(&self, state: &HudState, ctx: &mut UiContext) {
        let row_y = HOTBAR_Y - 0.04;
        let start_x = 0.5 - HOTBAR_WIDTH / 2.0;

        for i in 0..10 {
            let x = start_x + (i as f32) * ICON_SPACING;
            let threshold = ((i + 1) * 2) as f32;

            let color = if state.health >= threshold {
                HEART_COLOR
            } else if state.health >= threshold - 1.0 {
                HEART_HALF_COLOR
            } else {
                HEART_EMPTY_COLOR
            };

            ctx.draw_quad(Rect::new(x, row_y, ICON_SIZE, ICON_SIZE), color);
        }
    }

    // -- Hunger (drumsticks) ----------------------------------------------

    fn render_hunger(&self, state: &HudState, ctx: &mut UiContext) {
        let row_y = HOTBAR_Y - 0.04;
        // Right-aligned: rightmost icon at hotbar right edge.
        let end_x = 0.5 + HOTBAR_WIDTH / 2.0;

        for i in 0..10 {
            let x = end_x - ((i + 1) as f32) * ICON_SPACING;
            // Each icon represents 2 hunger points.
            let threshold = ((i + 1) * 2) as u32;

            let color = if state.hunger >= threshold {
                HUNGER_COLOR
            } else {
                HUNGER_EMPTY_COLOR
            };

            ctx.draw_quad(Rect::new(x, row_y, ICON_SIZE, ICON_SIZE), color);
        }
    }

    // -- Armor ------------------------------------------------------------

    fn render_armor(&self, state: &HudState, ctx: &mut UiContext) {
        let row_y = HOTBAR_Y - 0.065;
        let start_x = 0.5 - HOTBAR_WIDTH / 2.0;

        for i in 0..10 {
            let x = start_x + (i as f32) * ICON_SPACING;
            let threshold = ((i + 1) * 2) as u32;

            if state.armor >= threshold {
                ctx.draw_quad(Rect::new(x, row_y, ICON_SIZE, ICON_SIZE), ARMOR_COLOR);
            }
        }
    }

    // -- Debug overlay (F3) -----------------------------------------------

    fn render_debug(&self, state: &HudState, ctx: &mut UiContext) {
        let scale = 0.6;
        let x = 0.01;
        let mut y = 0.02;
        let line_height = 0.025;

        ctx.draw_text(
            x,
            y,
            &format!("FPS: {}", state.fps),
            DEBUG_TEXT_COLOR,
            scale,
        );
        y += line_height;

        let (px, py, pz) = state.player_pos;
        ctx.draw_text(
            x,
            y,
            &format!("XYZ: {px:.1} / {py:.1} / {pz:.1}"),
            DEBUG_TEXT_COLOR,
            scale,
        );
        y += line_height;

        let chunk_x = (px as i32) >> 4;
        let chunk_z = (pz as i32) >> 4;
        ctx.draw_text(
            x,
            y,
            &format!("Chunk: {chunk_x} {chunk_z}"),
            DEBUG_TEXT_COLOR,
            scale,
        );
    }
}

impl Default for HudRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> UiContext {
        UiContext::new(1920.0, 1080.0)
    }

    fn default_state() -> HudState {
        HudState::default()
    }

    /// Count commands of each variant.
    fn count_variants(cmds: &[DrawCommand]) -> (usize, usize, usize) {
        let mut quads = 0;
        let mut textured = 0;
        let mut texts = 0;
        for c in cmds {
            match c {
                DrawCommand::Quad { .. } => quads += 1,
                DrawCommand::TexturedQuad { .. } => textured += 1,
                DrawCommand::Text { .. } => texts += 1,
            }
        }
        (quads, textured, texts)
    }

    #[test]
    fn full_health_renders_expected_quads() {
        let renderer = HudRenderer::new();
        let state = default_state(); // health = 20, full
        let mut ctx = make_ctx();

        renderer.render(&state, &mut ctx);
        let cmds = ctx.take_commands();

        // Crosshair: 2 quads
        // Hotbar: 9 quads
        // XP bar: 2 quads (bg + filled since xp_progress == 0 → only bg, filled_width = 0)
        // Health: 10 quads (all full hearts)
        // Hunger: 10 quads (all full drumsticks)
        // Armor: 0 (armor == 0)
        // Texts: 1 (xp level)
        let (quads, _textured, texts) = count_variants(&cmds);

        // 2 (crosshair) + 9 (hotbar) + 1 (xp bg, no fill since progress=0) + 10 (hearts) + 10 (hunger)
        assert_eq!(quads, 32, "expected 32 quads for full-health default state");
        assert_eq!(texts, 1, "expected 1 text (xp level)");
    }

    #[test]
    fn half_health_produces_half_hearts() {
        let renderer = HudRenderer::new();
        let mut state = default_state();
        state.health = 11.0; // 5 full hearts + 1 half heart + 4 empty

        let mut ctx = make_ctx();
        renderer.render(&state, &mut ctx);
        let cmds = ctx.take_commands();

        // Filter by heart colors to avoid fragile index offsets.
        let full_count = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad { color, .. } if *color == HEART_COLOR))
            .count();
        let half_count = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad { color, .. } if *color == HEART_HALF_COLOR))
            .count();
        let empty_count = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad { color, .. } if *color == HEART_EMPTY_COLOR))
            .count();

        assert_eq!(full_count, 5, "expected 5 full hearts");
        assert_eq!(half_count, 1, "expected 1 half heart");
        assert_eq!(empty_count, 4, "expected 4 empty hearts");
    }

    #[test]
    fn debug_screen_adds_text_commands() {
        let renderer = HudRenderer::new();
        let mut state = default_state();
        state.show_debug = true;
        state.fps = 60;
        state.player_pos = (100.5, 64.0, -200.3);

        let mut ctx = make_ctx();
        renderer.render(&state, &mut ctx);
        let cmds = ctx.take_commands();

        let (_, _, texts) = count_variants(&cmds);
        // 1 (xp level) + 3 (FPS, XYZ, Chunk)
        assert_eq!(texts, 4, "expected 4 text commands with debug on");

        // Verify debug text content
        let text_cmds: Vec<_> = cmds
            .iter()
            .filter_map(|c| match c {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();

        assert!(text_cmds.iter().any(|t| t.contains("FPS: 60")));
        assert!(text_cmds.iter().any(|t| t.contains("XYZ:")));
        assert!(text_cmds.iter().any(|t| t.contains("Chunk:")));
    }

    #[test]
    fn armor_hidden_when_zero() {
        let renderer = HudRenderer::new();
        let state = default_state(); // armor == 0
        let mut ctx = make_ctx();

        renderer.render(&state, &mut ctx);
        let cmds = ctx.take_commands();

        let armor_quads = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad { color, .. } if *color == ARMOR_COLOR))
            .count();

        assert_eq!(armor_quads, 0, "no armor quads when armor == 0");
    }

    #[test]
    fn armor_shown_when_nonzero() {
        let renderer = HudRenderer::new();
        let mut state = default_state();
        state.armor = 10; // 5 full armor icons

        let mut ctx = make_ctx();
        renderer.render(&state, &mut ctx);
        let cmds = ctx.take_commands();

        let armor_quads = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad { color, .. } if *color == ARMOR_COLOR))
            .count();

        assert_eq!(armor_quads, 5, "5 armor icons for armor value 10");
    }

    #[test]
    fn xp_bar_fills_proportionally() {
        let renderer = HudRenderer::new();
        let mut state = default_state();
        state.xp_progress = 0.5;
        state.xp_level = 7;

        let mut ctx = make_ctx();
        renderer.render(&state, &mut ctx);
        let cmds = ctx.take_commands();

        // XP bar should have a fill quad.
        let xp_fill = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad { color, .. } if *color == XP_BAR_COLOR))
            .count();
        assert_eq!(xp_fill, 1, "xp bar fill should appear when progress > 0");

        // Level text should show "7".
        let level_text = cmds.iter().find_map(|c| match c {
            DrawCommand::Text { text, color, .. } if *color == XP_BAR_COLOR => Some(text.as_str()),
            _ => None,
        });
        assert_eq!(level_text, Some("7"));
    }

    #[test]
    fn selected_slot_is_highlighted() {
        let renderer = HudRenderer::new();
        let mut state = default_state();
        state.selected_slot = 4;

        let mut ctx = make_ctx();
        renderer.render(&state, &mut ctx);
        let cmds = ctx.take_commands();

        let selected_count = cmds
            .iter()
            .filter(
                |c| matches!(c, DrawCommand::Quad { color, .. } if *color == SLOT_SELECTED_COLOR),
            )
            .count();
        assert_eq!(selected_count, 1, "exactly one slot should be highlighted");

        let normal_slot_count = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad { color, .. } if *color == SLOT_BG_COLOR))
            .count();
        assert_eq!(
            normal_slot_count, 8,
            "8 normal slots + 1 selected = 9 total"
        );
    }
}
