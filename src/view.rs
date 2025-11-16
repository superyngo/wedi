use crate::buffer::RopeBuffer;
use crate::cursor::Cursor;
use crate::terminal::Terminal;
use crate::utils::visual_width;
use anyhow::Result;
use crossterm::{
    cursor, execute, queue,
    style::{self, Attribute, Color},
};
use std::io::{self, Write};
use unicode_width::UnicodeWidthChar;

const TAB_WIDTH: usize = 4;

#[derive(Clone, Debug)]
pub struct LineLayout {
    /// 視覺行（已處理 Tab 並依螢幕寬度換行）
    pub visual_lines: Vec<String>,
    /// 視覺行高度（visual_lines.len()）
    pub visual_height: usize,
    /// logical_col -> visual_col（整行累計視覺座標）
    pub logical_to_visual: Vec<usize>,
}

impl LineLayout {
    pub fn new(buffer: &RopeBuffer, row: usize, available_width: usize) -> Option<Self> {
        let line = buffer.line(row)?;
        let mut line_str = line.to_string();
        // 去掉結尾換行符
        while matches!(line_str.chars().last(), Some('\n' | '\r')) {
            line_str.pop();
        }

        let (displayed_line, logical_to_visual) = expand_tabs_and_build_map(&line_str);
        let visual_lines = wrap_line(&displayed_line, available_width);
        let visual_height = visual_lines.len();

        Some(LineLayout {
            visual_lines,
            visual_height,
            logical_to_visual,
        })
    }
}

fn expand_tabs_and_build_map(line: &str) -> (String, Vec<usize>) {
    let mut displayed = String::new();
    let mut logical_to_visual = Vec::new();
    let mut visual_col = 0;

    for ch in line.chars() {
        // 記錄「這個 logical_col 對應的視覺座標」
        logical_to_visual.push(visual_col);

        if ch == '\t' {
            for _ in 0..TAB_WIDTH {
                displayed.push(' ');
            }
            visual_col += TAB_WIDTH;
        } else {
            let w = UnicodeWidthChar::width(ch).unwrap_or(1);
            displayed.push(ch);
            visual_col += w;
        }
    }

    // 尾端一個 mapping，讓「行尾」也有對應視覺座標
    logical_to_visual.push(visual_col);

    (displayed, logical_to_visual)
}

#[allow(dead_code)]
fn calculate_hash(line: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub start: (usize, usize), // (row, col)
    pub end: (usize, usize),   // (row, col)
}

pub struct View {
    pub offset_row: usize, // 視窗頂部顯示的行號（邏輯行）
    pub show_line_numbers: bool,
    pub screen_rows: usize,
    pub screen_cols: usize,
    // 行快取：從 offset_row 起往下的數行
    line_layout_cache: Vec<Option<LineLayout>>,
}

impl View {
    pub fn new(terminal: &Terminal) -> Self {
        let (cols, rows) = terminal.size();
        let screen_rows = rows.saturating_sub(1) as usize; // 減去狀態欄
        let cache_size = screen_rows.max(1) * 3; // 多留一些緩衝高度

        Self {
            offset_row: 0,
            show_line_numbers: true,
            screen_rows,
            screen_cols: cols as usize,
            line_layout_cache: vec![None; cache_size],
        }
    }

    pub fn invalidate_cache(&mut self) {
        let cache_size = self.screen_rows.max(1) * 3;
        self.line_layout_cache.clear();
        self.line_layout_cache.resize(cache_size, None);
    }

    #[allow(dead_code)]
    pub fn update_size(&mut self) {
        let size = crossterm::terminal::size().unwrap_or((80, 24));
        let new_screen_rows = size.1.saturating_sub(1) as usize;
        let new_screen_cols = size.0 as usize;

        if self.screen_rows != new_screen_rows || self.screen_cols != new_screen_cols {
            self.screen_rows = new_screen_rows;
            self.screen_cols = new_screen_cols;
            self.invalidate_cache(); // 寬度或高度改變時使快取失效
        }
    }

