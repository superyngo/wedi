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
use std::sync::atomic::{AtomicBool, Ordering};
use wedi_core::utils::{display_width, visual_width};

// 對話框期間發生過視窗大小改變（事件已被對話框取走，編輯器需自行同步尺寸）
static RESIZED: AtomicBool = AtomicBool::new(false);

/// Returns whether the terminal was resized while a dialog was open, and clears the flag.
pub fn take_resized() -> bool {
    RESIZED.swap(false, Ordering::Relaxed)
}

/// 對話框收到的輸入
enum DialogEvent {
    Key(KeyEvent),
    Paste(String),
    // 視窗大小改變：尺寸已更新，呼叫端清除舊位置後重繪
    Resize,
}

/// 讀取下一個對話框事件；只回傳 Press / Repeat 按鍵，忽略 Release 以免重複輸入
fn next_event(size: &mut (u16, u16)) -> Result<DialogEvent> {
    loop {
        match event::read()? {
            Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                return Ok(DialogEvent::Key(key))
            }
            Event::Paste(text) => return Ok(DialogEvent::Paste(text)),
            Event::Resize(w, h) => {
                *size = (w, h);
                RESIZED.store(true, Ordering::Relaxed);
                return Ok(DialogEvent::Resize);
            }
            _ => {}
        }
    }
}

/// 單行輸入框按鍵的結果
#[derive(Debug, PartialEq)]
enum PromptAction {
    Continue,
    Submit(String),
    Cancel,
}

/// 單行輸入框的狀態（不含 I/O，可單元測試）；cursor 為字元索引
struct LineInput {
    text: String,
    cursor: usize,
}

impl LineInput {
    fn new(default: &str) -> Self {
        Self {
            text: default.to_string(),
            cursor: default.chars().count(),
        }
    }

    fn byte_at(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map_or(self.text.len(), |(b, _)| b)
    }

    fn handle_key(&mut self, key: KeyEvent) -> PromptAction {
        match key.code {
            KeyCode::Enter => return PromptAction::Submit(self.text.clone()),
            KeyCode::Esc => return PromptAction::Cancel,
            KeyCode::Char(c) => {
                let b = self.byte_at(self.cursor);
                self.text.insert(b, c);
                self.cursor += 1;
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                let b = self.byte_at(self.cursor);
                self.text.remove(b);
            }
            KeyCode::Delete if self.cursor < self.text.chars().count() => {
                let b = self.byte_at(self.cursor);
                self.text.remove(b);
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.text.chars().count()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.text.chars().count(),
            _ => {}
        }
        PromptAction::Continue
    }

    /// 單行輸入框：只取貼上內容的第一行（終端貼上的換行常為 \r，與 \n 一併視為行尾）
    fn paste(&mut self, pasted: &str) {
        let line = pasted.split(['\r', '\n']).next().unwrap_or("");
        let b = self.byte_at(self.cursor);
        self.text.insert_str(b, line);
        self.cursor += line.chars().count();
    }

    /// 游標前文字的視覺寬度（CJK 雙寬）
    fn cursor_width(&self) -> usize {
        visual_width(&self.text[..self.byte_at(self.cursor)])
    }
}

/// 確認框按鍵：Y/Enter = 是（預設，以大寫 Y 標示），N/ESC = 否，其他忽略
fn confirm_answer(key: KeyEvent) -> Option<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(true),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(false),
        _ => None,
    }
}

/// 幫助/關於面板的頁籤與捲動狀態（不含 I/O，可單元測試）
struct HelpPanelState {
    tab: usize,
    scroll: usize,
}

impl HelpPanelState {
    const TABS: usize = 2;

    /// 處理按鍵；回傳 false 表示關閉面板
    fn handle_key(&mut self, key: KeyEvent, visible: usize, total: usize) -> bool {
        let max_scroll = total.saturating_sub(visible);
        match key.code {
            KeyCode::Esc => return false,
            KeyCode::Tab | KeyCode::Right => {
                self.tab = (self.tab + 1) % Self::TABS;
                self.scroll = 0;
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.tab = (self.tab + Self::TABS - 1) % Self::TABS;
                self.scroll = 0;
            }
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = (self.scroll + 1).min(max_scroll),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(visible / 2),
            KeyCode::PageDown => self.scroll = (self.scroll + visible / 2).min(max_scroll),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = max_scroll,
            _ => {}
        }
        true
    }
}

