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
use wedi_core::utils::visual_width;

/// 依視覺寬度截斷字串，回傳（截斷後字串, 實際視覺寬度）
/// 以字元為單位處理，避免 byte 切割造成 panic，並正確計算 CJK 雙寬字元
fn truncate_to_width(s: &str, max_width: usize) -> (String, usize) {
    let mut result = String::new();
    let mut width = 0;
    for ch in s.chars() {
        let w = visual_width(ch.encode_utf8(&mut [0u8; 4]));
        if width + w > max_width {
            break;
        }
        result.push(ch);
        width += w;
    }
    (result, width)
}

/// 幫助/關於面板的一行內容
enum PanelLine {
    Blank,
    /// 區段標題（粗體青色）
    Section(String),
    /// 按鍵/標籤（黃色）+ 說明（預設色），key 已含對齊用空白
    Item {
        key: String,
        desc: String,
    },
    /// 純文字行
    Text(String),
}

/// 將結構化幫助資料轉為面板行（key 欄位對齊）
fn build_help_panel_lines() -> Vec<PanelLine> {
    let sections = crate::help::get_help_sections();
    let key_width = sections
        .iter()
        .flat_map(|(_, items)| items.iter())
        .map(|(key, _)| key.chars().count())
        .max()
        .unwrap_or(0);

    let mut lines = Vec::new();
    for (i, (title, items)) in sections.iter().enumerate() {
        if i > 0 {
            lines.push(PanelLine::Blank);
        }
        lines.push(PanelLine::Section(title.to_string()));
        for (key, desc) in items {
            lines.push(PanelLine::Item {
                key: format!("  {:<width$}  ", key, width = key_width),
                desc: desc.to_string(),
            });
        }
    }
    lines
}

/// 將 About 資料轉為面板行
fn build_about_panel_lines() -> Vec<PanelLine> {
    let entries = crate::help::get_about_entries();
    let label_width = entries
        .iter()
        .filter(|(label, _)| !label.is_empty())
        .map(|(label, _)| label.chars().count())
        .max()
        .unwrap_or(0);

    let mut lines = Vec::new();
    for (i, (label, content)) in entries.iter().enumerate() {
        if label.is_empty() {
            if content.is_empty() {
                lines.push(PanelLine::Blank);
            } else if i == 0 || *content == "Privacy" {
                // 首行（名稱+版本）與 Privacy 標題以區段樣式呈現
                lines.push(PanelLine::Section(content.clone()));
            } else {
                lines.push(PanelLine::Text(content.clone()));
            }
        } else {
            lines.push(PanelLine::Item {
                key: format!("  {:<width$}  ", label, width = label_width),
                desc: content.clone(),
            });
        }
    }
    lines
}

