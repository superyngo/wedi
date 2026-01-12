/// 編輯器配置
#[derive(Debug, Clone)]
pub struct EditorConfig {
    pub show_line_numbers: bool,
    pub wrap_mode: bool,
    pub tab_width: usize,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn with_wrap_mode(mut self, wrap: bool) -> Self {
        self.wrap_mode = wrap;
        self
    }

    pub fn with_tab_width(mut self, width: usize) -> Self {
        self.tab_width = width;
        self
    }

    pub fn with_theme(mut self, theme: String) -> Self {
        self.theme = theme;
        self
    }
}
