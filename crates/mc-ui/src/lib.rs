//! HUD data model and widget rendering system.
//!
//! Provides [`HudState`] for health/hunger/XP bar data and [`HudRenderer`] for display,
//! plus a lightweight [`UiContext`] with [`DrawCommand`]-based widget primitives.

pub mod debug_screen;
pub mod hotbar;
pub mod hud;
pub mod inventory_screen;
pub mod widget;

pub use debug_screen::DebugInfo;
pub use hotbar::{HotbarData, HotbarRenderInfo, HotbarSlot};
pub use hud::{HudRenderer, HudState};
pub use widget::{Color, DrawCommand, Rect, UiContext};
