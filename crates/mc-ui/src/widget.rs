/// Screen-space rectangle in normalized coordinates (0,0 = top-left, 1,1 = bottom-right).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// RGBA color with each channel in 0.0..=1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
}

/// A draw command emitted by UI widgets for the renderer to execute.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    /// Solid colored rectangle.
    Quad { rect: Rect, color: Color },
    /// Textured rectangle with an atlas reference.
    TexturedQuad {
        rect: Rect,
        tex_index: u16,
        color: Color,
    },
    /// Text rendering placeholder.
    Text {
        position: (f32, f32),
        text: String,
        color: Color,
        scale: f32,
    },
}

/// Accumulates draw commands from UI widgets for a single frame.
pub struct UiContext {
    pub screen_width: f32,
    pub screen_height: f32,
    commands: Vec<DrawCommand>,
}

impl UiContext {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            commands: Vec::new(),
        }
    }

    pub fn draw_quad(&mut self, rect: Rect, color: Color) {
        self.commands.push(DrawCommand::Quad { rect, color });
    }

    pub fn draw_textured_quad(&mut self, rect: Rect, tex_index: u16, color: Color) {
        self.commands.push(DrawCommand::TexturedQuad {
            rect,
            tex_index,
            color,
        });
    }

    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, color: Color, scale: f32) {
        self.commands.push(DrawCommand::Text {
            position: (x, y),
            text: text.to_string(),
            color,
            scale,
        });
    }

    /// Drains and returns all accumulated draw commands.
    pub fn take_commands(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Returns a read-only view of the accumulated commands.
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_stores_coordinates() {
        let r = Rect::new(0.1, 0.2, 0.5, 0.3);
        assert_eq!(r.x, 0.1);
        assert_eq!(r.y, 0.2);
        assert_eq!(r.width, 0.5);
        assert_eq!(r.height, 0.3);
    }

    #[test]
    fn color_constants_are_correct() {
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn draw_quad_adds_command() {
        let mut ctx = UiContext::new(1920.0, 1080.0);
        ctx.draw_quad(Rect::new(0.0, 0.0, 0.5, 0.5), Color::RED);

        let cmds = ctx.take_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(&cmds[0], DrawCommand::Quad { .. }));
    }

    #[test]
    fn draw_textured_quad_adds_command() {
        let mut ctx = UiContext::new(800.0, 600.0);
        ctx.draw_textured_quad(Rect::new(0.0, 0.0, 0.1, 0.1), 42, Color::WHITE);

        let cmds = ctx.take_commands();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            DrawCommand::TexturedQuad { tex_index, .. } => assert_eq!(*tex_index, 42),
            other => panic!("expected TexturedQuad, got {other:?}"),
        }
    }

    #[test]
    fn draw_text_adds_command() {
        let mut ctx = UiContext::new(800.0, 600.0);
        ctx.draw_text(0.1, 0.2, "Hello", Color::WHITE, 1.0);

        let cmds = ctx.take_commands();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            DrawCommand::Text { text, position, .. } => {
                assert_eq!(text, "Hello");
                assert_eq!(*position, (0.1, 0.2));
            }
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn take_commands_drains_buffer() {
        let mut ctx = UiContext::new(800.0, 600.0);
        ctx.draw_quad(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        ctx.draw_quad(Rect::new(0.0, 0.0, 0.5, 0.5), Color::RED);

        let first = ctx.take_commands();
        assert_eq!(first.len(), 2);

        let second = ctx.take_commands();
        assert!(second.is_empty());
    }

    #[test]
    fn commands_returns_read_only_view() {
        let mut ctx = UiContext::new(800.0, 600.0);
        ctx.draw_quad(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);

        assert_eq!(ctx.commands().len(), 1);
        // Commands are still there after reading.
        assert_eq!(ctx.commands().len(), 1);
    }
}
