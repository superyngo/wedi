/// 螢幕佈局資訊
///
/// 管理編輯器視窗的尺寸和滾動位置。
///
/// # 範例
///
/// ```rust
/// use wedi_widget::ScreenLayout;
///
/// let mut layout = ScreenLayout::new(24, 80);
/// layout.scroll_to(10, 0); // 滾動到第 10 行
/// layout.resize(30, 100);  // 調整尺寸
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ScreenLayout {
    /// 視窗頂部顯示的邏輯行號
    pub offset_row: usize,
    /// 水平偏移（單行模式使用）
    pub offset_col: usize,
    /// 螢幕行數
    pub screen_rows: usize,
    /// 螢幕列數
    pub screen_cols: usize,
}

impl ScreenLayout {
    /// 建立新的螢幕佈局
    ///
    /// # Arguments
    /// * `rows` - 螢幕行數
    /// * `cols` - 螢幕列數
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            offset_row: 0,
            offset_col: 0,
            screen_rows: rows,
            screen_cols: cols,
        }
    }

    /// 調整螢幕尺寸
    ///
    /// # Arguments
    /// * `rows` - 新的螢幕行數
    /// * `cols` - 新的螢幕列數
    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.screen_rows = rows;
        self.screen_cols = cols;
    }

    /// 滾動到指定位置
    ///
    /// # Arguments
    /// * `row` - 目標行號
    /// * `col` - 目標列號
    pub fn scroll_to(&mut self, row: usize, col: usize) {
        self.offset_row = row;
        self.offset_col = col;
    }
}
