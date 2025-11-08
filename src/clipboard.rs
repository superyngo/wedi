use anyhow::Result;
use arboard::Clipboard;

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            clipboard: Clipboard::new()?,
        })
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        self.clipboard.set_text(text)?;
        Ok(())
    }

    pub fn get_text(&mut self) -> Result<String> {
        let text = self.clipboard.get_text()?;
        Ok(text)
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize clipboard")
    }
}