    pub fn render(
        &mut self,
        buffer: &RopeBuffer,
        cursor: &Cursor,
        selection: Option<&Selection>,
        message: Option<&str>,
    ) -> Result<()> {
        let has_debug_ruler = message.is_some_and(|m| m.starts_with("DEBUG"));

        self.scroll_if_needed(cursor, buffer, has_debug_ruler);

        let mut stdout = io::stdout();

        execute!(stdout, cursor::Hide)?;
        execute!(stdout, cursor::MoveTo(0, 0))?;

        let ruler_offset = if has_debug_ruler {
            self.render_column_ruler(&mut stdout, buffer)?;
            1
        } else {
            0
        };

        let line_num_width = self.calculate_line_number_width(buffer);
        let available_width = self.get_available_width(buffer);

        // 計算選擇範圍（轉換為視覺列）
        let sel_visual_range = selection.map(|sel| {
            let (start_row, start_col) = sel.start.min(sel.end);
            let (end_row, end_col) = sel.start.max(sel.end);

            // 將start_col轉換為視覺列
            let start_visual_col = if start_row < buffer.line_count() {
                let line = buffer
                    .line(start_row)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let line = line.trim_end_matches(['\n', '\r']);
                self.logical_col_to_visual_col(line, start_col)
            } else {
                start_col
            };

            // 將end_col轉換為視覺列
            let end_visual_col = if end_row < buffer.line_count() {
                let line = buffer
                    .line(end_row)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let line = line.trim_end_matches(['\n', '\r']);
                self.logical_col_to_visual_col(line, end_col)
            } else {
                end_col
            };

            ((start_row, start_visual_col), (end_row, end_visual_col))
        });

        let mut screen_row = ruler_offset;
        let mut file_row = self.offset_row;

        while screen_row < self.screen_rows && file_row < buffer.line_count() {
            queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;

            if self.show_line_numbers {
                let line_num = format!("{:>width$} ", file_row + 1, width = line_num_width - 1);
                queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
                queue!(stdout, style::Print(&line_num))?;
                queue!(stdout, style::ResetColor)?;
            }

            let cache_index = file_row.saturating_sub(self.offset_row);
            let layout_opt = self
                .line_layout_cache
                .get(cache_index)
                .and_then(|l| l.as_ref())
                .cloned();

            let layout = if let Some(layout) = layout_opt {
                layout
            } else if let Some(new_layout) = LineLayout::new(buffer, file_row, available_width) {
                if cache_index < self.line_layout_cache.len() {
                    self.line_layout_cache[cache_index] = Some(new_layout.clone());
                }
                new_layout
            } else {
                // 空行或超出範圍
                LineLayout {
                    visual_lines: vec![String::new()],
                    visual_height: 1,
                    logical_to_visual: vec![0],
                }
            };

            for (visual_idx, visual_line) in layout.visual_lines.iter().enumerate() {
                if screen_row >= self.screen_rows {
                    break;
                }

                if visual_idx > 0 {
                    screen_row += 1;
                    if screen_row >= self.screen_rows {
                        break;
                    }
                    queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;

                    if self.show_line_numbers {
                        for _ in 0..line_num_width {
                            queue!(stdout, style::Print(" "))?;
                        }
                    }
                }

                // 渲染視覺行，支持selection高亮
                if let Some(((start_row, start_col), (end_row, end_col))) = sel_visual_range {
                    if file_row >= start_row && file_row <= end_row {
                        // 這一行有選擇，需要逐字符渲染
                        // 計算這個visual_line在整個邏輯行中的視覺起始位置
                        let visual_line_start: usize = layout
                            .visual_lines
                            .iter()
                            .take(visual_idx)
                            .map(|line| visual_width(line))
                            .sum();

                        let chars: Vec<char> = visual_line.chars().collect();
                        let mut current_visual_pos = visual_line_start;

                        for &ch in chars.iter() {
                            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);

                            // 判斷這個字符是否在選擇範圍內
                            let is_selected = if file_row == start_row && file_row == end_row {
                                // 選擇在同一行
                                current_visual_pos >= start_col && current_visual_pos < end_col
                            } else if file_row == start_row {
                                // 選擇起始行
                                current_visual_pos >= start_col
                            } else if file_row == end_row {
                                // 選擇結束行
                                current_visual_pos < end_col
                            } else {
                                // 選擇中間的行，全選
                                true
                            };

                            if is_selected {
                                queue!(stdout, style::SetAttribute(Attribute::Reverse))?;
                            }
                            queue!(stdout, style::Print(ch))?;
                            if is_selected {
                                queue!(stdout, style::SetAttribute(Attribute::NoReverse))?;
                            }

                            current_visual_pos += ch_width;
                        }
                    } else {
                        // 這一行沒有選擇，直接打印
                        queue!(stdout, style::Print(visual_line))?;
                    }
                } else {
                    // 沒有選擇，直接打印
                    queue!(stdout, style::Print(visual_line))?;
                }

                queue!(
                    stdout,
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
                )?;
            }

            screen_row += 1;
            file_row += 1;
        }

