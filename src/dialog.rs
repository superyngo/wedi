// 對話框模組 - 用於輸入框、確認框等

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::{self, Color},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

/// 顯示輸入對話框並獲取用戶輸入
pub fn prompt(prompt_text: &str, terminal_size: (u16, u16)) -> Result<Option<String>> {
    let mut input = String::new();
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

        // 設置光標位置
        let cursor_x = (prompt_text.len() + 2 + input.len()).min(cols as usize - 1) as u16;
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
                        // 添加字符
                        input.push(c);
                        break;
                    }
                    KeyCode::Backspace => {
                        // 刪除字符
                        input.pop();
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
