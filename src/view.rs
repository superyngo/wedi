use crate::buffer::RopeBuffer;
use crate::comment::CommentHandler;
use crate::cursor::Cursor;
use crate::terminal::Terminal;
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
        self.scroll_if_needed(cursor);

        let mut stdout = io::stdout();

        // 隱藏光標
        execute!(stdout, cursor::Hide)?;

        // 移動到左上角但不清空屏幕
        execute!(stdout, cursor::MoveTo(0, 0))?;

        // 計算行號寬度
        let line_num_width = if self.show_line_numbers {
            buffer.line_count().to_string().len() + 1
        } else {
            0
        };

        // 計算選擇範圍（轉換為視覺列）
        let sel_visual_range = selection.map(|sel| {
            let (start_row, start_col) = sel.start.min(sel.end);
            let (end_row, end_col) = sel.start.max(sel.end);
            
            // 將start_col轉換為視覺列
            let start_visual_col = if start_row < buffer.line_count() {
                let line = buffer.line(start_row).map(|s| s.to_string()).unwrap_or_default();
                Self::calculate_visual_column(&line, start_col)
            } else {
                start_col
            };
            
            // 將end_col轉換為視覺列
            let end_visual_col = if end_row < buffer.line_count() {
                let line = buffer.line(end_row).map(|s| s.to_string()).unwrap_or_default();
                Self::calculate_visual_column(&line, end_col)
            } else {
                end_col
            };
            
            ((start_row, start_visual_col), (end_row, end_visual_col))
        });

        // 渲染文本行
        let mut screen_row = 0;
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
                        comment_display_idx = Some(displayed_line.chars().count());
                    }
                    
                    if ch == '\t' {
                        displayed_line.push_str("    ");
                    } else {
                        displayed_line.push(ch);
                    }
                }

                // 可用寬度（保留一個字符的邊距，避免自動換行）
                let available_width = self.screen_cols.saturating_sub(line_num_width).saturating_sub(1);
                
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
                    for i in 0..visual_idx {
                        visual_line_start += visual_lines[i].chars().count();
                    }
                    let visual_line_end = visual_line_start + visual_line.chars().count();
                    
                    // 如果註解在當前視覺行之前或之內，則整個視覺行需要部分或全部變色
                    let is_comment_started = comment_display_idx.map_or(false, |pos| pos < visual_line_end);
                    let comment_pos_in_visual_line = if is_comment_started {
                        comment_display_idx.and_then(|pos| {
                            if pos >= visual_line_start {
                                Some(pos - visual_line_start)
                            } else {
                                Some(0) // 註解已經在前面的視覺行開始，整行都是註解
                            }
                        })
                    } else {
                        None
                    };
                    
                    // 處理選擇高亮
                    if let Some(((start_row, start_col), (end_row, end_col))) = sel_visual_range {
                        if file_row >= start_row && file_row <= end_row {
                            // 這一行有選擇
                            let chars: Vec<char> = visual_line.chars().collect();

                            for (idx, &ch) in chars.iter().enumerate() {
                                // 當前字符在displayed_line中的絕對位置
                                let abs_pos = visual_line_start + idx;
                                
                                let is_selected = if file_row == start_row && file_row == end_row {
                                    abs_pos >= start_col && abs_pos < end_col
                                } else if file_row == start_row {
                                    abs_pos >= start_col
                                } else if file_row == end_row {
                                    abs_pos < end_col
                                } else {
                                    true
                                };

                                let is_in_comment = comment_pos_in_visual_line.is_some_and(|pos| idx >= pos);

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
        let line_num_width = if self.show_line_numbers {
            buffer.line_count().to_string().len() + 1
        } else {
            0
        };
        let available_width = self.screen_cols.saturating_sub(line_num_width);
        
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
            let vline_width = vline.chars().count();
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
        let cursor_y = cursor_screen_y as u16;

        // 設置普通光標位置
        execute!(stdout, cursor::MoveTo(cursor_x, cursor_y))?;
        execute!(stdout, cursor::Show)?;

        stdout.flush()?;
        Ok(())
    }

    pub fn scroll_if_needed(&mut self, cursor: &Cursor) {
        // 向上滾動
        if cursor.row < self.offset_row {
            self.offset_row = cursor.row;
        }
        // 向下滾動
        if cursor.row >= self.offset_row + self.screen_rows {
            self.offset_row = cursor.row - self.screen_rows + 1;
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
        let status = if status.len() < self.screen_cols {
            format!("{:width$}", status, width = self.screen_cols)
        } else {
            status[..self.screen_cols].to_string()
        };

        queue!(stdout, style::Print(status))?;
        queue!(stdout, style::ResetColor)?;

        Ok(())
    }

    fn truncate_line(&self, line: &str, max_width: usize) -> String {
        let mut width = 0;
        let mut result = String::new();

        for ch in line.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            if width + char_width > max_width {
                break;
            }
            result.push(ch);
            width += char_width;
        }

        result
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
}