/// 顯示幫助/關於面板（Tab 或 ←/→ 切換頁籤，ESC 關閉）
#[allow(dead_code)]
pub fn show_help(terminal_size: (u16, u16)) -> Result<()> {
    let (cols, rows) = terminal_size;
    let tabs = ["Help", "About"];
    let pages = [build_help_panel_lines(), build_about_panel_lines()];
    let mut active_tab = 0usize;
    // 頁籤列 + 分隔線 + 底部狀態列
    let max_display_lines = (rows.saturating_sub(3)) as usize;
    let mut scroll_offset = 0usize;

    loop {
        let lines = &pages[active_tab];
        let total_lines = lines.len();
        scroll_offset = scroll_offset.min(total_lines.saturating_sub(max_display_lines));

        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        // 頁籤列
        queue!(io::stdout(), cursor::MoveTo(0, 0))?;
        for (i, tab) in tabs.iter().enumerate() {
            if i == active_tab {
                queue!(
                    io::stdout(),
                    style::SetBackgroundColor(Color::Cyan),
                    style::SetForegroundColor(Color::Black),
                    style::Print(format!("  {}  ", tab)),
                    style::ResetColor,
                )?;
            } else {
                queue!(
                    io::stdout(),
                    style::SetForegroundColor(Color::DarkGrey),
                    style::Print(format!("  {}  ", tab)),
                    style::ResetColor,
                )?;
            }
        }

        // 分隔線
        queue!(
            io::stdout(),
            cursor::MoveTo(0, 1),
            style::SetForegroundColor(Color::DarkGrey),
            style::Print("─".repeat(cols as usize)),
            style::ResetColor,
        )?;

        // 內容
        let end_line = (scroll_offset + max_display_lines).min(total_lines);
        for (i, line) in lines[scroll_offset..end_line].iter().enumerate() {
            queue!(io::stdout(), cursor::MoveTo(0, (i + 2) as u16))?;
            match line {
                PanelLine::Blank => {}
                PanelLine::Section(title) => {
                    let (text, _) = truncate_to_width(title, cols as usize);
                    queue!(
                        io::stdout(),
                        style::SetForegroundColor(Color::Cyan),
                        style::SetAttribute(style::Attribute::Bold),
                        style::Print(text),
                        style::SetAttribute(style::Attribute::Reset),
                        style::ResetColor,
                    )?;
                }
                PanelLine::Item { key, desc } => {
                    let (key_text, key_visual) = truncate_to_width(key, cols as usize);
                    let (desc_text, _) =
                        truncate_to_width(desc, (cols as usize).saturating_sub(key_visual));
                    queue!(
                        io::stdout(),
                        style::SetForegroundColor(Color::Yellow),
                        style::Print(key_text),
                        style::ResetColor,
                        style::Print(desc_text),
                    )?;
                }
                PanelLine::Text(text) => {
                    let (text, _) = truncate_to_width(text, cols as usize);
                    queue!(io::stdout(), style::Print(text))?;
                }
            }
        }

        // 底部狀態列
        let status_row = rows.saturating_sub(1);
        let scroll_info = if total_lines > max_display_lines {
            format!(
                " {}-{}/{} | ↑/↓ PgUp/PgDn scroll | Tab/←/→ switch tab | ESC close ",
                scroll_offset + 1,
                end_line,
                total_lines
            )
        } else {
            " Tab/←/→ switch tab | ESC close ".to_string()
        };
        let (status_text, status_width) = truncate_to_width(&scroll_info, cols as usize);
        queue!(
            io::stdout(),
            cursor::MoveTo(0, status_row),
            style::SetBackgroundColor(Color::DarkBlue),
            style::SetForegroundColor(Color::White),
            style::Print(&status_text),
            style::Print(" ".repeat((cols as usize).saturating_sub(status_width))),
            style::ResetColor,
        )?;

        io::stdout().flush()?;

        // 處理按鍵
        loop {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                match key_event.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Tab | KeyCode::Right => {
                        active_tab = (active_tab + 1) % tabs.len();
                        scroll_offset = 0;
                        break;
                    }
                    KeyCode::BackTab | KeyCode::Left => {
                        active_tab = (active_tab + tabs.len() - 1) % tabs.len();
                        scroll_offset = 0;
                        break;
                    }
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
        let (display, display_width) = truncate_to_width(&display, cols as usize);

        queue!(io::stdout(), style::Print(&display))?;

        // 填滿剩餘空間
        let remaining = (cols as usize).saturating_sub(display_width);
        if remaining > 0 {
            queue!(io::stdout(), style::Print(" ".repeat(remaining)))?;
        }

        queue!(io::stdout(), style::ResetColor)?;

        // 設置光標位置（以視覺寬度計算，正確處理 CJK 雙寬字元）
        let input_before_cursor: String = input.chars().take(cursor_pos).collect();
        let cursor_x = (visual_width(prompt_text) + 2 + visual_width(&input_before_cursor))
            .min(cols as usize - 1) as u16;
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

        let display = format!(" {} (Y/n)", message);
        let (display, display_width) = truncate_to_width(&display, cols as usize);

        queue!(io::stdout(), style::Print(&display))?;

        // 填滿剩餘空間
        let remaining = (cols as usize).saturating_sub(display_width);
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
                    // Enter = 確認（預設為 yes，以大寫 Y 標示）
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                    _ => {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::truncate_to_width;

    #[test]
    fn truncate_cjk_at_boundary_does_not_panic() {
        // 舊實作以 byte 索引切割，在 CJK 字元中間切割會 panic
        let s = " Search: 中文搜尋關鍵字測試";
        let (out, w) = truncate_to_width(s, 12);
        assert!(w <= 12);
        assert!(s.starts_with(&out));
    }

    #[test]
    fn truncate_counts_cjk_double_width() {
        let (out, w) = truncate_to_width("中文abc", 5);
        assert_eq!(out, "中文a"); // 2+2+1 = 5
        assert_eq!(w, 5);
    }

    #[test]
    fn truncate_shorter_than_limit_unchanged() {
        let (out, w) = truncate_to_width("abc", 10);
        assert_eq!(out, "abc");
        assert_eq!(w, 3);
    }
}
