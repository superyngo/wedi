/// 螢幕佈局資訊
#[derive(Debug, Clone, Copy)]
pub struct ScreenLayout {
    pub offset_row: usize,
    pub offset_col: usize,
    pub screen_rows: usize,
    pub screen_cols: usize,
}

impl ScreenLayout {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            offset_row: 0,
            offset_col: 0,
            screen_rows: rows,
            screen_cols: cols,
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.screen_rows = rows;
        self.screen_cols = cols;
    }

    pub fn scroll_to(&mut self, row: usize, col: usize) {
        self.offset_row = row;
        self.offset_col = col;
    }
}
