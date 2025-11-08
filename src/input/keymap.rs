use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::handler::{Command, Direction};

pub fn handle_key_event(event: KeyEvent, _has_selection: bool) -> Option<Command> {
    match (event.code, event.modifiers) {
        // 基本移動
        (KeyCode::Up, KeyModifiers::NONE) => Some(Command::MoveUp),
        (KeyCode::Down, KeyModifiers::NONE) => Some(Command::MoveDown),
        (KeyCode::Left, KeyModifiers::NONE) => Some(Command::MoveLeft),
        (KeyCode::Right, KeyModifiers::NONE) => Some(Command::MoveRight),
        (KeyCode::Home, KeyModifiers::NONE) => Some(Command::MoveHome),
        (KeyCode::End, KeyModifiers::NONE) => Some(Command::MoveEnd),
        (KeyCode::PageUp, KeyModifiers::NONE) => Some(Command::PageUp),
        (KeyCode::PageDown, KeyModifiers::NONE) => Some(Command::PageDown),

        // Ctrl 快速移動
        (KeyCode::Up, KeyModifiers::CONTROL) => Some(Command::MoveToFileStart),
        (KeyCode::Down, KeyModifiers::CONTROL) => Some(Command::MoveToFileEnd),
        (KeyCode::Left, KeyModifiers::CONTROL) => Some(Command::MoveToLineStart),
        (KeyCode::Right, KeyModifiers::CONTROL) => Some(Command::MoveToLineEnd),
        // 替代按鍵:Ctrl+Home/End
        (KeyCode::Home, KeyModifiers::CONTROL) => Some(Command::MoveToFileStart),
        (KeyCode::End, KeyModifiers::CONTROL) => Some(Command::MoveToFileEnd),

        // 選擇模式移動
        (KeyCode::Up, KeyModifiers::SHIFT) => Some(Command::ExtendSelection(Direction::Up)),
        (KeyCode::Down, KeyModifiers::SHIFT) => Some(Command::ExtendSelection(Direction::Down)),
        (KeyCode::Left, KeyModifiers::SHIFT) => Some(Command::ExtendSelection(Direction::Left)),
        (KeyCode::Right, KeyModifiers::SHIFT) => Some(Command::ExtendSelection(Direction::Right)),
        (KeyCode::Home, KeyModifiers::SHIFT) => Some(Command::ExtendSelection(Direction::Home)),
        (KeyCode::End, KeyModifiers::SHIFT) => Some(Command::ExtendSelection(Direction::End)),
        (KeyCode::PageUp, KeyModifiers::SHIFT) => Some(Command::ExtendSelection(Direction::PageUp)),
        (KeyCode::PageDown, KeyModifiers::SHIFT) => {
            Some(Command::ExtendSelection(Direction::PageDown))
        }

        // Ctrl+Shift 快速選擇
        (KeyCode::Left, m)
            if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
        {
            Some(Command::ExtendSelection(Direction::Home))
        }
        (KeyCode::Right, m)
            if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
        {
            Some(Command::ExtendSelection(Direction::End))
        }

        // 字符輸入
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            Some(Command::Insert(c))
        }
        (KeyCode::Enter, _) => Some(Command::Insert('\n')),
        (KeyCode::Tab, KeyModifiers::NONE) => Some(Command::Indent),
        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => Some(Command::Unindent),

        // 刪除操作
        (KeyCode::Backspace, _) => Some(Command::Backspace),
        (KeyCode::Delete, _) => Some(Command::Delete),

        // Ctrl 組合鍵
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(Command::Save),
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Command::Quit),
        (KeyCode::Char('z'), KeyModifiers::CONTROL) => Some(Command::Undo),
        (KeyCode::Char('y'), KeyModifiers::CONTROL) => Some(Command::Redo),
        (KeyCode::Char('f'), KeyModifiers::CONTROL) => Some(Command::Find),
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => Some(Command::ToggleLineNumbers),
        (KeyCode::Char('g'), KeyModifiers::CONTROL) => Some(Command::GoToLine),
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => Some(Command::SelectAll),
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => Some(Command::DeleteLine),
        (KeyCode::Char('\\'), KeyModifiers::CONTROL) => Some(Command::ToggleComment),
        (KeyCode::Char('/'), KeyModifiers::CONTROL) => Some(Command::ToggleComment),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(Command::ToggleComment),

        // 剪貼板操作
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Command::Copy),
        (KeyCode::Char('x'), KeyModifiers::CONTROL) => Some(Command::Cut),
        (KeyCode::Char('v'), KeyModifiers::CONTROL) => Some(Command::Paste),
        // F20 是 Paste 事件的標記（Windows Terminal 的 Ctrl+V）
        (KeyCode::F(20), KeyModifiers::NONE) => Some(Command::Paste),

        // ESC 清除選擇和訊息
        (KeyCode::Esc, _) => Some(Command::ClearMessage),

        // F3 搜索導航
        (KeyCode::F(3), KeyModifiers::NONE) => Some(Command::FindNext),
        (KeyCode::F(3), KeyModifiers::SHIFT) => Some(Command::FindPrev),

        _ => None,
    }
}
