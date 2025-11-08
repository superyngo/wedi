// 自動換行邏輯
// 這個模組將在後續階段實現

#[allow(dead_code)]
pub struct LineWrapper {
    max_width: usize,
}

impl LineWrapper {
    #[allow(dead_code)]
    pub fn new(max_width: usize) -> Self {
        Self { max_width }
    }

    #[allow(dead_code)]
    pub fn wrap_line(&self, line: &str) -> Vec<String> {
        if line.len() <= self.max_width {
            return vec![line.to_string()];
        }

        let mut wrapped = Vec::new();
        let mut current = String::new();
        let mut current_width = 0;

        for ch in line.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);

            if current_width + char_width > self.max_width {
                wrapped.push(current.clone());
                current.clear();
                current_width = 0;
            }

            current.push(ch);
            current_width += char_width;
        }

        if !current.is_empty() {
            wrapped.push(current);
        }

        wrapped
    }

    #[allow(dead_code)]
    pub fn set_max_width(&mut self, width: usize) {
        self.max_width = width;
    }
}
