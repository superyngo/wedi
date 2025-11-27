// 語法高亮主模組

// 語法高亮功能（可選）
#[cfg(feature = "syntax-highlighting")]
mod cache;
#[cfg(feature = "syntax-highlighting")]
mod engine;

// 導出公開 API
#[cfg(feature = "syntax-highlighting")]
pub use cache::{CachedLine, EditType, HighlightCache};
#[cfg(feature = "syntax-highlighting")]
pub use engine::{supports_true_color, HighlightEngine};

/// 語法高亮設定
#[cfg(feature = "syntax-highlighting")]
#[derive(Clone, Debug)]
pub struct HighlightConfig {
    /// 是否啟用語法高亮
    pub enabled: bool,
    /// 主題名稱
    pub theme: String,
    /// 是否使用真彩色
    pub true_color: bool,
}

#[cfg(feature = "syntax-highlighting")]
impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "base16-ocean.dark".to_string(),
            true_color: supports_true_color(),
        }
    }
}
