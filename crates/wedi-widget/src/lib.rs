//! wedi-widget - 可嵌入的編輯器 Widget
//!
//! 提供可在其他 TUI 應用程式中嵌入的編輯器元件。
//!
//! # 主要類型
//!
//! - [`EditorConfig`] - 編輯器配置選項
//! - [`ScreenLayout`] - 螢幕佈局管理
//! - [`LineLayout`] - 行佈局處理
//!
//! # 使用範例
//!
//! ```rust
//! use wedi_widget::{EditorConfig, ScreenLayout};
//!
//! let config = EditorConfig::new()
//!     .with_line_numbers(true)
//!     .with_tab_width(4);
//!
//! let layout = ScreenLayout::new(24, 80);
//! ```

pub mod config;
pub mod layout;

// 重新匯出 wedi-core 的主要類型供便利使用
pub use wedi_core::{
    buffer::RopeBuffer,
    cursor::Cursor,
    keymap::{Command, Direction, Keymap},
    search::Search,
    view::{Selection, View},
    Terminal,
};

// 重新匯出本地類型
pub use config::EditorConfig;
pub use layout::{LineLayout, ScreenLayout};
