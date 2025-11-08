use anyhow::{Context, Result};
use ropey::{Rope, RopeSlice};
use std::fs;
use std::path::{Path, PathBuf};

use super::history::{Action, History};

pub struct RopeBuffer {
    rope: Rope,
    file_path: Option<PathBuf>,
    modified: bool,
    history: History,
    in_undo_redo: bool, // 防止在撤銷/重做時記錄歷史
}

impl RopeBuffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            modified: false,
            history: History::default(),
            in_undo_redo: false,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        // 如果文件存在，讀取內容；否則創建空緩衝區
        let (rope, modified) = if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            (Rope::from_str(&content), false)
        } else {
            // 文件不存在，創建空緩衝區（將在保存時創建文件）
            (Rope::new(), true)
        };

        Ok(Self {
            rope,
            file_path: Some(path.to_path_buf()),
            modified,
            history: History::default(),
            in_undo_redo: false,
        })
    }

    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let pos = pos.min(self.rope.len_chars());

        // 記錄到歷史
        if !self.in_undo_redo {
            self.history.push(Action::Insert {
                pos,
                text: ch.to_string(),
            });
        }

        self.rope.insert_char(pos, ch);
        self.modified = true;
    }

    pub fn insert(&mut self, pos: usize, text: &str) {
        let pos = pos.min(self.rope.len_chars());

        // 記錄到歷史
        if !self.in_undo_redo {
            self.history.push(Action::Insert {
                pos,
                text: text.to_string(),
            });
        }

        self.rope.insert(pos, text);
        self.modified = true;
    }

    pub fn delete_char(&mut self, pos: usize) {
        if pos < self.rope.len_chars() {
            // 獲取要刪除的字符
            let deleted_char = self.rope.char(pos).to_string();

            // 記錄到歷史
            if !self.in_undo_redo {
                self.history.push(Action::Delete {
                    pos,
                    text: deleted_char,
                });
            }

            self.rope.remove(pos..pos + 1);
            self.modified = true;
        }
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        if start < end && start < self.rope.len_chars() {
            let end = end.min(self.rope.len_chars());

            // 獲取要刪除的文本
            let deleted_text = self.rope.slice(start..end).to_string();

            // 記錄到歷史
            if !self.in_undo_redo {
                self.history.push(Action::DeleteRange {
                    start,
                    end,
                    text: deleted_text,
                });
            }

            self.rope.remove(start..end);
            self.modified = true;
        }
    }

    pub fn delete_line(&mut self, row: usize) {
        if row < self.line_count() {
            let start = self.rope.line_to_char(row);
            let end = if row + 1 < self.line_count() {
                self.rope.line_to_char(row + 1)
            } else {
                self.rope.len_chars()
            };

            // 獲取要刪除的行
            let deleted_line = self.rope.slice(start..end).to_string();

            // 記錄到歷史
            if !self.in_undo_redo {
                self.history.push(Action::DeleteRange {
                    start,
                    end,
                    text: deleted_line,
                });
            }

            self.rope.remove(start..end);
            self.modified = true;
        }
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, idx: usize) -> Option<RopeSlice<'_>> {
        if idx < self.line_count() {
            Some(self.rope.line(idx))
        } else {
            None
        }
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx.min(self.line_count()))
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx.min(self.rope.len_chars()))
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.file_path.clone() {
            let contents = self.rope.to_string();
            std::fs::write(path, contents)?;
            self.modified = false;
            Ok(())
        } else {
            anyhow::bail!("No file path set")
        }
    }

    #[allow(dead_code)]
    pub fn save_to(&mut self, path: &Path) -> Result<()> {
        let contents = self.rope.to_string();
        std::fs::write(path, contents)?;
        self.modified = false;
        self.file_path = Some(path.to_path_buf());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_as(&mut self, path: &Path) -> Result<()> {
        fs::write(path, self.rope.to_string())
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
        self.file_path = Some(path.to_path_buf());
        self.modified = false;
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    #[allow(dead_code)]
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]")
            .to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn get_line_content(&self, line_idx: usize) -> String {
        if let Some(line) = self.line(line_idx) {
            line.to_string()
        } else {
            String::new()
        }
    }

    /// 獲取完整行內容（包括尾部空格和換行符）
    pub fn get_line_full(&self, line_idx: usize) -> String {
        let line_start = self.line_to_char(line_idx);
        let line_end = if line_idx + 1 < self.line_count() {
            self.line_to_char(line_idx + 1)
        } else {
            self.rope.len_chars()
        };
        self.rope.slice(line_start..line_end).to_string()
    }

    // 撤銷/重做方法
    pub fn undo(&mut self) -> Option<usize> {
        if let Some(action) = self.history.undo() {
            self.in_undo_redo = true;

            let result_pos = match action {
                Action::Insert { pos, text } => {
                    // 撤銷插入 = 刪除
                    self.rope.remove(pos..pos + text.len());
                    self.modified = true;
                    Some(pos)
                }
                Action::Delete { pos, text } => {
                    // 撤銷刪除 = 插入
                    self.rope.insert(pos, &text);
                    self.modified = true;
                    Some(pos)
                }
                Action::DeleteRange { start, text, .. } => {
                    // 撤銷範圍刪除 = 插入
                    self.rope.insert(start, &text);
                    self.modified = true;
                    Some(start)
                }
            };

            self.in_undo_redo = false;
            result_pos
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<usize> {
        if let Some(action) = self.history.redo() {
            self.in_undo_redo = true;

            let result_pos = match action {
                Action::Insert { pos, text } => {
                    // 重做插入
                    self.rope.insert(pos, &text);
                    self.modified = true;
                    Some(pos + text.len())
                }
                Action::Delete { pos, text } => {
                    // 重做刪除
                    self.rope.remove(pos..pos + text.len());
                    self.modified = true;
                    Some(pos)
                }
                Action::DeleteRange { start, end, .. } => {
                    // 重做範圍刪除
                    self.rope.remove(start..end);
                    self.modified = true;
                    Some(start)
                }
            };

            self.in_undo_redo = false;
            result_pos
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    #[allow(dead_code)]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }
}

impl Default for RopeBuffer {
    fn default() -> Self {
        Self::new()
    }
}
