use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, ClearType},
};

use std::io;

/// 輸入事件類型：鍵盤事件或貼上事件
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// 鍵盤按鍵事件
    Key(KeyEvent),
    /// 貼上事件（包含已正規化的文字）
    Paste(String),
    /// 終端視窗大小改變
    Resize,
}

pub struct Terminal {
    size: (u16, u16),
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let size = terminal::size()?;
        Ok(Self { size })
    }

    /// Create a terminal handle with a fixed size, without querying the tty
    /// (for headless use such as tests).
    pub fn with_size(size: (u16, u16)) -> Self {
        Self { size }
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

    pub fn update_size(&mut self) -> Result<()> {
        self.size = terminal::size()?;
        Ok(())
    }

    /// 讀取輸入事件（支援 Bracketed Paste）
    ///
    /// 返回 `InputEvent`，區分鍵盤事件和貼上事件，並在貼上事件中攜帶完整文字。
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
                Event::Resize(_cols, _rows) => return Ok(InputEvent::Resize),
                Event::Paste(text) => {
                    // Bracketed Paste 事件
                    // 正規化行尾符號：\r\n 和 \r 都轉換為 \n
                    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                    return Ok(InputEvent::Paste(normalized));
                }
                _ => {
                    // 忽略其他事件
                }
            }
        }
    }

    pub fn show_cursor() -> Result<()> {
        execute!(io::stdout(), cursor::Show)?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = Self::exit_raw_mode();
        let _ = Self::show_cursor();
    }
}
