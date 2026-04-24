//! HUD data model and widget rendering system.

pub mod achievement_tracker;
pub mod armor_trim_render;
pub mod credits;
pub mod damage_indicator;
pub mod debug_screen;
pub mod hotbar;
pub mod hud;
pub mod hud_visibility;
pub mod inventory_screen;
pub mod key_bindings;
pub mod loading_screen;
pub mod scoreboard_display;
pub mod settings_menu;
pub mod subtitles;
pub mod tab_list;
pub mod title_screen;
pub mod toast;
pub mod widget;

pub use debug_screen::DebugInfo;
pub use hotbar::{HotbarData, HotbarRenderInfo, HotbarSlot};
pub use hud::{HudRenderer, HudState};
pub use widget::{Color, DrawCommand, Rect, UiContext};
