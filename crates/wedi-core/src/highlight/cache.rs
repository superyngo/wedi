//! 語法高亮快取
//!
//! 每 [`CHECKPOINT_INTERVAL`] 行保存一次 syntect 解析狀態（檢查點），
//! 渲染時從可見區上方最近的有效檢查點開始解析，跨行語法（多行註解、字串）
//! 因此永遠正確，且不必每幀從檔案開頭重新解析。

use std::collections::HashMap;

use super::engine::{HighlightEngine, LineState};

/// Lines between two saved parser states.
pub const CHECKPOINT_INTERVAL: usize = 64;

/// 已高亮行的快取上限；超過時只保留可見區附近的行
const MAX_CACHED_LINES: usize = 2000;

/// One highlighted line and the text it was produced from.
#[derive(Clone, Debug)]
pub struct CachedLine {
    /// 原始文字內容（用於驗證快取是否有效）
    pub text: String,
    /// 高亮後的 ANSI 字串
    pub highlighted: String,
}

/// Highlighted lines plus parser checkpoints for one buffer.
pub struct HighlightCache {
    /// 已高亮的行（行號 -> 快取項目）
    lines: HashMap<usize, CachedLine>,
    /// checkpoints[k] = 第 k * CHECKPOINT_INTERVAL 行開始前的解析狀態
    checkpoints: Vec<LineState>,
    /// 累計解析行數（測試用來確認不會從頭解析）
    parsed_lines: usize,
}

impl HighlightCache {
    pub fn new() -> Self {
        Self {
            lines: HashMap::new(),
            checkpoints: Vec::new(),
            parsed_lines: 0,
        }
    }

    /// Forget everything that may depend on `row` or later lines.
    ///
    /// Call with the first row the buffer changed.
    pub fn invalidate_from(&mut self, row: usize) {
        self.lines.retain(|&idx, _| idx < row);
        // 檢查點 k 只依賴 k * INTERVAL 之前的行，k * INTERVAL <= row 者仍有效
        self.checkpoints.truncate(row / CHECKPOINT_INTERVAL + 1);
    }

    /// Drop all cached lines and checkpoints.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.checkpoints.clear();
    }

    /// Total lines run through the parser so far.
    pub fn parsed_lines(&self) -> usize {
        self.parsed_lines
    }

    /// Highlight rows `start..=end`, reusing cached lines and checkpoints.
    ///
    /// `line_text` returns the text of a row; rows past `line_count` are ignored.
    pub fn highlight_rows<F>(
        &mut self,
        engine: &HighlightEngine,
        line_count: usize,
        start: usize,
        end: usize,
        line_text: F,
    ) -> HashMap<usize, String>
    where
        F: Fn(usize) -> Option<String>,
    {
        let mut result = HashMap::new();
        if line_count == 0 {
            return result;
        }
        let end = end.min(line_count - 1);
        if start > end {
            return result;
        }

        // syntect 需要換行符才能正確維護跨行狀態
        let text_of = |row: usize| {
            let mut text = line_text(row).unwrap_or_default();
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text
        };

        // 快速路徑：可見行全部命中快取
        let all_cached = (start..=end).all(|row| {
            self.lines
                .get(&row)
                .is_some_and(|cached| cached.text == text_of(row))
        });
        if all_cached {
            for row in start..=end {
                result.insert(row, self.lines[&row].highlighted.clone());
            }
            return result;
        }

        // 從可見區上方最近的有效檢查點開始解析
        let (mut highlighter, from) = if self.checkpoints.is_empty() {
            let Some(mut highlighter) = engine.create_highlighter() else {
                return result;
            };
            self.checkpoints.push(highlighter.state());
            (highlighter, 0)
        } else {
            let k = (start / CHECKPOINT_INTERVAL).min(self.checkpoints.len() - 1);
            (
                engine.resume_highlighter(&self.checkpoints[k]),
                k * CHECKPOINT_INTERVAL,
            )
        };

        for row in from..=end {
            if row % CHECKPOINT_INTERVAL == 0 && row / CHECKPOINT_INTERVAL == self.checkpoints.len()
            {
                self.checkpoints.push(highlighter.state());
            }
            let text = text_of(row);
            let highlighted = highlighter
                .highlight_line(&text)
                .trim_end_matches(['\n', '\r'])
                .to_string();
            self.parsed_lines += 1;
            if row >= start {
                result.insert(row, highlighted.clone());
                self.lines.insert(row, CachedLine { text, highlighted });
            }
        }

        // 快取過大時只保留可見區附近的行
        if self.lines.len() > MAX_CACHED_LINES {
            let keep = MAX_CACHED_LINES / 2;
            let low = start.saturating_sub(keep);
            self.lines
                .retain(|&idx, _| idx >= low && idx <= end.saturating_add(keep));
        }

        result
    }
}

