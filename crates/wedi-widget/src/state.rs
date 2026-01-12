use wedi_core::{Command, Cursor, Keymap, RopeBuffer, Search, Selection, View};
use crate::config::EditorConfig;
use anyhow::Result;
use crossterm::event::KeyEvent;
use std::path::Path;

#[cfg(feature = "syntax-highlighting")]
use wedi_core::highlight::{HighlightCache, HighlightEngine};

/// 編輯器事件
#[derive(Debug, Clone, PartialEq)]
pub enum EditorEvent {
    None,
    ContentChanged,
    SaveRequested,
    QuitRequested,
    Custom(String),
}

/// 編輯器狀態
pub struct EditorState {
    pub buffer: RopeBuffer,
    pub cursor: Cursor,
    pub selection: Option<Selection>,
    pub keymap: Keymap,
    pub search: Search,
    pub view: View,
    pub config: EditorConfig,

    selection_mode: bool,
    modified: bool,
    file_path: Option<std::path::PathBuf>,

    #[cfg(feature = "syntax-highlighting")]
    pub highlight_engine: Option<HighlightEngine>,
    #[cfg(feature = "syntax-highlighting")]
    pub highlight_cache: HighlightCache,
}

impl EditorState {
    /// 建立新的編輯器狀態
    pub fn new() -> Self {
        let buffer = RopeBuffer::new();
        let cursor = Cursor::new();
        let view = View::new_simple(24, 80); // 預設尺寸
        let keymap = Keymap::default();
        let search = Search::new();
        let config = EditorConfig::default();

        Self {
            buffer,
            cursor,
            selection: None,
            keymap,
            search,
            view,
            config,
            selection_mode: false,
            modified: false,
            file_path: None,
            #[cfg(feature = "syntax-highlighting")]
            highlight_engine: None,
            #[cfg(feature = "syntax-highlighting")]
            highlight_cache: HighlightCache::new(1000),
        }
    }

    /// 從內容建立編輯器
    pub fn with_content(content: &str) -> Self {
        let mut state = Self::new();
        state.set_content(content);
        state
    }

    /// 從檔案載入
    pub fn from_file(path: &Path) -> Result<Self> {
        let buffer = RopeBuffer::from_file(path)?;
        let cursor = Cursor::new();
        let view = View::new_simple(24, 80);
        let keymap = Keymap::default();
        let search = Search::new();
        let config = EditorConfig::default();

        Ok(Self {
            buffer,
            cursor,
            selection: None,
            keymap,
            search,
            view,
            config,
            selection_mode: false,
            modified: false,
            file_path: Some(path.to_path_buf()),
            #[cfg(feature = "syntax-highlighting")]
            highlight_engine: None,
            #[cfg(feature = "syntax-highlighting")]
            highlight_cache: HighlightCache::new(1000),
        })
    }

    /// 設定內容
    pub fn set_content(&mut self, text: &str) {
        self.buffer = RopeBuffer::from_str(text);
        self.cursor = Cursor::new();
        self.modified = true;
        self.view.invalidate_cache();
    }

    /// 取得內容
    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    /// 游標位置
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor.row, self.cursor.col)
    }

    /// 是否已修改
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// 處理按鍵事件
    pub fn handle_key(&mut self, event: KeyEvent) -> EditorEvent {
        if let Some(cmd) = self.keymap.get_command(event, self.selection_mode) {
            self.execute_command(cmd)
        } else {
            EditorEvent::None
        }
    }

    /// 處理視窗大小調整
    pub fn handle_resize(&mut self, cols: u16, rows: u16) {
        self.view.resize(rows as usize, cols as usize);
    }

    /// 執行命令
    fn execute_command(&mut self, command: Command) -> EditorEvent {
        match command {
            Command::Save => EditorEvent::SaveRequested,
            Command::Quit => EditorEvent::QuitRequested,
            Command::Insert(ch) => {
                self.buffer.insert_char(self.cursor.row, self.cursor.col, ch);
                self.modified = true;
                self.view.invalidate_line(self.cursor.row);
                EditorEvent::ContentChanged
            }
            Command::Backspace => {
                if self.cursor.col > 0 {
                    self.buffer.delete_char(self.cursor.row, self.cursor.col - 1);
                    self.cursor.col -= 1;
                    self.modified = true;
                    self.view.invalidate_line(self.cursor.row);
                }
                EditorEvent::ContentChanged
            }
            Command::Delete => {
                self.buffer.delete_char(self.cursor.row, self.cursor.col);
                self.modified = true;
                self.view.invalidate_line(self.cursor.row);
                EditorEvent::ContentChanged
            }
            Command::MoveUp => {
                self.cursor.move_up(&self.buffer, &self.view);
                EditorEvent::None
            }
            Command::MoveDown => {
                self.cursor.move_down(&self.buffer, &self.view);
                EditorEvent::None
            }
            Command::MoveLeft => {
                self.cursor.move_left(&self.buffer, &self.view);
                EditorEvent::None
            }
            Command::MoveRight => {
                self.cursor.move_right(&self.buffer, &self.view);
                EditorEvent::None
            }
            Command::MoveHome => {
                self.cursor.col = 0;
                EditorEvent::None
            }
            Command::MoveEnd => {
                self.cursor.move_to_line_end(&self.buffer, &self.view);
                EditorEvent::None
            }
            Command::ToggleSelectionMode => {
                self.selection_mode = !self.selection_mode;
                EditorEvent::None
            }
            _ => EditorEvent::None,
        }
    }

    /// 取得配置的可變引用
    pub fn config_mut(&mut self) -> &mut EditorConfig {
        &mut self.config
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}