        // 畫底部的 ~ 行
        while screen_row < self.screen_rows {
            queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;
            queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
            queue!(stdout, style::Print("~"))?;
            queue!(stdout, style::ResetColor)?;
            queue!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
            )?;
            screen_row += 1;
        }

        self.render_status_bar(buffer, selection.is_some(), message, cursor)?;

        // 移動終端光標到當前cursor位置
        let ruler_offset = if has_debug_ruler { 1 } else { 0 };
        let (cursor_x, cursor_y) = self.get_cursor_visual_position(cursor, buffer);
        let cursor_y = cursor_y + ruler_offset;
        execute!(stdout, cursor::MoveTo(cursor_x as u16, cursor_y as u16))?;

        execute!(stdout, cursor::Show)?;
        stdout.flush()?;
        Ok(())
    }

    pub fn scroll_if_needed(
        &mut self,
        cursor: &Cursor,
        buffer: &RopeBuffer,
        has_debug_ruler: bool,
    ) {
        // 向上滾動
        if cursor.row < self.offset_row {
            self.offset_row = cursor.row;
            self.invalidate_cache();
            return;
        }

        let effective_rows = self.get_effective_screen_rows(has_debug_ruler);

        // 計算目前 offset_row ~ cursor.row 的視覺高度
        let mut visual_offset = 0;
        let available_width = self.get_available_width(buffer);

        for row in self.offset_row..=cursor.row {
            let cache_index = row.saturating_sub(self.offset_row);
            if let Some(Some(layout)) = self.line_layout_cache.get(cache_index) {
                visual_offset += layout.visual_height;
            } else if let Some(layout) = LineLayout::new(buffer, row, available_width) {
                visual_offset += layout.visual_height;
                if cache_index < self.line_layout_cache.len() {
                    self.line_layout_cache[cache_index] = Some(layout);
                }
            }
        }

        // 如果沒超出螢幕，就不用動
        if visual_offset < effective_rows {
            return;
        }

        // 向下推 offset_row，每次扣掉最上面那一行的視覺高度
        while self.offset_row < cursor.row && visual_offset >= effective_rows {
            let top_layout_opt = self
                .line_layout_cache
                .first()
                .and_then(|l| l.as_ref())
                .cloned();

            if let Some(layout) = top_layout_opt {
                visual_offset = visual_offset.saturating_sub(layout.visual_height);
            } else if let Some(layout) = LineLayout::new(buffer, self.offset_row, available_width) {
                visual_offset = visual_offset.saturating_sub(layout.visual_height);
                if !self.line_layout_cache.is_empty() {
                    self.line_layout_cache[0] = Some(layout);
                }
            }

            self.offset_row += 1;

            if !self.line_layout_cache.is_empty() {
                self.line_layout_cache.remove(0);
                self.line_layout_cache.push(None);
            }
        }
    }

    fn render_status_bar(
        &self,
        buffer: &RopeBuffer,
        selection_mode: bool,
        message: Option<&str>,
        cursor: &Cursor,
    ) -> Result<()> {
        let mut stdout = io::stdout();
        queue!(stdout, cursor::MoveTo(0, self.screen_rows as u16))?;

        queue!(stdout, style::SetBackgroundColor(Color::DarkGrey))?;
        queue!(stdout, style::SetForegroundColor(Color::White))?;

        let modified = if buffer.is_modified() {
            " [modified]"
        } else {
            ""
        };
        let filename = buffer.file_name();

        let mode_indicator = if selection_mode {
            " [Selection Mode]"
        } else {
            ""
        };

        let status = if let Some(msg) = message {
            format!(" {}{}{}  - {}", filename, modified, mode_indicator, msg)
        } else {
            format!(
                " {}{}{}  Line {}/{}  Ctrl+W:Save Ctrl+Q:Quit",
                filename,
                modified,
                mode_indicator,
                cursor.row + 1,
                buffer.line_count()
            )
        };

        // 確保狀態欄填滿整行（使用視覺寬度）
        let status = if visual_width(&status) < self.screen_cols {
            format!("{:width$}", status, width = self.screen_cols)
        } else {
            let mut result = String::new();
            let mut current_width = 0;
            for ch in status.chars() {
                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
                if current_width + ch_width > self.screen_cols {
                    break;
                }
                result.push(ch);
                current_width += ch_width;
            }
            result
        };

        queue!(stdout, style::Print(status))?;
        queue!(stdout, style::ResetColor)?;

        Ok(())
    }

    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// 計算行號寬度（包含右側空格）
    fn calculate_line_number_width(&self, buffer: &RopeBuffer) -> usize {
        if self.show_line_numbers {
            buffer.line_count().to_string().len() + 1
        } else {
            0
        }
    }

    /// 獲取可用於顯示內容的寬度（扣除行號寬度）
    pub fn get_available_width(&self, buffer: &RopeBuffer) -> usize {
        let line_num_width = self.calculate_line_number_width(buffer);
        self.screen_cols
            .saturating_sub(line_num_width)
            .saturating_sub(1)
    }

    /// 計算指定邏輯行的視覺行分割（給其他模組用，不依賴 cache 也可以）
    pub fn calculate_visual_lines_for_row(&self, buffer: &RopeBuffer, row: usize) -> Vec<String> {
        if row >= buffer.line_count() {
            return vec![String::new()];
        }

        // 如果 row 剛好在快取範圍內，優先使用快取
        let cache_index = row.saturating_sub(self.offset_row);
        if let Some(Some(layout)) = self.line_layout_cache.get(cache_index) {
            return layout.visual_lines.clone();
        }

        let available_width = self.get_available_width(buffer);
        let line = buffer.line(row).map(|s| s.to_string()).unwrap_or_default();
        let mut line = line;
        while matches!(line.chars().last(), Some('\n' | '\r')) {
            line.pop();
        }

        let (displayed_line, _) = expand_tabs_and_build_map(&line);
        wrap_line(&displayed_line, available_width)
    }

    /// 將邏輯列轉換為視覺列（考慮 Tab 展開和字符寬度）
    pub fn logical_col_to_visual_col(&self, line: &str, logical_col: usize) -> usize {
        // 這個函式目前只拿到一行字串，不知道 row，無法用 cache。
        // 保留原來的行為：直接掃一遍。
        let mut visual_col = 0;
        for (idx, ch) in line.chars().enumerate() {
            if idx >= logical_col {
                break;
            }
            if ch == '\t' {
                visual_col += TAB_WIDTH;
            } else {
                visual_col += UnicodeWidthChar::width(ch).unwrap_or(1);
            }
        }
        visual_col
    }

    /// 從視覺行索引和視覺列轉換為邏輯列
    pub fn visual_to_logical_col(
        &self,
        buffer: &RopeBuffer,
        row: usize,
        visual_line_index: usize,
        visual_col: usize,
    ) -> usize {
        // 優先使用快取（如果該行目前在視窗 cache 內）
        let cache_index = row.saturating_sub(self.offset_row);
        if let Some(Some(layout)) = self.line_layout_cache.get(cache_index) {
            if visual_line_index >= layout.visual_lines.len() {
                return 0;
            }

            // 計算前面視覺行的總視覺寬度
            let mut accumulated_width = 0;
            for line in layout.visual_lines.iter().take(visual_line_index) {
                accumulated_width += visual_width(line);
            }

            // 加上當前視覺行內的列位置
            let col_in_visual =
                visual_col.min(visual_width(&layout.visual_lines[visual_line_index]));
            let visual_col_total = accumulated_width + col_in_visual;

            // 在 logical_to_visual 中尋找「視覺座標 >= visual_col_total」的最小 logical_col
            let mut logical_col = 0;
            for (idx, &vcol) in layout.logical_to_visual.iter().enumerate() {
                if vcol > visual_col_total {
                    break;
                }
                logical_col = idx;
            }
            return logical_col;
        }

        // 若不在 cache 範圍，退回原本的計算方式（慢但安全）
        let visual_lines = self.calculate_visual_lines_for_row(buffer, row);

        if visual_line_index >= visual_lines.len() {
            return 0;
        }

        // 計算前面視覺行的總視覺寬度
        let mut accumulated_width = 0;
        for line in visual_lines.iter().take(visual_line_index) {
            accumulated_width += visual_width(line);
        }

        let col_in_visual = visual_col.min(visual_width(&visual_lines[visual_line_index]));
        let visual_col_total = accumulated_width + col_in_visual;

        if let Some(line) = buffer.line(row) {
            let mut line_str = line.to_string();
            while matches!(line_str.chars().last(), Some('\n' | '\r')) {
                line_str.pop();
            }

            let mut logical_col = 0;
            let mut current_visual = 0;

            for ch in line_str.chars() {
                if current_visual >= visual_col_total {
                    break;
                }

                if ch == '\t' {
                    current_visual += TAB_WIDTH;
                } else {
                    current_visual += UnicodeWidthChar::width(ch).unwrap_or(1);
                }

                logical_col += 1;
            }

            logical_col
        } else {
            0
        }
    }

    /// 實際可用於顯示文本的螢幕行數（扣除 debug 標尺）
    pub fn get_effective_screen_rows(&self, has_debug_ruler: bool) -> usize {
        if has_debug_ruler {
            self.screen_rows.saturating_sub(1)
        } else {
            self.screen_rows
        }
    }

    /// 獲取cursor的視覺位置（螢幕座標）
    pub fn get_cursor_visual_position(
        &self,
        cursor: &Cursor,
        buffer: &RopeBuffer,
    ) -> (usize, usize) {
        let line_num_width = self.calculate_line_number_width(buffer);

        // 計算cursor所在的螢幕行
        let mut screen_y = 0;
        let mut file_row = self.offset_row;

        while file_row < cursor.row && screen_y < self.screen_rows {
            let cache_index = file_row.saturating_sub(self.offset_row);
            let layout_opt = self
                .line_layout_cache
                .get(cache_index)
                .and_then(|l| l.as_ref())
                .cloned();

            let layout = if let Some(layout) = layout_opt {
                layout
            } else {
                LineLayout::new(buffer, file_row, self.get_available_width(buffer)).unwrap_or_else(
                    || LineLayout {
                        visual_lines: vec![String::new()],
                        visual_height: 1,
                        logical_to_visual: vec![0],
                    },
                )
            };

            screen_y += layout.visual_height;
            file_row += 1;
        }

        // 添加cursor行內的視覺行偏移
        screen_y += cursor.visual_line_index;

        // 如果超出螢幕，返回最後一行
        let screen_y = screen_y.min(self.screen_rows.saturating_sub(1));

        // 計算cursor在視覺行內的x位置
        let visual_lines = self.calculate_visual_lines_for_row(buffer, cursor.row);
        let mut screen_x = line_num_width;

        if cursor.visual_line_index < visual_lines.len() {
            // 計算前面視覺行的累計寬度
            let mut accumulated_width = 0;
            for line in visual_lines.iter().take(cursor.visual_line_index) {
                accumulated_width += visual_width(line);
            }

            // cursor在整個邏輯行中的視覺col
            let line_str = buffer
                .line(cursor.row)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let line_str = line_str.trim_end_matches(['\n', '\r']);
            let cursor_visual_col = self.logical_col_to_visual_col(line_str, cursor.col);

            // 在當前視覺行內的col
            let visual_col_in_line = cursor_visual_col.saturating_sub(accumulated_width);

            // 加上行號寬度
            screen_x += visual_col_in_line;
        }

        (screen_x, screen_y)
    }

    /// 渲染列標尺（顯示列位置個位數字）
    fn render_column_ruler(&self, stdout: &mut io::Stdout, buffer: &RopeBuffer) -> Result<()> {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;

        let line_num_width = self.calculate_line_number_width(buffer);

        for _ in 0..line_num_width {
            queue!(stdout, style::Print(" "))?;
        }

        let available_cols = self
            .screen_cols
            .saturating_sub(line_num_width)
            .saturating_sub(1);
        for col in 0..available_cols {
            let digit = col % 10;
            queue!(stdout, style::Print(digit))?;
        }

        queue!(stdout, style::ResetColor)?;
        Ok(())
    }
}

/// 將行按可用寬度切分成多個視覺行（共用）
fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for ch in line.chars() {
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(1);

        if current_width + char_width > max_width && !current_line.is_empty() {
            result.push(current_line);
            current_line = String::new();
            current_width = 0;
        }

        current_line.push(ch);
        current_width += char_width;
    }

    if !current_line.is_empty() {
        result.push(current_line);
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}
