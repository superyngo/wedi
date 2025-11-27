//! 語法高亮快取系統（簡化版本）
//!
//! 由於 syntect 的 ParseState 是私有的，我們只快取已高亮的字串

use std::collections::HashMap;

/// 單行的高亮快取項目
///
/// ⚠️ 注意：不包含 ParseState，因為 syntect 的 ParseState 是私有的
/// 快取失效策略：修改任何一行時，使該行及之後所有行失效
#[derive(Clone, Debug)]
pub struct CachedLine {
    /// 原始文字內容（用於驗證快取是否有效）
    pub text: String,
    /// 高亮後的 ANSI 字串
    pub highlighted: String,
}

/// 語法狀態快取（用於優化效能）
pub struct HighlightCache {
    /// 快取的行（行號 -> 快取項目）
    lines: HashMap<usize, CachedLine>,
    /// 快取大小限制
    max_size: usize,
}

impl HighlightCache {
    /// 建立新的快取（預設快取 1000 行）
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// 建立指定容量的快取
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            lines: HashMap::with_capacity(max_size.min(1000)),
            max_size,
        }
    }

    /// 取得快取的行
    pub fn get(&self, line_idx: usize) -> Option<&CachedLine> {
        self.lines.get(&line_idx)
    }

    /// 檢查行是否已快取且內容相同
    pub fn is_valid(&self, line_idx: usize, text: &str) -> bool {
        self.lines
            .get(&line_idx)
            .map(|cached| cached.text == text)
            .unwrap_or(false)
    }

    /// 插入快取項目
    pub fn insert(&mut self, line_idx: usize, cached: CachedLine) {
        // 如果超過容量，清除舊的快取
        if self.lines.len() >= self.max_size {
            // 簡單策略：清除所有快取（更複雜的可以用 LRU）
            self.lines.clear();
        }

        self.lines.insert(line_idx, cached);
    }

    /// 使指定行失效
    #[allow(dead_code)]
    pub fn invalidate(&mut self, line_idx: usize) {
        self.lines.remove(&line_idx);
    }

    /// 使範圍內的行失效（包含 start 和 end）
    #[allow(dead_code)]
    pub fn invalidate_range(&mut self, start: usize, end: usize) {
        for idx in start..=end {
            self.lines.remove(&idx);
        }
    }

    /// 使從指定行開始的所有行失效
    ///
    /// ⚠️ 這是因為語法狀態可能影響後續所有行（如多行註解）
    pub fn invalidate_from(&mut self, line_idx: usize) {
        self.lines.retain(|&idx, _| idx < line_idx);
    }

    /// 智慧失效：根據編輯操作類型決定失效範圍
    pub fn invalidate_from_edit(&mut self, line_idx: usize, edit_type: EditType) {
        match edit_type {
            EditType::CharInsert | EditType::CharDelete => {
                // 字元級編輯：使當前行及之後所有行失效
                // （因為可能影響語法狀態，例如開始/結束多行註解）
                self.invalidate_from(line_idx);
            }
            EditType::LineInsert | EditType::LineDelete | EditType::MultiLineEdit => {
                // 行級編輯：清除所有快取（行號改變）
                self.clear();
            }
        }
    }

    /// 清除所有快取
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// 取得快取統計資訊
    #[allow(dead_code)]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            cached_lines: self.lines.len(),
            capacity: self.max_size,
        }
    }

    /// 取得快取的行數
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// 快取是否為空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for HighlightCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 快取統計資訊
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub cached_lines: usize,
    pub capacity: usize,
}

/// 編輯操作類型（用於智慧快取失效）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditType {
    /// 插入單個字元
    CharInsert,
    /// 刪除單個字元
    #[allow(dead_code)]
    CharDelete,
    /// 插入新行
    #[allow(dead_code)]
    LineInsert,
    /// 刪除整行
    #[allow(dead_code)]
    LineDelete,
    /// 多行編輯（複製/貼上等）
    #[allow(dead_code)]
    MultiLineEdit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: "\x1b[0mtest\x1b[0m".to_string(),
        };

        cache.insert(0, cached.clone());
        assert!(cache.is_valid(0, "test"));
        assert!(!cache.is_valid(0, "different"));
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: String::new(),
        };

        cache.insert(0, cached.clone());
        cache.insert(1, cached.clone());
        cache.insert(2, cached);

        assert_eq!(cache.len(), 3);

        // 使第 1 行及之後所有行失效
        cache.invalidate_from(1);

        assert_eq!(cache.len(), 1);
        assert!(cache.get(0).is_some());
        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_none());
    }

    #[test]
    fn test_smart_invalidation() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: String::new(),
        };

        // 建立 10 行快取
        for i in 0..10 {
            cache.insert(i, cached.clone());
        }

        assert_eq!(cache.len(), 10);

        // 字元編輯：使第 5 行及之後失效
        cache.invalidate_from_edit(5, EditType::CharInsert);

        assert_eq!(cache.len(), 5);
        assert!(cache.get(4).is_some());
        assert!(cache.get(5).is_none());
    }

    #[test]
    fn test_line_edit_clears_all() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: String::new(),
        };

        for i in 0..10 {
            cache.insert(i, cached.clone());
        }

        // 插入行：清除所有快取
        cache.invalidate_from_edit(5, EditType::LineInsert);

        assert_eq!(cache.len(), 0);
    }
}
