use crate::buffer::RopeBuffer;
use crate::comment::CommentHandler;
use crate::cursor::Cursor;
use crate::terminal::Terminal;
use crate::utils::visual_width;
use anyhow::Result;
use crossterm::{
    cursor, execute, queue,
    style::{self, Attribute, Color},
};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub start: (usize, usize), // (row, col)
    pub end: (usize, usize),   // (row, col)
}

pub struct View {
    pub offset_row: usize, // 視窗頂部顯示的行號
    pub show_line_numbers: bool,
    pub screen_rows: usize,
    pub screen_cols: usize,
}

impl View {
    pub fn new(terminal: &Terminal) -> Self {
        let (cols, rows) = terminal.size();
        Self {
            offset_row: 0,
            show_line_numbers: true,
            screen_rows: rows.saturating_sub(1) as usize, // 減去狀態欄
            screen_cols: cols as usize,
        }
    }

    #[allow(dead_code)]
    pub fn update_size(&mut self) {
        let size = crossterm::terminal::size().unwrap_or((80, 24));
        self.screen_rows = size.1.saturating_sub(1) as usize;
        self.screen_cols = size.0 as usize;
    }

    pub fn render(
        &mut self,
        buffer: &RopeBuffer,
        cursor: &Cursor,
        selection: Option<&Selection>,
        message: Option<&str>,
        comment_handler: &CommentHandler,
    ) -> Result<()> {
        // 判斷是否有 debug 標尺
        let has_debug_ruler = message.is_some_and(|m| m.starts_with("DEBUG"));

        self.scroll_if_needed(cursor, buffer, has_debug_ruler);

        let mut stdout = io::stdout();

        // 隱藏光標
        execute!(stdout, cursor::Hide)?;

        // 移動到左上角但不清空屏幕
        execute!(stdout, cursor::MoveTo(0, 0))?;

        // 渲染列標尺（debug模式下才顯示）
        let ruler_offset = if message.is_some_and(|m| m.starts_with("DEBUG")) {
            self.render_column_ruler(&mut stdout, buffer)?;
            1 // 佔用一行
        } else {
            0
        };

        // 計算行號寬度
        let line_num_width = self.calculate_line_number_width(buffer);

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
                Self::calculate_visual_column(&line, start_col)
            } else {
                start_col
            };

            // 將end_col轉換為視覺列
            let end_visual_col = if end_row < buffer.line_count() {
                let line = buffer
                    .line(end_row)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                Self::calculate_visual_column(&line, end_col)
            } else {
                end_col
            };

