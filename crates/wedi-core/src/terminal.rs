use anyhow::Result;
use crossterm::{
    cursor,
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{self, ClearType},
};

#[cfg(feature = "mouse-support")]
use crossterm::event::{MouseEvent, MouseEventKind};

use std::io::{self, Write};

/// 輸入事件類型：鍵盤事件或貼上事件
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// 鍵盤按鍵事件
    Key(KeyEvent),
    /// 貼上事件（包含已正規化的文字）
    Paste(String),
}

pub struct Terminal {
    size: (u16, u16),
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let size = terminal::size()?;
        Ok(Self { size })
    }

    pub fn enter_raw_mode() -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            io::stdout(),
            terminal::EnterAlternateScreen,
            EnableBracketedPaste
        )?;
        Ok(())
    }

    pub fn exit_raw_mode() -> Result<()> {
        execute!(
            io::stdout(),
            DisableBracketedPaste,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn clear_screen() -> Result<()> {
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;
        Ok(())
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    #[allow(dead_code)]
    pub fn update_size(&mut self) -> Result<()> {
        self.size = terminal::size()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn flush() -> Result<()> {
        io::stdout().flush()?;
        Ok(())
    }

    pub fn read_key() -> Result<KeyEvent> {
        loop {
            let event = event::read()?;

            match event {
                Event::Key(key_event) => {
                    // 處理正常的 Press 和 Repeat 事件
                    if key_event.kind == KeyEventKind::Press
                        || key_event.kind == KeyEventKind::Repeat
                    {
                        return Ok(key_event);
                    }
                }
                Event::Resize(_cols, _rows) => {
                    // 視窗大小改變,返回特殊標記
                    return Ok(KeyEvent::new(KeyCode::F(21), KeyModifiers::NONE));
                }
                Event::Paste(_text) => {
                    // Bracketed Paste 事件
                    // 返回一個特殊按鍵標記（使用 read_input 可獲取完整文字）
                    return Ok(KeyEvent::new(KeyCode::F(20), KeyModifiers::NONE));
                }
                #[cfg(feature = "mouse-support")]
                Event::Mouse(mouse_event) => {
                    // 滑鼠滾輪事件轉換為虛擬按鍵
                    if let Some(key_event) = handle_mouse_event(mouse_event) {
                        return Ok(key_event);
                    }
                }
                _ => {
                    // 忽略其他事件（鼠標、調整大小等）
                }
            }
        }
    }

    /// 讀取輸入事件（支援 Bracketed Paste）
    ///
    /// 與 `read_key()` 不同，此方法返回 `InputEvent` 枚舉，
    /// 可以區分鍵盤事件和貼上事件，並在貼上事件中攜帶完整文字。
    pub fn read_input() -> Result<InputEvent> {
        loop {
            let event = event::read()?;

            match event {
                Event::Key(key_event) => {
                    // 處理正常的 Press 和 Repeat 事件
                    if key_event.kind == KeyEventKind::Press
                        || key_event.kind == KeyEventKind::Repeat
                    {
                        return Ok(InputEvent::Key(key_event));
                    }
                }
                Event::Resize(_cols, _rows) => {
                    // 視窗大小改變,返回特殊標記
                    return Ok(InputEvent::Key(KeyEvent::new(
                        KeyCode::F(21),
                        KeyModifiers::NONE,
                    )));
                }
                Event::Paste(text) => {
                    // Bracketed Paste 事件
                    // 正規化行尾符號：\r\n 和 \r 都轉換為 \n
                    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                    return Ok(InputEvent::Paste(normalized));
                }
                #[cfg(feature = "mouse-support")]
                Event::Mouse(mouse_event) => {
                    // 滑鼠滾輪事件轉換為虛擬按鍵
                    if let Some(key_event) = handle_mouse_event(mouse_event) {
                        return Ok(InputEvent::Key(key_event));
                    }
                }
                _ => {
                    // 忽略其他事件
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_cursor_position(x: u16, y: u16) -> Result<()> {
        execute!(io::stdout(), cursor::MoveTo(x, y))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn hide_cursor() -> Result<()> {
        execute!(io::stdout(), cursor::Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<()> {
        execute!(io::stdout(), cursor::Show)?;
        Ok(())
    }
}

#[cfg(feature = "mouse-support")]
fn handle_mouse_event(mouse_event: MouseEvent) -> Option<KeyEvent> {
    match mouse_event.kind {
        MouseEventKind::ScrollUp => Some(KeyEvent::new(KeyCode::F(22), KeyModifiers::NONE)),
        MouseEventKind::ScrollDown => Some(KeyEvent::new(KeyCode::F(23), KeyModifiers::NONE)),
        _ => None,
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = Self::exit_raw_mode();
        let _ = Self::show_cursor();
    }
}
