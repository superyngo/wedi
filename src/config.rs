// 配置管理
// 這個模組將在後續階段實現

pub struct Config {
    pub tab_width: usize,
    pub line_numbers: bool,
    pub auto_indent: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            tab_width: 4,
            line_numbers: true,
            auto_indent: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