            ((start_row, start_visual_col), (end_row, end_visual_col))
        });

        // 渲染文本行
        let mut screen_row = ruler_offset; // 從標尺行之後開始
        let mut file_row = self.offset_row;

        while screen_row < self.screen_rows && file_row < buffer.line_count() {
            queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;

            // 顯示行號（只在該文件行的第一個視覺行顯示）
            if self.show_line_numbers {
                let line_num = format!("{:>width$} ", file_row + 1, width = line_num_width - 1);
                queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
                queue!(stdout, style::Print(&line_num))?;
                queue!(stdout, style::ResetColor)?;
            }

            // 顯示行內容
            if let Some(line) = buffer.line(file_row) {
                let line_str = line.to_string();
                let line_str = line_str.trim_end_matches(['\n', '\r']);

                // 查找註解符號的起始位置(在原始字符串中的字符索引)
                let comment_start_char_idx = comment_handler.find_comment_start(line_str);

                // 處理Tab鍵顯示為空格,並計算註解位置在顯示字符串中的索引
                let mut displayed_line = String::new();
                let mut comment_display_idx = None;

                for (char_idx, ch) in line_str.chars().enumerate() {
                    // 如果這是註解開始的字符,記錄當前顯示字符串的長度(即將添加的字符位置)
                    if comment_start_char_idx == Some(char_idx) {
                        comment_display_idx = Some(visual_width(&displayed_line));
                    }

                    if ch == '\t' {
                        displayed_line.push_str("    ");
                    } else {
                        displayed_line.push(ch);
                    }
                }

                // 可用寬度（保留一個字符的邊距，避免自動換行）
                let available_width = self
                    .screen_cols
                    .saturating_sub(line_num_width)
                    .saturating_sub(1);

                // 將行內容按可用寬度切分成多個視覺行
                let visual_lines = self.wrap_line(&displayed_line, available_width);

                // 渲染所有視覺行
                for (visual_idx, visual_line) in visual_lines.iter().enumerate() {
                    if screen_row >= self.screen_rows {
                        break;
                    }

                    // 如果是後續視覺行，需要移到新行並空出行號位置
                    if visual_idx > 0 {
                        screen_row += 1;
                        if screen_row >= self.screen_rows {
                            break;
                        }
                        queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;

                        // 空出行號位置
                        if self.show_line_numbers {
                            for _ in 0..line_num_width {
                                queue!(stdout, style::Print(" "))?;
                            }
                        }
                    }

                    // 計算當前視覺行在整個displayed_line中的起始位置
                    let mut visual_line_start = 0;
                    for line in visual_lines.iter().take(visual_idx) {
                        visual_line_start += visual_width(line);
                    }
                    let visual_line_end = visual_line_start + visual_width(visual_line);

                    // 如果註解在當前視覺行之前或之內，則整個視覺行需要部分或全部變色
                    let is_comment_started =
                        comment_display_idx.is_some_and(|pos| pos < visual_line_end);
                    let comment_pos_in_visual_line = if is_comment_started {
                        comment_display_idx.map(|pos| pos.saturating_sub(visual_line_start))
                    } else {
                        None
                    };

                    // 處理選擇高亮
                    if let Some(((start_row, start_col), (end_row, end_col))) = sel_visual_range {
                        if file_row >= start_row && file_row <= end_row {
                            // 這一行有選擇
                            let chars: Vec<char> = visual_line.chars().collect();
                            let mut current_visual_pos = visual_line_start;

                            for &ch in chars.iter() {
                                // 計算當前字符的視覺寬度
                                let ch_width =
                                    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);

                                let is_selected = if file_row == start_row && file_row == end_row {
                                    current_visual_pos >= start_col && current_visual_pos < end_col
                                } else if file_row == start_row {
                                    current_visual_pos >= start_col
                                } else if file_row == end_row {
                                    current_visual_pos < end_col
                                } else {
                                    true
                                };

                                let is_in_comment = comment_pos_in_visual_line.is_some_and(|pos| {
                                    current_visual_pos >= visual_line_start + pos
                                });

                                if is_selected {
                                    queue!(stdout, style::SetAttribute(Attribute::Reverse))?;
                                } else if is_in_comment {
                                    queue!(stdout, style::SetForegroundColor(Color::DarkGreen))?;
                                }
                                queue!(stdout, style::Print(ch))?;
                                if is_selected {
                                    queue!(stdout, style::SetAttribute(Attribute::NoReverse))?;
                                } else if is_in_comment {
                                    queue!(stdout, style::ResetColor)?;
                                }

                                // 移動視覺位置
                                current_visual_pos += ch_width;
                            }
                        } else {
                            // 沒有選擇，如果有註解則部分變色
                            if let Some(comment_pos) = comment_pos_in_visual_line {
                                let chars: Vec<char> = visual_line.chars().collect();
                                for (idx, &ch) in chars.iter().enumerate() {
                                    if idx >= comment_pos {
                                        if idx == comment_pos {
                                            queue!(
                                                stdout,
                                                style::SetForegroundColor(Color::DarkGreen)
                                            )?;
                                        }
                                        queue!(stdout, style::Print(ch))?;
                                    } else {
                                        queue!(stdout, style::Print(ch))?;
                                    }
                                }
                                queue!(stdout, style::ResetColor)?;
                            } else {
                                queue!(stdout, style::Print(visual_line))?;
                            }
                        }
                    } else {
                        // 沒有選擇，如果有註解則部分變色
                        if let Some(comment_pos) = comment_pos_in_visual_line {
                            let chars: Vec<char> = visual_line.chars().collect();
                            for (idx, &ch) in chars.iter().enumerate() {
                                if idx >= comment_pos {
                                    if idx == comment_pos {
                                        queue!(
                                            stdout,
                                            style::SetForegroundColor(Color::DarkGreen)
                                        )?;
                                    }
                                    queue!(stdout, style::Print(ch))?;
                                } else {
                                    queue!(stdout, style::Print(ch))?;
                                }
                            }
                            queue!(stdout, style::ResetColor)?;
                        } else {
                            queue!(stdout, style::Print(visual_line))?;
                        }
                    }

                    // 清除行的剩餘部分
                    queue!(
                        stdout,
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
                    )?;
                }
            }

            screen_row += 1;
            file_row += 1;
        }

        // 填充剩餘的螢幕行
        while screen_row < self.screen_rows {
            queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;
            // 空行顯示波浪號
            queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
            queue!(stdout, style::Print("~"))?;
            queue!(stdout, style::ResetColor)?;

            // 清除行的剩餘部分
            queue!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
            )?;

            screen_row += 1;
        }

        // 渲染狀態欄
        self.render_status_bar(buffer, message, cursor)?;

        // 計算光標的螢幕位置（考慮換行）
        let line_num_width = self.calculate_line_number_width(buffer);
        let available_width = self
            .screen_cols
            .saturating_sub(line_num_width)
            .saturating_sub(1);

        // 計算光標Y位置：累計從offset_row到cursor.row之前所有行的視覺行數
        let mut cursor_screen_y = 0;
        for row in self.offset_row..cursor.row {
            if row < buffer.line_count() {
                if let Some(line) = buffer.line(row) {
                    let line_str = line.to_string();
                    let line_str = line_str.trim_end_matches(['\n', '\r']);

                    // 處理Tab展開
                    let mut displayed_line = String::new();
                    for ch in line_str.chars() {
                        if ch == '\t' {
                            displayed_line.push_str("    ");
                        } else {
                            displayed_line.push(ch);
                        }
                    }

                    let visual_lines = self.wrap_line(&displayed_line, available_width);
                    cursor_screen_y += visual_lines.len();
                }
            }
        }

        // 計算光標在當前行的視覺列和視覺行索引
        let current_line = buffer
            .line(cursor.row)
            .map(|s| s.to_string())
            .unwrap_or_default();

        // 處理Tab展開
        let mut displayed_line = String::new();
        for ch in current_line.trim_end_matches(['\n', '\r']).chars() {
            if ch == '\t' {
                displayed_line.push_str("    ");
            } else {
                displayed_line.push(ch);
            }
        }

        let visual_col = Self::calculate_visual_column(&current_line, cursor.col);
        let visual_lines = self.wrap_line(&displayed_line, available_width);

        // 找出光標在哪個視覺行
        let mut accumulated_width = 0;
        let mut visual_line_idx = 0;
        let mut col_in_visual_line = visual_col;

        for (idx, vline) in visual_lines.iter().enumerate() {
            let vline_width = visual_width(vline);
            if visual_col < accumulated_width + vline_width || idx == visual_lines.len() - 1 {
                visual_line_idx = idx;
                col_in_visual_line = visual_col - accumulated_width;
                break;
            }
            accumulated_width += vline_width;
        }

        cursor_screen_y += visual_line_idx;

        let cursor_x = if self.show_line_numbers {
            (line_num_width + col_in_visual_line) as u16
        } else {
            col_in_visual_line as u16
        };
        let cursor_y = (cursor_screen_y + ruler_offset) as u16; // 加上標尺偏移

        // 設置普通光標位置
        execute!(stdout, cursor::MoveTo(cursor_x, cursor_y))?;
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
            return;
        }

        // 向下滾動：基於視覺行數判斷
        let effective_rows = self.get_effective_screen_rows(has_debug_ruler);
        let visual_offset =
            self.count_visual_lines(buffer, self.offset_row, cursor.row) + cursor.visual_line_index;

        // 如果光標視覺位置超出螢幕，向下調整 offset_row
        if visual_offset >= effective_rows {
            while self.offset_row < cursor.row {
                self.offset_row += 1;
                let new_offset = self.count_visual_lines(buffer, self.offset_row, cursor.row)
                    + cursor.visual_line_index;
                if new_offset < effective_rows {
                    break;
                }
            }
        }
    }

    fn render_status_bar(
        &self,
        buffer: &RopeBuffer,
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

        let status = if let Some(msg) = message {
            // 如果有消息，優先顯示消息
            format!(" {}{} - {}", filename, modified, msg)
        } else {
            format!(
                " {}{}  Line {}/{}  Ctrl+S:Save Ctrl+Q:Quit",
                filename,
                modified,
                cursor.row + 1,
                buffer.line_count()
            )
        };

        // 確保狀態欄填滿整行
        let status = if visual_width(&status) < self.screen_cols {
            format!("{:width$}", status, width = self.screen_cols)
        } else {
            // 使用視覺寬度安全截斷
            let mut result = String::new();
            let mut current_width = 0;
            for ch in status.chars() {
                let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
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

    /// 將行按可用寬度切分成多個視覺行
    fn wrap_line(&self, line: &str, max_width: usize) -> Vec<String> {
        if max_width == 0 {
            return vec![String::new()];
        }

        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for ch in line.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);

            // 如果加入這個字符會超過或等於寬度限制，且當前行不為空
            if current_width + char_width > max_width && !current_line.is_empty() {
                // 保存當前行並開始新行
                result.push(current_line);
                current_line = String::new();
                current_width = 0;
            }

            // 將字符加入當前行
            current_line.push(ch);
            current_width += char_width;
        }

        // 添加最後一行
        if !current_line.is_empty() {
            result.push(current_line);
        }

        // 確保至少有一個空行
        if result.is_empty() {
            result.push(String::new());
        }

        result
    }

    /// 計算考慮 Tab 展開和字符寬度的視覺列位置
    /// Tab 會被展開為 4 個空格顯示
    /// 中文字符等寬字符會佔用 2 個顯示位置
    fn calculate_visual_column(line: &str, buffer_col: usize) -> usize {
        let mut visual_col = 0;
        for (idx, ch) in line.chars().enumerate() {
            if idx >= buffer_col {
                break;
            }
            if ch == '\t' {
                visual_col += 4; // Tab 展開為 4 個空格
            } else {
                // 使用 unicode-width 計算字符實際寬度
                visual_col += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            }
        }
        visual_col
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

    /// 計算指定邏輯行的視覺行分割
    pub fn calculate_visual_lines_for_row(&self, buffer: &RopeBuffer, row: usize) -> Vec<String> {
        let available_width = self.get_available_width(buffer);

        if row >= buffer.line_count() {
            return vec![String::new()];
        }

        let line = buffer.line(row).map(|s| s.to_string()).unwrap_or_default();
        let line = line.trim_end_matches(['\n', '\r']);

        // 處理 Tab 展開
        let mut displayed_line = String::new();
        for ch in line.chars() {
            if ch == '\t' {
                displayed_line.push_str("    ");
            } else {
                displayed_line.push(ch);
            }
        }

        self.wrap_line(&displayed_line, available_width)
    }

    /// 將邏輯列轉換為視覺列（考慮 Tab 展開和字符寬度）
    pub fn logical_col_to_visual_col(&self, line: &str, logical_col: usize) -> usize {
        Self::calculate_visual_column(line, logical_col)
    }

    /// 從視覺行索引和視覺列轉換為邏輯列
    pub fn visual_to_logical_col(
        &self,
        buffer: &RopeBuffer,
        row: usize,
        visual_line_index: usize,
        visual_col: usize,
    ) -> usize {
        let visual_lines = self.calculate_visual_lines_for_row(buffer, row);

        if visual_line_index >= visual_lines.len() {
            return 0;
        }

        // 計算前面視覺行的總視覺寬度
        let mut accumulated_width = 0;
        for line in visual_lines.iter().take(visual_line_index) {
            accumulated_width += visual_width(line);
        }

        // 加上當前視覺行內的列位置
        let col_in_visual = visual_col.min(visual_width(&visual_lines[visual_line_index]));
        let visual_col_total = accumulated_width + col_in_visual;

        // 轉換回邏輯列（反向處理 Tab）
        if let Some(line) = buffer.line(row) {
            let line_str = line.to_string();
            let line_str = line_str.trim_end_matches(['\n', '\r']);

            let mut logical_col = 0;
            let mut current_visual = 0;

            for ch in line_str.chars() {
                if current_visual >= visual_col_total {
                    break;
                }

                if ch == '\t' {
                    current_visual += 4;
                } else {
                    current_visual += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                }

                logical_col += 1;
            }

            logical_col
        } else {
            0
        }
    }

    /// 計算從 start_row 到 end_row（不含）之間的總視覺行數
    pub fn count_visual_lines(
        &self,
        buffer: &RopeBuffer,
        start_row: usize,
        end_row: usize,
    ) -> usize {
        let mut total = 0;
        for row in start_row..end_row.min(buffer.line_count()) {
            let visual_lines = self.calculate_visual_lines_for_row(buffer, row);
            total += visual_lines.len();
        }
        total
    }

    /// 獲取實際可用於顯示文本的螢幕行數（扣除 debug 標尺）
    pub fn get_effective_screen_rows(&self, has_debug_ruler: bool) -> usize {
        if has_debug_ruler {
            self.screen_rows.saturating_sub(1)
        } else {
            self.screen_rows
        }
    }

    /// 渲染列標尺（顯示列位置個位數字）
    fn render_column_ruler(&self, stdout: &mut io::Stdout, buffer: &RopeBuffer) -> Result<()> {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;

        let line_num_width = self.calculate_line_number_width(buffer);

        // 輸出行號區域的空白
        for _ in 0..line_num_width {
            queue!(stdout, style::Print(" "))?;
        }

        // 輸出列標尺數字（個位數）
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
