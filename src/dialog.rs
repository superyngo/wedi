// 對話框模組 - 用於輸入框、確認框等

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::{self, Color},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

/// 顯示幫助對話框
#[allow(dead_code)]
pub fn show_help(terminal_size: (u16, u16)) -> Result<()> {
    let (cols, rows) = terminal_size;

    // 從 help 模組獲取幫助內容
    let mut help_lines =
        vec!["═══════════════════════ WEDI HELP ════════════════════════".to_string()];
    help_lines.push(String::new());
    help_lines.extend(crate::help::get_keyboard_shortcuts());
    help_lines.push(String::new());
    help_lines.push("Press ESC to close this help".to_string());
    help_lines.push("═══════════════════════════════════════════════════════════".to_string());

    let total_lines = help_lines.len();
    let max_display_lines = (rows.saturating_sub(3)) as usize;
    let mut scroll_offset = 0;

    loop {
        // 清除螢幕
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        // 計算可顯示的行數
        let visible_lines = total_lines.min(max_display_lines);
        let end_line = (scroll_offset + visible_lines).min(total_lines);

        // 顯示幫助內容
        for (i, line) in help_lines[scroll_offset..end_line].iter().enumerate() {
            queue!(
                io::stdout(),
                cursor::MoveTo(0, i as u16),
                style::SetForegroundColor(Color::Cyan),
            )?;

            // 截斷過長的行
            let display_line = if line.chars().count() > cols as usize {
                line.chars().take(cols as usize).collect::<String>()
            } else {
                line.to_string()
            };

            queue!(io::stdout(), style::Print(display_line))?;
        }

        queue!(io::stdout(), style::ResetColor)?;

        // 顯示滾動提示
        if total_lines > max_display_lines {
            let status_row = rows.saturating_sub(1);
            queue!(
                io::stdout(),
                cursor::MoveTo(0, status_row),
                style::SetBackgroundColor(Color::DarkBlue),
                style::SetForegroundColor(Color::White),
            )?;

            let scroll_info = format!(
                " Lines {}-{}/{} | Use ↑/↓ or PgUp/PgDn to scroll | ESC to close ",
                scroll_offset + 1,
                end_line,
                total_lines
            );

            let padded = if scroll_info.len() < cols as usize {
                format!(
                    "{}{}",
                    scroll_info,
                    " ".repeat(cols as usize - scroll_info.len())
                )
            } else {
                scroll_info.chars().take(cols as usize).collect()
            };

            queue!(io::stdout(), style::Print(padded), style::ResetColor)?;
        }

        io::stdout().flush()?;

        // 處理按鍵
        loop {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                match key_event.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Up => {
                        scroll_offset = scroll_offset.saturating_sub(1);
                        break;
                    }
                    KeyCode::Down => {
                        if scroll_offset + max_display_lines < total_lines {
                            scroll_offset += 1;
                        }
                        break;
                    }
                    KeyCode::PageUp => {
                        scroll_offset = scroll_offset.saturating_sub(max_display_lines / 2);
                        break;
                    }
                    KeyCode::PageDown => {
                        scroll_offset = (scroll_offset + max_display_lines / 2)
                            .min(total_lines.saturating_sub(max_display_lines));
                        break;
                    }
                    KeyCode::Home => {
                        scroll_offset = 0;
                        break;
                    }
                    KeyCode::End => {
                        scroll_offset = total_lines.saturating_sub(max_display_lines);
                        break;
                    }
                    _ => break,
                }
            }
        }
    }
}

/// 顯示輸入對話框並獲取用戶輸入
#[allow(dead_code)]
pub fn prompt(prompt_text: &str, terminal_size: (u16, u16)) -> Result<Option<String>> {
    prompt_with_default(prompt_text, "", terminal_size)
}

