//! wedi - 輕量級跨平台終端文字編輯器

// 導出公開模組
#[cfg(feature = "syntax-highlighting")]
pub mod highlight;

// 內部模組（供 lib 編譯）
mod buffer;
mod clipboard;
mod comment;
mod config;
mod cursor;
mod dialog;
mod input;
mod search;
mod terminal;
mod utils;
mod view;

// 重新導出常用類型（供 examples 使用）
pub use buffer::RopeBuffer;
pub use cursor::Cursor;