impl Default for HighlightCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn c_engine() -> HighlightEngine {
        let mut engine = HighlightEngine::new(None, true).unwrap();
        engine.set_file(Some(Path::new("t.c")));
        engine
    }

    /// 第 5 行開啟區塊註解，第 806 行才關閉
    fn commented_file() -> Vec<String> {
        let mut lines = vec!["int a = 1;".to_string(); 5];
        lines.push("/* block comment opens here".to_string());
        lines.extend((0..800).map(|i| format!("int x{i} = {i};")));
        lines.push("*/".to_string());
        lines.push("int z = 0;".to_string());
        lines
    }

    fn render(
        cache: &mut HighlightCache,
        engine: &HighlightEngine,
        lines: &[String],
        start: usize,
        end: usize,
    ) -> HashMap<usize, String> {
        cache.highlight_rows(engine, lines.len(), start, end, |r| lines.get(r).cloned())
    }

    #[test]
    fn test_comment_far_above_viewport() {
        // 稽核 F17：註解開在可見區上方 150+ 行時仍要以註解上色
        let engine = c_engine();
        let lines = commented_file();
        let deep = render(&mut HighlightCache::new(), &engine, &lines, 300, 310);
        let full = render(&mut HighlightCache::new(), &engine, &lines, 0, 310);
        assert_eq!(deep[&300], full[&300]);
        // 與同樣文字在註解外（第 0 行的程式碼色）不同
        let code = render(
            &mut HighlightCache::new(),
            &engine,
            &["int x293 = 293;".to_string()],
            0,
            0,
        );
        assert_ne!(deep[&300], code[&0]);
    }

    #[test]
    fn test_invalidate_recomputes_following_lines() {
        let engine = c_engine();
        let mut lines = commented_file();
        let mut cache = HighlightCache::new();
        let before = render(&mut cache, &engine, &lines, 300, 310);
        // 移除註解開頭：後續行應恢復程式碼色
        lines[5] = "int b = 2;".to_string();
        cache.invalidate_from(5);
        let after = render(&mut cache, &engine, &lines, 300, 310);
        let fresh = render(&mut HighlightCache::new(), &engine, &lines, 300, 310);
        assert_ne!(before[&300], after[&300]);
        assert_eq!(after, fresh);
    }

    #[test]
    fn test_edit_near_viewport_does_not_reparse_from_top() {
        let engine = c_engine();
        let lines: Vec<String> = (0..5000).map(|i| format!("int x{i} = {i};")).collect();
        let mut cache = HighlightCache::new();
        render(&mut cache, &engine, &lines, 4900, 4920);
        let first = cache.parsed_lines();
        assert!(first >= 4900);
        // 未變更時完全不解析
        render(&mut cache, &engine, &lines, 4900, 4920);
        assert_eq!(cache.parsed_lines(), first);
        // 可見區內的編輯只從最近的檢查點重解析
        cache.invalidate_from(4910);
        render(&mut cache, &engine, &lines, 4900, 4920);
        assert!(cache.parsed_lines() - first <= CHECKPOINT_INTERVAL + 21);
    }

    #[test]
    fn test_invalidate_keeps_earlier_checkpoints() {
        let engine = c_engine();
        let lines: Vec<String> = (0..300).map(|i| format!("int x{i};")).collect();
        let mut cache = HighlightCache::new();
        render(&mut cache, &engine, &lines, 250, 260);
        assert_eq!(cache.checkpoints.len(), 5); // 0, 64, 128, 192, 256
        cache.invalidate_from(130);
        assert_eq!(cache.checkpoints.len(), 3); // 192、256 依賴第 130 行
        cache.invalidate_from(128);
        assert_eq!(cache.checkpoints.len(), 3); // 128 只依賴 0..128
        cache.clear();
        assert!(cache.checkpoints.is_empty());
    }
}
