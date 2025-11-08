use anyhow::Result;
use arboard::Clipboard;

pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        // 嘗試初始化剪貼簿,如果失敗(如無圖形界面)則設為 None
        let clipboard = Clipboard::new().ok();
        if clipboard.is_none() {
            eprintln!("Warning: Clipboard not available (no GUI detected). Copy/Paste disabled.");
        }
        Ok(Self { clipboard })
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard {
            clipboard.set_text(text)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Clipboard not available"))
        }
    }

    pub fn get_text(&mut self) -> Result<String> {
        if let Some(clipboard) = &mut self.clipboard {
            let text = clipboard.get_text()?;
            Ok(text)
        } else {
            Err(anyhow::anyhow!("Clipboard not available"))
        }
    }

    pub fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize clipboard manager")
    }
}
