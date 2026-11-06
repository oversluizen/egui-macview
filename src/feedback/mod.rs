//! State feedback surfaces: Context Menus and Animated Toast Notifications.

pub mod context_menu;
pub mod toast;

pub use context_menu::{clamp_menu_rect, MacContextMenu};
pub use toast::{MacToast, MacToastManager, ToastLevel, DEFAULT_TOAST_DURATION};
