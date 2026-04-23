//! HUD data model and widget rendering system.
//!
//! Provides [`HudState`] for health/hunger/XP bar data and [`HudRenderer`] for display,
//! plus a lightweight [`UiContext`] with [`DrawCommand`]-based widget primitives.

pub mod debug_screen;
pub mod hud;
pub mod widget;

pub use hud::{HudRenderer, HudState};
pub use widget::{Color, DrawCommand, Rect, UiContext};
