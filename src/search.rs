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
                start = actual_pos + 1;
            }
        }
    }

    pub fn next_match(&mut self) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }

        let result = self.matches[self.current_match];
        self.current_match = (self.current_match + 1) % self.matches.len();
        Some(result)
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
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}
