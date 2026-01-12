//! wedi-widget - 可嵌入的編輯器 Widget
//!
//! 提供可在其他 TUI 應用程式中嵌入的編輯器元件。

pub mod config;

// 重新匯出 wedi-core 的主要類型供便利使用
pub use wedi_core::{
    buffer::RopeBuffer,
    cursor::Cursor,
    keymap::{Command, Direction, Keymap},
    view::{Selection, View},
    Terminal,
};

pub use config::EditorConfig;