/// 依視覺寬度截斷字串，回傳（截斷後字串, 實際視覺寬度）
/// 以字元為單位處理，避免 byte 切割造成 panic，並正確計算 CJK 雙寬字元
fn truncate_to_width(s: &str, max_width: usize) -> (String, usize) {
    let mut result = String::new();
    let mut width = 0;
    for ch in s.chars() {
        let w = display_width(ch);
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
pub fn show_help(terminal_size: (u16, u16)) -> Result<()> {
    let mut size = terminal_size;
    let tabs = ["Help", "About"];
    let pages = [build_help_panel_lines(), build_about_panel_lines()];
    let mut state = HelpPanelState { tab: 0, scroll: 0 };

    loop {
        let (cols, rows) = size;
        // 頁籤列 + 分隔線 + 底部狀態列
        let max_display_lines = (rows.saturating_sub(3)) as usize;
        let lines = &pages[state.tab];
        let total_lines = lines.len();
        state.scroll = state
            .scroll
            .min(total_lines.saturating_sub(max_display_lines));
        let scroll_offset = state.scroll;

        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        // 頁籤列
        queue!(io::stdout(), cursor::MoveTo(0, 0))?;
        for (i, tab) in tabs.iter().enumerate() {
            if i == state.tab {
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

        // 處理按鍵；視窗大小改變時重繪
        loop {
            match next_event(&mut size)? {
                DialogEvent::Key(key) => {
                    if !state.handle_key(key, max_display_lines, total_lines) {
                        return Ok(());
                    }
                    break;
                }
                DialogEvent::Resize => break,
                DialogEvent::Paste(_) => {}
            }
        }
    }
}

/// 顯示輸入對話框並獲取用戶輸入
pub fn prompt(prompt_text: &str, terminal_size: (u16, u16)) -> Result<Option<String>> {
    prompt_with_default(prompt_text, "", terminal_size)
}

/// 顯示輸入對話框並獲取用戶輸入，支持預設值
pub fn prompt_with_default(
    prompt_text: &str,
    default: &str,
    terminal_size: (u16, u16),
) -> Result<Option<String>> {
    let mut input = LineInput::new(default);
    let mut size = terminal_size;

    loop {
        let (cols, rows) = size;
        let dialog_row = rows.saturating_sub(2);
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

        let display = format!(" {} {}", prompt_text, input.text);
        let (display, display_width) = truncate_to_width(&display, cols as usize);

        queue!(io::stdout(), style::Print(&display))?;

        // 填滿剩餘空間
        let remaining = (cols as usize).saturating_sub(display_width);
        if remaining > 0 {
            queue!(io::stdout(), style::Print(" ".repeat(remaining)))?;
        }

        queue!(io::stdout(), style::ResetColor)?;

        // 設置光標位置（以視覺寬度計算，正確處理 CJK 雙寬字元）
        let cursor_x = (visual_width(prompt_text) + 2 + input.cursor_width())
            .min((cols as usize).saturating_sub(1)) as u16;
        execute!(io::stdout(), cursor::MoveTo(cursor_x, dialog_row))?;
        execute!(io::stdout(), cursor::Show)?;

        io::stdout().flush()?;

        match next_event(&mut size)? {
            DialogEvent::Key(key) => match input.handle_key(key) {
                PromptAction::Submit(text) => return Ok(Some(text)),
                PromptAction::Cancel => return Ok(None),
                PromptAction::Continue => {}
            },
            DialogEvent::Paste(text) => input.paste(&text),
            DialogEvent::Resize => {
                // 清除舊位置的對話框，再以新尺寸重繪
                execute!(
                    io::stdout(),
                    cursor::MoveTo(0, dialog_row),
                    terminal::Clear(ClearType::FromCursorDown)
                )?;
            }
        }
    }
}

/// 顯示確認對話框
pub fn confirm(message: &str, terminal_size: (u16, u16)) -> Result<bool> {
    let mut size = terminal_size;

    loop {
        let (cols, rows) = size;
        let dialog_row = rows.saturating_sub(2);
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

        // 讀取按鍵；其他按鍵忽略，視窗大小改變時清除舊位置後重繪
        loop {
            match next_event(&mut size)? {
                DialogEvent::Key(key) => {
                    if let Some(answer) = confirm_answer(key) {
                        return Ok(answer);
                    }
                }
                DialogEvent::Resize => {
                    execute!(
                        io::stdout(),
                        cursor::MoveTo(0, dialog_row),
                        terminal::Clear(ClearType::FromCursorDown)
                    )?;
                    break;
                }
                DialogEvent::Paste(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

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

    fn k(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_str(input: &mut LineInput, s: &str) {
        for c in s.chars() {
            input.handle_key(k(KeyCode::Char(c)));
        }
    }

    #[test]
    fn test_line_input_cjk_edit_and_cursor_width() {
        // 稽核 B13/B17：CJK 輸入的插入、退格與游標寬度
        let mut input = LineInput::new("");
        type_str(&mut input, "中文ab");
        assert_eq!(input.cursor_width(), 6);
        input.handle_key(k(KeyCode::Left));
        input.handle_key(k(KeyCode::Left));
        input.handle_key(k(KeyCode::Backspace));
        assert_eq!(input.text, "中ab");
        assert_eq!(input.cursor_width(), 2);
        type_str(&mut input, "字");
        assert_eq!(input.text, "中字ab");
    }

    #[test]
    fn test_line_input_delete_and_bounds() {
        let mut input = LineInput::new("ab");
        // 游標在結尾：Delete 與 Right 不動
        input.handle_key(k(KeyCode::Delete));
        input.handle_key(k(KeyCode::Right));
        assert_eq!((input.text.as_str(), input.cursor), ("ab", 2));
        input.handle_key(k(KeyCode::Home));
        input.handle_key(k(KeyCode::Backspace));
        input.handle_key(k(KeyCode::Delete));
        assert_eq!((input.text.as_str(), input.cursor), ("b", 0));
        input.handle_key(k(KeyCode::End));
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn test_line_input_default_submit_and_cancel() {
        let mut input = LineInput::new("abc");
        type_str(&mut input, "d");
        assert_eq!(
            input.handle_key(k(KeyCode::Enter)),
            PromptAction::Submit("abcd".to_string())
        );
        assert_eq!(input.handle_key(k(KeyCode::Esc)), PromptAction::Cancel);
    }

    #[test]
    fn test_line_input_paste_keeps_first_line() {
        let mut input = LineInput::new("[]");
        input.handle_key(k(KeyCode::Left));
        input.paste("中x\nsecond");
        assert_eq!(input.text, "[中x]");
        assert_eq!(input.cursor, 3);
        // 終端（含 tmux）貼上時換行送成 \r，不可把 \r 插入輸入框
        input.paste("y\rz");
        assert_eq!(input.text, "[中xy]");
    }

    #[test]
    fn test_confirm_answer_keys() {
        for (code, want) in [
            (KeyCode::Char('y'), Some(true)),
            (KeyCode::Char('Y'), Some(true)),
            (KeyCode::Enter, Some(true)),
            (KeyCode::Char('n'), Some(false)),
            (KeyCode::Char('N'), Some(false)),
            (KeyCode::Esc, Some(false)),
            (KeyCode::Char('x'), None),
        ] {
            assert_eq!(confirm_answer(k(code)), want, "{code:?}");
        }
    }

    #[test]
    fn test_help_panel_tabs_and_scroll_bounds() {
        let mut state = HelpPanelState { tab: 0, scroll: 0 };
        // 10 行可見、共 25 行：最多捲到 15
        state.handle_key(k(KeyCode::End), 10, 25);
        assert_eq!(state.scroll, 15);
        state.handle_key(k(KeyCode::Down), 10, 25);
        state.handle_key(k(KeyCode::PageDown), 10, 25);
        assert_eq!(state.scroll, 15);
        state.handle_key(k(KeyCode::PageUp), 10, 25);
        assert_eq!(state.scroll, 10);
        // 切換頁籤回到頂端，左右循環
        state.handle_key(k(KeyCode::Left), 10, 25);
        assert_eq!((state.tab, state.scroll), (1, 0));
        state.handle_key(k(KeyCode::Tab), 10, 25);
        assert_eq!(state.tab, 0);
        // 內容比視窗短時不捲動
        state.handle_key(k(KeyCode::Down), 10, 5);
        assert_eq!(state.scroll, 0);
        assert!(!state.handle_key(k(KeyCode::Esc), 10, 5));
    }
}