/// 顯示輸入對話框並獲取用戶輸入，支持預設值
#[allow(dead_code)]
pub fn prompt_with_default(
    prompt_text: &str,
    default: &str,
    terminal_size: (u16, u16),
) -> Result<Option<String>> {
    let mut input = default.to_string();
    let mut cursor_pos = input.chars().count(); // 光標位置（字符索引）
    let (cols, rows) = terminal_size;
    let dialog_row = rows.saturating_sub(2);

    loop {
        // 清除對話框行
        execute!(
            io::stdout(),
            cursor::MoveTo(0, dialog_row),
            terminal::Clear(ClearType::CurrentLine)
        )?;

        // 顯示提示和當前輸入
        queue!(
            io::stdout(),
            style::SetBackgroundColor(Color::DarkBlue),
            style::SetForegroundColor(Color::White),
            cursor::MoveTo(0, dialog_row),
        )?;

        let display = format!(" {} {}", prompt_text, input);
        let display = if display.len() > cols as usize {
            &display[..cols as usize]
        } else {
            &display
        };

        queue!(io::stdout(), style::Print(display))?;

        // 填滿剩餘空間
        let remaining = cols as usize - display.len();
        if remaining > 0 {
            queue!(io::stdout(), style::Print(" ".repeat(remaining)))?;
        }

        queue!(io::stdout(), style::ResetColor)?;

        // 設置光標位置（基於字符位置，不是字節位置）
        let cursor_x = (prompt_text.len() + 2 + cursor_pos).min(cols as usize - 1) as u16;
        execute!(io::stdout(), cursor::MoveTo(cursor_x, dialog_row))?;
        execute!(io::stdout(), cursor::Show)?;

        io::stdout().flush()?;

        // 讀取按鍵,只處理 Press 和 Repeat 事件
        loop {
            if let Event::Key(key_event) = event::read()? {
                // 忽略 Release 事件,避免重複輸入
                if key_event.kind != KeyEventKind::Press && key_event.kind != KeyEventKind::Repeat {
                    continue;
                }

                match key_event.code {
                    KeyCode::Enter => {
                        // 確認輸入
                        return Ok(Some(input));
                    }
                    KeyCode::Esc => {
                        // 取消
                        return Ok(None);
                    }
                    KeyCode::Char(c) => {
                        // 在光標位置插入字符
                        let byte_pos = input.chars().take(cursor_pos).collect::<String>().len();
                        input.insert(byte_pos, c);
                        cursor_pos += 1;
                        break;
                    }
                    KeyCode::Backspace => {
                        // 刪除光標前的字符
                        if cursor_pos > 0 {
                            let byte_pos =
                                input.chars().take(cursor_pos - 1).collect::<String>().len();
                            input.remove(byte_pos);
                            cursor_pos -= 1;
                        }
                        break;
                    }
                    KeyCode::Delete => {
                        // 刪除光標後的字符
                        if cursor_pos < input.chars().count() {
                            let byte_pos = input.chars().take(cursor_pos).collect::<String>().len();
                            input.remove(byte_pos);
                        }
                        break;
                    }
                    KeyCode::Left => {
                        // 向左移動光標
                        cursor_pos = cursor_pos.saturating_sub(1);
                        break;
                    }
                    KeyCode::Right => {
                        // 向右移動光標
                        if cursor_pos < input.chars().count() {
                            cursor_pos += 1;
                        }
                        break;
                    }
                    KeyCode::Home => {
                        // 移動到開頭
                        cursor_pos = 0;
                        break;
                    }
                    KeyCode::End => {
                        // 移動到結尾
                        cursor_pos = input.chars().count();
                        break;
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }
}

/// 顯示確認對話框
#[allow(dead_code)]
pub fn confirm(message: &str, terminal_size: (u16, u16)) -> Result<bool> {
    let (cols, rows) = terminal_size;
    let dialog_row = rows.saturating_sub(2);

    loop {
        // 清除對話框行
        execute!(
            io::stdout(),
            cursor::MoveTo(0, dialog_row),
            terminal::Clear(ClearType::CurrentLine)
        )?;

        // 顯示消息
        queue!(
            io::stdout(),
            style::SetBackgroundColor(Color::DarkYellow),
            style::SetForegroundColor(Color::Black),
            cursor::MoveTo(0, dialog_row),
        )?;

        let display = format!(" {} (y/n)", message);
        let display = if display.len() > cols as usize {
            &display[..cols as usize]
        } else {
            &display
        };

        queue!(io::stdout(), style::Print(display))?;

        // 填滿剩餘空間
        let remaining = cols as usize - display.len();
        if remaining > 0 {
            queue!(io::stdout(), style::Print(" ".repeat(remaining)))?;
        }

        queue!(io::stdout(), style::ResetColor)?;
        io::stdout().flush()?;

        // 讀取按鍵,只處理 Press 事件
        loop {
            if let Event::Key(key_event) = event::read()? {
                // 忽略 Release 事件
                if key_event.kind != KeyEventKind::Press && key_event.kind != KeyEventKind::Repeat {
                    continue;
                }

                match key_event.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                    _ => {
                        break;
                    }
                }
            }
        }
    }
}
