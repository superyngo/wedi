/// 編輯器配置選項
///
/// 用於自訂編輯器的顯示和行為設定。
///
/// # 範例
///
/// ```rust
/// use wedi_widget::EditorConfig;
///
/// let config = EditorConfig::new()
///     .with_line_numbers(true)
///     .with_wrap_mode(true)
///     .with_tab_width(4)
///     .with_theme("base16-ocean.dark".to_string());
/// ```
#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// 是否顯示行號
    pub show_line_numbers: bool,
    /// 是否啟用自動換行（true=多行換行, false=單行水平滾動）
    pub wrap_mode: bool,
    /// Tab 鍵寬度（空格數）
    pub tab_width: usize,
    /// 語法高亮主題名稱
    pub theme: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            wrap_mode: true,
            tab_width: 4,
            theme: "base16-ocean.dark".into(),
        }
    }
}

impl EditorConfig {
    /// 建立預設配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 設定是否顯示行號
    ///
    /// # Arguments
    /// * `show` - true 為顯示，false 為隱藏
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// 設定是否啟用自動換行
    ///
    /// # Arguments
    /// * `wrap` - true 為多行換行模式，false 為單行水平滾動模式
    pub fn with_wrap_mode(mut self, wrap: bool) -> Self {
        self.wrap_mode = wrap;
        self
    }

    /// 設定 Tab 鍵寬度
    ///
    /// # Arguments
    /// * `width` - Tab 鍵對應的空格數
    pub fn with_tab_width(mut self, width: usize) -> Self {
        self.tab_width = width;
        self
    }

    /// 設定語法高亮主題
    ///
    /// # Arguments
    /// * `theme` - 主題名稱（例如 "base16-ocean.dark"）
    pub fn with_theme(mut self, theme: String) -> Self {
        self.theme = theme;
        self
    }
}
