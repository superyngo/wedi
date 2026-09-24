// 搜索功能
// 這個模組將在後續階段實現

use crate::buffer::RopeBuffer;

pub struct Search {
    query: String,
    matches: Vec<(usize, usize)>, // (line, col) pairs
    current_match: usize,
}

impl Search {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
        }
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.matches.clear();
        self.current_match = 0;
    }

    pub fn find_matches(&mut self, buffer: &RopeBuffer) {
        self.matches.clear();

        if self.query.is_empty() {
            return;
        }

        for line_idx in 0..buffer.line_count() {
            let line_content = buffer.get_line_content(line_idx);
            let line_content = line_content.trim_end_matches(['\n', '\r']);

            let mut start = 0;
            while let Some(pos) = line_content[start..].find(&self.query) {
                let actual_pos = start + pos;
                self.matches.push((line_idx, actual_pos));
                // 使用查詢字符串的字節長度來避免 UTF-8 字符邊界錯誤
                // 這樣可以正確處理中文等多字節字符
                start = actual_pos + self.query.len();
            }
        }
    }

    /// Make the first match at or after `(row, byte_col)` current, wrapping to the
    /// first match of the file, and return it.
    pub fn select_from(&mut self, row: usize, byte_col: usize) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }
        // matches 依 (行, byte) 排序，二分搜尋游標之後的第一個
        let idx = self.matches.partition_point(|&pos| pos < (row, byte_col));
        self.current_match = if idx == self.matches.len() { 0 } else { idx };
        Some(self.matches[self.current_match])
    }

    pub fn next_match(&mut self) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }

        self.current_match = (self.current_match + 1) % self.matches.len();
        Some(self.matches[self.current_match])
    }

    pub fn prev_match(&mut self) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }

        if self.current_match == 0 {
            self.current_match = self.matches.len() - 1;
        } else {
            self.current_match -= 1;
        }

        Some(self.matches[self.current_match])
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    pub fn current_index(&self) -> usize {
        self.current_match
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }

    pub fn get_matches(&self) -> &[(usize, usize)] {
        &self.matches
    }
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_from_starts_at_cursor_and_wraps() {
        // 稽核 F32：搜尋從游標處開始，而非永遠從檔首
        let mut buffer = RopeBuffer::new();
        buffer.insert(0, "ab\nxx\nab ab\nxx\n");
        let mut search = Search::new();
        search.set_query("ab".to_string());
        search.find_matches(&buffer);
        assert_eq!(search.select_from(1, 0), Some((2, 0)));
        assert_eq!(search.current_index(), 1);
        assert_eq!(search.next_match(), Some((2, 3)));
        // 游標正好在匹配上時選中該匹配
        assert_eq!(search.select_from(2, 3), Some((2, 3)));
        // 最後一個匹配之後回到檔首
        assert_eq!(search.select_from(3, 0), Some((0, 0)));
        assert_eq!(search.current_index(), 0);
    }
}
