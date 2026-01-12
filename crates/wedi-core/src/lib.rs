//! wedi-core - 核心編輯器元件
//!
//! 提供文字緩衝區、游標管理、快捷鍵對映和語法高亮等基礎功能。

pub mod buffer;
pub mod clipboard;
pub mod comment;
pub mod cursor;
pub mod keymap;
pub mod search;
pub mod terminal;
pub mod utils;
pub mod view;

#[cfg(feature = "syntax-highlighting")]
pub mod highlight;

// 重新匯出常用類型
pub use buffer::RopeBuffer;
pub use cursor::Cursor;
pub use keymap::{Command, Direction, Keymap};
pub use search::Search;
pub use terminal::Terminal;
pub use view::{Selection, View};
