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

/// 顯示輸入對話框並獲取用戶輸入
#[allow(dead_code)]
pub fn prompt(prompt_text: &str, terminal_size: (u16, u16)) -> Result<Option<String>> {
    prompt_with_default(prompt_text, "", terminal_size)
}

/// 顯示輸入對話框並獲取用戶輸入，支持預設值
#[allow(dead_code)]
pub fn prompt_with_default(prompt_text: &str, default: &str, terminal_size: (u16, u16)) -> Result<Option<String>> {
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
                            let byte_pos = input.chars().take(cursor_pos - 1).collect::<String>().len();
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
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                        }
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
