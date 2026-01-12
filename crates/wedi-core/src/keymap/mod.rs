mod bindings;
pub mod command;

pub use bindings::handle_key_event;
pub use command::{Command, Direction};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// 可自訂的快捷鍵對映
#[derive(Default)]
pub struct Keymap {
    bindings: HashMap<(KeyCode, KeyModifiers), Command>,
    selection_overrides: HashMap<(KeyCode, KeyModifiers), Command>,
}

impl Keymap {
    /// 綁定快捷鍵到命令
    pub fn bind(&mut self, key: KeyCode, modifiers: KeyModifiers, command: Command) -> &mut Self {
        self.bindings.insert((key, modifiers), command);
        self
    }

    /// 解除快捷鍵綁定
    pub fn unbind(&mut self, key: KeyCode, modifiers: KeyModifiers) -> &mut Self {
        self.bindings.remove(&(key, modifiers));
        self
    }

    /// 在選擇模式下綁定快捷鍵
    pub fn bind_selection(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        command: Command,
    ) -> &mut Self {
        self.selection_overrides.insert((key, modifiers), command);
        self
    }

    /// 查詢快捷鍵對應的命令
    pub fn get_command(&self, event: KeyEvent, selection_mode: bool) -> Option<Command> {
        // 如果是選擇模式，先查詢選擇模式覆蓋
        if selection_mode {
            if let Some(cmd) = self.selection_overrides.get(&(event.code, event.modifiers)) {
                return Some(cmd.clone());
            }
        }

        // 查詢普通綁定
        if let Some(cmd) = self.bindings.get(&(event.code, event.modifiers)) {
            return Some(cmd.clone());
        }

        // 回退到原有的硬編碼邏輯（向後相容）
        handle_key_event(event, selection_mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_keymap() {
        let mut keymap = Keymap::default();
        keymap.bind(KeyCode::Char('s'), KeyModifiers::CONTROL, Command::Save);

        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(keymap.get_command(event, false), Some(Command::Save));
    }

    #[test]
    fn test_unbind() {
        let mut keymap = Keymap::default();
        keymap.bind(KeyCode::Char('q'), KeyModifiers::CONTROL, Command::Quit);
        keymap.unbind(KeyCode::Char('q'), KeyModifiers::CONTROL);

        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        // 應該回退到原有綁定
        assert_eq!(keymap.get_command(event, false), Some(Command::Quit));
    }
}
