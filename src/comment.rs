// 註解處理
// 這個模組將在後續階段實現

use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum CommentStyle {
    Line(String), // 單行註解，如 "//"
    #[allow(dead_code)]
    Block(String, String), // 塊註解，如 "/*" 和 "*/"
}

pub struct CommentHandler {
    style: Option<CommentStyle>,
}

impl CommentHandler {
    pub fn new() -> Self {
        Self { style: None }
    }

    pub fn detect_from_path(&mut self, path: &Path) {
        let extension = path.extension().and_then(|s| s.to_str());

        self.style = match extension {
            // C-style comments: //
            Some("rs") | Some("c") | Some("cpp") | Some("cc") | Some("cxx") | Some("h")
            | Some("hpp") | Some("java") | Some("js") | Some("ts") | Some("jsx") | Some("tsx")
            | Some("go") | Some("cs") | Some("php") | Some("swift") | Some("kt") => {
                Some(CommentStyle::Line("//".to_string()))
            }
            // Hash/Pound comments: #
            Some("py") | Some("sh") | Some("bash") | Some("rb") | Some("pl") | Some("yaml")
            | Some("yml") | Some("toml") | Some("ps1") | Some("r") => {
                Some(CommentStyle::Line("#".to_string()))
            }
            // SQL-style comments: --
            Some("sql") | Some("lua") | Some("hs") | Some("elm") => {
                Some(CommentStyle::Line("--".to_string()))
            }
            // Batch/CMD comments: REM
            Some("bat") | Some("cmd") => Some(CommentStyle::Line("REM".to_string())),
            // Vim comments: "
            Some("vim") | Some("vimrc") => Some(CommentStyle::Line("\"".to_string())),
            // 默認使用 # 註解（適用於大多數腳本語言和配置文件）
            _ => Some(CommentStyle::Line("#".to_string())),
        };
    }

    pub fn toggle_line_comment(&self, line: &str) -> Option<String> {
        match &self.style {
            Some(CommentStyle::Line(prefix)) => {
                let trimmed = line.trim_start();

                // 檢查是否已有註解（註解符號後面可能有或沒有空格）
                if trimmed.starts_with(prefix) {
                    // 取消註解：移除 "prefix " 或 "prefix"
                    let after_prefix = trimmed.strip_prefix(prefix)?;
                    let uncommented = if after_prefix.starts_with(' ') {
                        after_prefix.strip_prefix(' ').unwrap_or(after_prefix)
                    } else {
                        after_prefix
                    };

                    // 如果取消註解後是空字串，不保留前導空格
                    if uncommented.is_empty() {
                        Some(String::new())
                    } else {
                        let leading_spaces = line.len() - line.trim_start().len();
                        Some(format!("{}{}", " ".repeat(leading_spaces), uncommented))
                    }
                } else {
                    // 添加註解：一律使用 "prefix "
                    // 如果是空行（trimmed 為空），不保留前導空格
                    if trimmed.is_empty() {
                        Some(format!("{} ", prefix))
                    } else {
                        let leading_spaces = line.len() - line.trim_start().len();
                        Some(format!(
                            "{}{} {}",
                            " ".repeat(leading_spaces),
                            prefix,
                            trimmed
                        ))
                    }
                }
            }
            _ => None,
        }
    }

    /// 檢查一行是否已經有註解
    pub fn is_commented(&self, line: &str) -> bool {
        match &self.style {
            Some(CommentStyle::Line(prefix)) => {
                let trimmed = line.trim_start();
                trimmed.starts_with(prefix)
            }
            _ => false,
        }
    }

    /// 添加註解到一行 - 一律使用 "prefix "
    pub fn add_comment(&self, line: &str) -> Option<String> {
        match &self.style {
            Some(CommentStyle::Line(prefix)) => {
                let trimmed = line.trim_start();

                // 如果是空行，不保留前導空格
                if trimmed.is_empty() {
                    Some(format!("{} ", prefix))
                } else {
                    let leading_spaces = line.len() - line.trim_start().len();
                    Some(format!(
                        "{}{} {}",
                        " ".repeat(leading_spaces),
                        prefix,
                        trimmed
                    ))
                }
            }
            _ => None,
        }
    }

    /// 移除註解從一行 - 移除 "prefix " 或 "prefix"
    pub fn remove_comment(&self, line: &str) -> Option<String> {
        match &self.style {
            Some(CommentStyle::Line(prefix)) => {
                let trimmed = line.trim_start();

                if trimmed.starts_with(prefix) {
                    let after_prefix = trimmed.strip_prefix(prefix)?;
                    let uncommented = if after_prefix.starts_with(' ') {
                        after_prefix.strip_prefix(' ').unwrap_or(after_prefix)
                    } else {
                        after_prefix
                    };

                    // 如果取消註解後是空字串，不保留前導空格
                    if uncommented.is_empty() {
                        Some(String::new())
                    } else {
                        let leading_spaces = line.len() - line.trim_start().len();
                        Some(format!("{}{}", " ".repeat(leading_spaces), uncommented))
                    }
                } else {
                    Some(line.to_string())
                }
            }
            _ => None,
        }
    }

    pub fn has_comment_style(&self) -> bool {
        self.style.is_some()
    }

    /// 查找行中註解符號的起始位置（如果有的話）
    /// 返回 Some(index) 表示從該位置開始是註解
    pub fn find_comment_start(&self, line: &str) -> Option<usize> {
        match &self.style {
            Some(CommentStyle::Line(prefix)) => line.find(prefix),
            _ => None,
        }
    }
}

impl Default for CommentHandler {
    fn default() -> Self {
        Self::new()
    }
}
