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
    pub fn update_size(&mut self, terminal: &Terminal) {
        let (cols, rows) = terminal.size();
        self.screen_rows = rows.saturating_sub(1) as usize;
        self.screen_cols = cols as usize;
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

        // 計算選擇範圍
        let sel_range = selection.map(|sel| {
            let (start_row, start_col) = sel.start.min(sel.end);
            let (end_row, end_col) = sel.start.max(sel.end);
            ((start_row, start_col), (end_row, end_col))
        });

        // 渲染文本行
        for screen_row in 0..self.screen_rows {
            let file_row = self.offset_row + screen_row;

            queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;

            if file_row < buffer.line_count() {
                // 顯示行號
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

                    // 查找註解符號的起始位置
                    let comment_start = comment_handler.find_comment_start(line_str);

                    // 處理Tab鍵顯示為空格
                    let displayed_line = line_str.replace('\t', "    ");

                    // 截斷超出屏幕的部分，並處理選擇高亮
                    let available_width = self.screen_cols.saturating_sub(line_num_width);

                    if let Some(((start_row, start_col), (end_row, end_col))) = sel_range {
                        if file_row >= start_row && file_row <= end_row {
                            // 這一行有選擇
                            let chars: Vec<char> = displayed_line.chars().collect();
                            let mut col = 0;

                            #[allow(clippy::explicit_counter_loop)]
                            for (idx, &ch) in chars.iter().enumerate() {
                                if col >= available_width {
                                    break;
                                }

                                let is_selected = if file_row == start_row && file_row == end_row {
                                    idx >= start_col && idx < end_col
                                } else if file_row == start_row {
                                    idx >= start_col
                                } else if file_row == end_row {
                                    idx < end_col
                                } else {
                                    true
                                };

                                let is_in_comment = comment_start.is_some_and(|pos| idx >= pos);

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
                                col += 1;
                            }
                        } else {
                            // 沒有選擇，如果有註解則部分變色
                            if let Some(comment_pos) = comment_start {
                                let chars: Vec<char> = displayed_line.chars().collect();
                                let mut col = 0;
                                #[allow(clippy::explicit_counter_loop)]
                                for (idx, &ch) in chars.iter().enumerate() {
                                    if col >= available_width {
                                        break;
                                    }
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
                                    col += 1;
                                }
                                queue!(stdout, style::ResetColor)?;
                            } else {
                                let truncated =
                                    self.truncate_line(&displayed_line, available_width);
                                queue!(stdout, style::Print(truncated))?;
                            }
                        }
                    } else {
                        // 沒有選擇，如果有註解則部分變色
                        if let Some(comment_pos) = comment_start {
                            let chars: Vec<char> = displayed_line.chars().collect();
                            let mut col = 0;
                            #[allow(clippy::explicit_counter_loop)]
                            for (idx, &ch) in chars.iter().enumerate() {
                                if col >= available_width {
                                    break;
                                }
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
                                col += 1;
                            }
                            queue!(stdout, style::ResetColor)?;
                        } else {
                            let truncated = self.truncate_line(&displayed_line, available_width);
                            queue!(stdout, style::Print(truncated))?;
                        }
                    }
                }
            } else {
                // 空行顯示波浪號
                queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
                queue!(stdout, style::Print("~"))?;
                queue!(stdout, style::ResetColor)?;
            }

            // 清除行的剩餘部分
            queue!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
            )?;
        }

        // 渲染狀態欄
        self.render_status_bar(buffer, message, cursor)?;

        // 計算光標的視覺列位置（考慮 Tab 展開）
        let current_line = buffer
            .line(cursor.row)
            .map(|s| s.to_string())
            .unwrap_or_default();
        let visual_col = Self::calculate_visual_column(&current_line, cursor.col);

        let cursor_x = if self.show_line_numbers {
            (line_num_width + visual_col) as u16
        } else {
            visual_col as u16
        };
        let cursor_y = (cursor.row - self.offset_row) as u16;

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
