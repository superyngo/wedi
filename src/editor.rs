use crate::buffer::{EncodingConfig, RopeBuffer};
use crate::clipboard::ClipboardManager;
use crate::comment::CommentHandler;
use crate::cursor::Cursor;
use crate::input::{handle_key_event, Command, Direction};
use crate::search::Search;
use crate::terminal::Terminal;
use crate::utils::visual_width;
use crate::view::{Selection, View};
use anyhow::Result;
use std::path::Path;

pub struct Editor {
    buffer: RopeBuffer,
    cursor: Cursor,
    view: View,
    terminal: Terminal,
    clipboard: ClipboardManager,
    internal_clipboard: String, // 內部剪貼簿作為後備
    search: Search,
    comment_handler: CommentHandler,
    should_quit: bool,
    selection: Option<Selection>,
    selection_mode: bool, // F1 選擇模式開關
    message: Option<String>,
    quit_times: u8, // 追蹤連續按 Ctrl+Q 的次數
    debug_mode: bool,
}

impl Editor {
    pub fn new(
        file_path: Option<&Path>,
        debug_mode: bool,
        encoding_config: &EncodingConfig,
    ) -> Result<Self> {
        let buffer = if let Some(path) = file_path {
            // 使用新的方法，支持指定編碼
            RopeBuffer::from_file_with_encoding(path, encoding_config)?
        } else {
            let mut buffer = RopeBuffer::new();
            // 如果指定了讀取編碼，設置編碼
            if let Some(enc) = encoding_config.read_encoding {
                buffer.set_read_encoding(enc);
            }
            // 如果指定了存檔編碼，設置存檔編碼
            if let Some(enc) = encoding_config.save_encoding {
                buffer.set_save_encoding(enc);
            }
            buffer
        };

        let terminal = Terminal::new()?;
        let view = View::new(&terminal);
        let clipboard = ClipboardManager::new()?;

        let mut comment_handler = CommentHandler::new();
        if let Some(path) = file_path {
            comment_handler.detect_from_path(path);
        }

        Ok(Self {
            buffer,
            cursor: Cursor::new(),
            view,
            terminal,
            clipboard,
            internal_clipboard: String::new(), // 初始化內部剪貼簿
            search: Search::new(),
            comment_handler,
            should_quit: false,
            selection: None,
            selection_mode: false, // 預設關閉選擇模式
            message: None,
            quit_times: 0,
            debug_mode,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        Terminal::enter_raw_mode()?;
        Terminal::clear_screen()?;

        while !self.should_quit {
            let debug_info = if self.debug_mode {
                Some(self.get_debug_info())
            } else {
                None
            };

            self.view.render(
                &self.buffer,
                &self.cursor,
                self.selection.as_ref(),
                self.selection_mode,
                if self.debug_mode {
                    debug_info.as_deref()
                } else {
                    self.message.as_deref()
                },
                &self.comment_handler,
            )?;

            let key_event = Terminal::read_key()?;

            if let Some(command) = handle_key_event(key_event, self.selection_mode) {
                self.handle_command(command)?;
            }
        }

        Terminal::exit_raw_mode()?;
        Ok(())
    }

    fn handle_command(&mut self, command: Command) -> Result<()> {
        // 任何非 Quit 的命令都重置 quit_times
        if !matches!(command, Command::Quit) {
            self.quit_times = 0;
        }

        match command {
            // 字符輸入
            Command::Insert(ch) => {
                if self.has_selection() {
                    self.delete_selection();
                }

                let pos = self.cursor.char_position(&self.buffer);
                self.buffer.insert_char(pos, ch);

                if ch == '\n' {
                    self.cursor.row += 1;
                    self.cursor.reset_to_line_start();
                } else {
                    self.cursor.set_position(
                        &self.buffer,
                        &self.view,
                        self.cursor.row,
                        self.cursor.col + 1,
                    );
                }

                self.selection = None;
                self.selection_mode = false; // 輸入後關閉選擇模式
            }

            // 刪除操作
            Command::Backspace => {
                if self.has_selection() {
                    self.delete_selection();
                } else if self.cursor.col > 0 {
                    let new_col = self.cursor.col - 1;
                    let pos = self.buffer.line_to_char(self.cursor.row) + new_col;
                    self.buffer.delete_char(pos);
                    self.cursor
                        .set_position(&self.buffer, &self.view, self.cursor.row, new_col);
                } else if self.cursor.row > 0 {
                    // 刪除換行符，合併到上一行
                    let new_row = self.cursor.row - 1;
                    let prev_line_len = self
                        .buffer
                        .get_line_content(new_row)
                        .trim_end_matches(['\n', '\r'])
                        .chars()
                        .count();

                    let pos = self.buffer.line_to_char(new_row) + prev_line_len;
                    self.buffer.delete_char(pos);

                    self.cursor
                        .set_position(&self.buffer, &self.view, new_row, prev_line_len);
                }
                self.selection_mode = false; // 刪除後關閉選擇模式
            }

            Command::Delete => {
                if self.has_selection() {
                    self.delete_selection();
                } else {
                    let pos = self.cursor.char_position(&self.buffer);
                    self.buffer.delete_char(pos);
                }
                self.selection_mode = false; // 刪除後關閉選擇模式
            }

            Command::DeleteLine => {
                if self.has_selection() {
                    self.delete_selection();
                } else {
                    self.buffer.delete_line(self.cursor.row);
                    // 如果刪除後超出範圍,調整到最後一行
                    if self.cursor.row >= self.buffer.line_count() && self.buffer.line_count() > 0 {
                        self.cursor.row = self.buffer.line_count() - 1;
                    }
                    self.cursor.reset_to_line_start();
                }
                self.selection_mode = false; // 刪除後關閉選擇模式
            }

            // 光標移動
            Command::MoveUp => {
                self.cursor.move_up(&self.buffer, &self.view);
                self.selection = None;
            }
            Command::MoveDown => {
                self.cursor.move_down(&self.buffer, &self.view);
                self.selection = None;
            }
            Command::MoveLeft => {
                self.cursor.move_left(&self.buffer, &self.view);
                self.selection = None;
            }
            Command::MoveRight => {
                self.cursor.move_right(&self.buffer, &self.view);
                self.selection = None;
            }
            Command::MoveHome => {
                self.cursor.move_to_line_start();
                self.selection = None;
            }
            Command::MoveEnd => {
                self.cursor.move_to_line_end(&self.buffer, &self.view);
                self.selection = None;
            }
            Command::PageUp => {
                let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
                self.cursor
                    .move_page_up(&self.buffer, &self.view, effective_rows);
                self.selection = None;
            }
            Command::PageDown => {
                let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
                self.cursor
                    .move_page_down(&self.buffer, &self.view, effective_rows);
                self.selection = None;
            }

            Command::MoveToFileStart => {
                self.cursor.move_to_file_start(&self.view);
                self.selection = None;
            }
            Command::MoveToFileEnd => {
                self.cursor.move_to_file_end(&self.buffer, &self.view);
                self.selection = None;
            }
            // Command::MoveToLineStart => {
            //     self.cursor.move_to_line_start();
            //     self.selection = None;
            // }
            // Command::MoveToLineEnd => {
            //     self.cursor.move_to_line_end(&self.buffer, &self.view);
            //     self.selection = None;
            // }

            // 選擇操作
            Command::ExtendSelection(direction) => {
                if self.selection.is_none() {
                    self.selection = Some(Selection {
                        start: (self.cursor.row, self.cursor.col),
                        end: (self.cursor.row, self.cursor.col),
                    });
                }

                match direction {
                    Direction::Up => self.cursor.move_up(&self.buffer, &self.view),
                    Direction::Down => self.cursor.move_down(&self.buffer, &self.view),
                    Direction::Left => self.cursor.move_left(&self.buffer, &self.view),
                    Direction::Right => self.cursor.move_right(&self.buffer, &self.view),
                    Direction::Home => self.cursor.move_to_line_start(),
                    Direction::End => self.cursor.move_to_line_end(&self.buffer, &self.view),
                    Direction::FileStart => {
                        self.cursor.move_to_file_start(&self.view);
                    }
                    Direction::FileEnd => {
                        self.cursor.move_to_file_end(&self.buffer, &self.view);
                    }
                    Direction::PageUp => {
                        let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
                        self.cursor
                            .move_page_up(&self.buffer, &self.view, effective_rows)
                    }
                    Direction::PageDown => {
                        let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
                        self.cursor
                            .move_page_down(&self.buffer, &self.view, effective_rows)
                    }
                }

                if let Some(sel) = &mut self.selection {
                    sel.end = (self.cursor.row, self.cursor.col);
                }
            }

            Command::SelectAll => {
                let last_line = self.buffer.line_count().saturating_sub(1);
                let last_col = self
                    .buffer
                    .get_line_content(last_line)
                    .trim_end_matches(['\n', '\r'])
                    .chars()
                    .count();

                self.selection = Some(Selection {
                    start: (0, 0),
                    end: (last_line, last_col),
                });
                self.cursor.row = last_line;
                self.cursor.col = last_col;
            }

            Command::ClearSelection => {
                self.selection = None;
            }

            Command::ClearMessage => {
                self.selection = None;
                self.selection_mode = false; // ESC 關閉選擇模式但保留選擇範圍
                self.message = None;
            }

            // 選擇模式切換
            Command::ToggleSelectionMode => {
                self.selection_mode = !self.selection_mode;

                // 開啟選擇模式時，如果沒有選擇範圍，初始化選擇
                if self.selection_mode && self.selection.is_none() {
                    self.selection = Some(Selection {
                        start: (self.cursor.row, self.cursor.col),
                        end: (self.cursor.row, self.cursor.col),
                    });
                }

                self.message = Some(format!(
                    "Selection Mode: {}",
                    if self.selection_mode { "ON" } else { "OFF" }
                ));
            }

            // 剪貼板操作
            Command::Copy => {
                let text = if self.has_selection() {
                    self.get_selected_text()
                } else {
                    // 複製當前整行（完整內容，包括尾部空格和換行符）
                    let line_text = self.buffer.get_line_full(self.cursor.row);
                    // 確保以換行符結尾（用於識別整行貼上）
                    if line_text.ends_with('\n') {
                        line_text
                    } else {
                        format!("{}\n", line_text)
                    }
                };

                // 嘗試系統剪貼簿,失敗則使用內部剪貼簿
                if self.clipboard.set_text(&text).is_err() {
                    self.internal_clipboard = text;
                    if !self.clipboard.is_available() {
                        self.message = Some("Copied (internal clipboard)".to_string());
                    }
                } else {
                    self.internal_clipboard = text; // 同步到內部剪貼簿
                }

                // 複製後關閉選擇模式但保留選擇範圍
                self.selection_mode = false;

                // 直接使用內部剪貼簿
                // self.internal_clipboard = text;
            }

            Command::Cut => {
                let text = if self.has_selection() {
                    self.get_selected_text()
                } else {
                    // 剪切當前整行（完整內容）
                    let line_text = self.buffer.get_line_full(self.cursor.row);
                    // 確保以換行符結尾
                    if line_text.ends_with('\n') {
                        line_text
                    } else {
                        format!("{}\n", line_text)
                    }
                };

                // 嘗試系統剪貼簿,失敗則使用內部剪貼簿
                let copy_success = if self.clipboard.set_text(&text).is_err() {
                    self.internal_clipboard = text;
                    if !self.clipboard.is_available() {
                        self.message = Some("Cut (internal clipboard)".to_string());
                    }
                    true
                } else {
                    self.internal_clipboard = text; // 同步到內部剪貼簿
                    true
                };

                // 直接使用內部剪貼簿
                // self.internal_clipboard = text;
                // let copy_success = true;

                // 剪切成功後刪除內容
                if copy_success {
                    if self.has_selection() {
                        self.delete_selection();
                    } else {
                        self.buffer.delete_line(self.cursor.row);
                        // 剪切後光標上移一行
                        // if self.cursor.row > 0 {
                        //     self.cursor.row -= 1;
                        // }
                        // 如果刪除後超出範圍,調整到最後一行
                        if self.cursor.row >= self.buffer.line_count()
                            && self.buffer.line_count() > 0
                        {
                            self.cursor.row = self.buffer.line_count() - 1;
                        }
                        self.cursor.col = 0;
                        self.cursor.desired_visual_col = 0;
                    }
                }

                // 剪切後關閉選擇模式並清除選擇
                self.selection_mode = false;
            }

            Command::Paste => {
                // 嘗試從系統剪貼簿獲取,失敗則使用內部剪貼簿
                let text = self.clipboard.get_text().unwrap_or_else(|_| {
                    if self.internal_clipboard.is_empty() {
                        if !self.clipboard.is_available() {
                            self.message =
                                Some("Nothing to paste (internal clipboard)".to_string());
                        }
                        String::new()
                    } else {
                        self.internal_clipboard.clone()
                    }
                });

                // 使用內部剪貼簿
                // let text = self.internal_clipboard.clone();

                if !text.is_empty() {
                    if self.has_selection() {
                        self.delete_selection();
                    }

                    // 檢查是否為整行貼上（文字以換行結尾）
                    let is_whole_line = text.ends_with('\n');

                    if is_whole_line {
                        // 整行貼上：在光標所在行的開始處插入
                        // 這樣會將原行內容推到下一行
                        let line_start = self.buffer.line_to_char(self.cursor.row);
                        self.buffer.insert(line_start, &text);

                        // 光標移動到新插入行的開始
                        self.cursor.col = 0;
                        self.cursor.desired_visual_col = 0;
                    } else {
                        // 普通貼上：在光標位置插入
                        let pos = self.cursor.char_position(&self.buffer);
                        self.buffer.insert(pos, &text);

                        // 移動到貼上內容末尾
                        for ch in text.chars() {
                            if ch == '\n' {
                                self.cursor.row += 1;
                                self.cursor.col = 0;
                            } else {
                                self.cursor.col += 1;
                            }
                        }
                        self.cursor.desired_visual_col = self.cursor.col;
                    }
                }
                self.selection_mode = false; // 貼上後關閉選擇模式
            }

            // 內部剪貼板操作（僅使用內部剪貼簿）
            Command::CopyInternal => {
                let text = if self.has_selection() {
                    self.get_selected_text()
                } else {
                    // 複製當前整行（完整內容，包括尾部空格和換行符）
                    let line_text = self.buffer.get_line_full(self.cursor.row);
                    // 確保以換行符結尾（用於識別整行貼上）
                    if line_text.ends_with('\n') {
                        line_text
                    } else {
                        format!("{}\n", line_text)
                    }
                };

                // 直接使用內部剪貼簿
                self.internal_clipboard = text;
                self.message = Some("Copied (internal clipboard)".to_string());
                self.selection_mode = false; // 複製後關閉選擇模式
            }

            Command::CutInternal => {
                let text = if self.has_selection() {
                    self.get_selected_text()
                } else {
                    // 剪切當前整行（完整內容）
                    let line_text = self.buffer.get_line_full(self.cursor.row);
                    // 確保以換行符結尾
                    if line_text.ends_with('\n') {
                        line_text
                    } else {
                        format!("{}\n", line_text)
                    }
                };

                // 直接使用內部剪貼簿
                self.internal_clipboard = text;
                self.message = Some("Cut (internal clipboard)".to_string());

                // 剪切後刪除內容
                if self.has_selection() {
                    self.delete_selection();
                } else {
                    self.buffer.delete_line(self.cursor.row);
                    // 如果刪除後超出範圍,調整到最後一行
                    if self.cursor.row >= self.buffer.line_count() && self.buffer.line_count() > 0 {
                        self.cursor.row = self.buffer.line_count() - 1;
                    }
                    self.cursor.col = 0;
                    self.cursor.desired_visual_col = 0;
                }
                self.selection_mode = false; // 剪切後關閉選擇模式
            }

            Command::PasteInternal => {
                // 直接使用內部剪貼簿
                let text = self.internal_clipboard.clone();

                if text.is_empty() {
                    self.message = Some("Nothing to paste (internal clipboard)".to_string());
                } else {
                    if self.has_selection() {
                        self.delete_selection();
                    }

                    // 檢查是否為整行貼上（文字以換行結尾）
                    let is_whole_line = text.ends_with('\n');

                    if is_whole_line {
                        // 整行貼上：在光標所在行的開始處插入
                        // 這樣會將原行內容推到下一行
                        let line_start = self.buffer.line_to_char(self.cursor.row);
                        self.buffer.insert(line_start, &text);

                        // 光標移動到新插入行的開始
                        self.cursor.col = 0;
                        self.cursor.desired_visual_col = 0;
                    } else {
                        // 普通貼上：在光標位置插入
                        let pos = self.cursor.char_position(&self.buffer);
                        self.buffer.insert(pos, &text);

                        // 移動到貼上內容末尾
                        for ch in text.chars() {
                            if ch == '\n' {
                                self.cursor.row += 1;
                                self.cursor.col = 0;
                            } else {
                                self.cursor.col += 1;
                            }
                        }
                        self.cursor.desired_visual_col = self.cursor.col;
                    }
                }
                self.selection_mode = false; // 貼上後關閉選擇模式
            }

            // 文件操作
            Command::Save => {
                if let Err(e) = self.buffer.save() {
                    self.message = Some(format!("Save failed: {}", e));
                } else {
                    self.message = Some("File saved".to_string());
                }
            }

            Command::Quit => {
                if self.buffer.is_modified() {
                    if self.quit_times > 0 {
                        // 第二次按 Ctrl+Q，強制退出
                        self.should_quit = true;
                    } else {
                        // 第一次按 Ctrl+Q，顯示警告
                        self.quit_times = 1;
                        self.message = Some(
                            "Unsaved changes! Press Ctrl+Q again to force quit, or Ctrl+W to save"
                                .to_string(),
                        );
                    }
                } else {
                    self.should_quit = true;
                }
            }

            // 視窗調整
            Command::Resize => {
                self.view.update_size();
            }

            // 撤銷/重做
            Command::Undo => {
                if let Some(pos) = self.buffer.undo() {
                    // 將光標移動到撤銷操作的位置
                    let row = self.buffer.char_to_line(pos);
                    let line_start = self.buffer.line_to_char(row);
                    let col = pos - line_start;

                    self.cursor.row = row;
                    self.cursor.col = col;
                    self.cursor.desired_visual_col = col;
                    self.message = Some("Undo".to_string());
                } else {
                    self.message = Some("Nothing to undo".to_string());
                }
            }

            Command::Redo => {
                if let Some(pos) = self.buffer.redo() {
                    // 將光標移動到重做操作的位置
                    let row = self.buffer.char_to_line(pos);
                    let line_start = self.buffer.line_to_char(row);
                    let col = pos - line_start;

                    self.cursor.row = row;
                    self.cursor.col = col;
                    self.cursor.desired_visual_col = col;
                    self.message = Some("Redo".to_string());
                } else {
                    self.message = Some("Nothing to redo".to_string());
                }
            }

            // 搜索
            Command::Find => {
                // 獲取搜索查詢
                if let Ok(Some(query)) = crate::dialog::prompt("Search:", self.terminal.size()) {
                    if !query.is_empty() {
                        self.search.set_query(query.clone());
                        self.search.find_matches(&self.buffer);

                        if self.search.match_count() > 0 {
                            if let Some((row, col)) = self.search.next_match() {
                                self.cursor.row = row;
                                self.cursor.col = col;
                                self.cursor.desired_visual_col = col;
                                self.message = Some(format!(
                                    "Found {} matches (F3: next, Shift+F3: prev)",
                                    self.search.match_count()
                                ));
                            }
                        } else {
                            self.message = Some(format!("No matches found for '{}'", query));
                        }
                    }
                }
            }

            Command::FindNext => {
                if self.search.match_count() > 0 {
                    if let Some((row, col)) = self.search.next_match() {
                        self.cursor.row = row;
                        self.cursor.col = col;
                        self.cursor.desired_visual_col = col;
                        self.message = Some(format!(
                            "Match {}/{}",
                            (self.search.match_count() + 1) % self.search.match_count() + 1,
                            self.search.match_count()
                        ));
                    }
                } else {
                    self.message = Some("No active search".to_string());
                }
            }

            Command::FindPrev => {
                if self.search.match_count() > 0 {
                    if let Some((row, col)) = self.search.prev_match() {
                        self.cursor.row = row;
                        self.cursor.col = col;
                        self.cursor.desired_visual_col = col;
                        self.message = Some(format!(
                            "Match {}/{}",
                            (self.search.match_count() + 1) % self.search.match_count() + 1,
                            self.search.match_count()
                        ));
                    }
                } else {
                    self.message = Some("No active search".to_string());
                }
            }

            // 視圖控制
            Command::ToggleLineNumbers => {
                self.view.toggle_line_numbers();
            }

            // 註解切換
            Command::ToggleComment => {
                if !self.comment_handler.has_comment_style() {
                    self.message = Some("No comment style for this file type".to_string());
                } else if self.has_selection() {
                    // 多行選擇：智能切換註解
                    if let Some(sel) = self.selection {
                        let (start_row, _) = sel.start.min(sel.end);
                        let (end_row, _) = sel.start.max(sel.end);

                        // 檢查是否有任何一行沒有註解
                        let mut has_uncommented = false;
                        for row in start_row..=end_row {
                            let line_content = self.buffer.get_line_content(row);
                            if !self.comment_handler.is_commented(&line_content) {
                                has_uncommented = true;
                                break;
                            }
                        }

                        // 如果有任何一行沒註解，全部加註解；否則全部取消註解
                        let should_add_comment = has_uncommented;

                        // 從後往前處理，避免行號變化
                        for row in (start_row..=end_row).rev() {
                            let line_content = self.buffer.get_line_content(row);

                            let new_line = if should_add_comment {
                                // 全部加註解（即使已經有註解的也保持不變）
                                if self.comment_handler.is_commented(&line_content) {
                                    Some(line_content.clone())
                                } else {
                                    self.comment_handler.add_comment(&line_content)
                                }
                            } else {
                                // 全部取消註解
                                self.comment_handler.remove_comment(&line_content)
                            };

                            if let Some(new_line) = new_line {
                                // 計算行的起始和結束位置
                                let line_start = self.buffer.line_to_char(row);
                                let line_end = if row + 1 < self.buffer.line_count() {
                                    self.buffer.line_to_char(row + 1)
                                } else {
                                    self.buffer.len_chars()
                                };

                                // 刪除舊行（包括換行符）
                                self.buffer.delete_range(line_start, line_end);

                                // 插入新行（保留換行符）
                                let new_line_with_newline = if line_content.ends_with('\n')
                                    || line_content.ends_with("\r\n")
                                {
                                    format!("{}\n", new_line.trim_end_matches(['\n', '\r']))
                                } else {
                                    new_line.trim_end_matches(['\n', '\r']).to_string()
                                };
                                self.buffer.insert(line_start, &new_line_with_newline);
                            }
                        }

                        // 保留選擇狀態（不清除選取）
                        self.cursor.row = start_row;
                        self.cursor.col = 0;
                        self.cursor.desired_visual_col = 0;

                        let action = if should_add_comment {
                            "Added"
                        } else {
                            "Removed"
                        };
                        self.message = Some(format!("{} comments", action));
                    }
                } else {
                    // 單行：直接切換註解
                    let line_content = self.buffer.get_line_content(self.cursor.row);
                    if let Some(new_line) = self.comment_handler.toggle_line_comment(&line_content)
                    {
                        // 計算行的起始和結束位置
                        let line_start = self.buffer.line_to_char(self.cursor.row);
                        let line_end = if self.cursor.row + 1 < self.buffer.line_count() {
                            self.buffer.line_to_char(self.cursor.row + 1)
                        } else {
                            self.buffer.len_chars()
                        };

                        // 刪除舊行（包括換行符）
                        self.buffer.delete_range(line_start, line_end);

                        // 插入新行（保留換行符）
                        let new_line_with_newline =
                            if line_content.ends_with('\n') || line_content.ends_with("\r\n") {
                                format!("{}\n", new_line.trim_end_matches(['\n', '\r']))
                            } else {
                                new_line.trim_end_matches(['\n', '\r']).to_string()
                            };
                        self.buffer.insert(line_start, &new_line_with_newline);

                        self.message = Some("Toggled comment".to_string());
                    }
                }
            }

            // 縮排（Tab 鍵）
            Command::Indent => {
                if self.has_selection() {
                    // 多行選擇：對每行添加 4 個空格
                    if let Some(sel) = self.selection {
                        let (start_row, _) = sel.start.min(sel.end);
                        let (end_row, _) = sel.start.max(sel.end);

                        // 從後往前處理，避免行號變化
                        for row in (start_row..=end_row).rev() {
                            let line_start = self.buffer.line_to_char(row);
                            self.buffer.insert(line_start, "    ");
                        }

                        // 保留選擇狀態
                        self.cursor.row = start_row;
                        self.cursor.col = 0;
                        self.cursor.desired_visual_col = 0;
                    }
                } else {
                    // 單行：在光標位置插入 4 個空格
                    let pos = self.cursor.char_position(&self.buffer);
                    self.buffer.insert(pos, "    ");
                    self.cursor.col += 4;
                    self.cursor.desired_visual_col = self.cursor.col;
                }
            }

            // 退位（Shift+Tab 鍵）
            Command::Unindent => {
                if self.has_selection() {
                    // 多行選擇：對每行刪除最多 4 個前導空格
                    if let Some(sel) = self.selection {
                        let (start_row, _) = sel.start.min(sel.end);
                        let (end_row, _) = sel.start.max(sel.end);

                        // 從後往前處理，避免行號變化
                        for row in (start_row..=end_row).rev() {
                            let line_content = self.buffer.get_line_content(row);
                            let spaces_to_remove = line_content
                                .chars()
                                .take_while(|&c| c == ' ')
                                .take(4)
                                .count();

                            if spaces_to_remove > 0 {
                                let line_start = self.buffer.line_to_char(row);
                                self.buffer
                                    .delete_range(line_start, line_start + spaces_to_remove);
                            }
                        }

                        // 保留選擇狀態
                        self.cursor.row = start_row;
                        self.cursor.col = 0;
                        self.cursor.desired_visual_col = 0;
                    }
                } else {
                    // 單行：刪除光標前最多 4 個空格
                    let line_content = self.buffer.get_line_content(self.cursor.row);
                    let before_cursor: String =
                        line_content.chars().take(self.cursor.col).collect();
                    let spaces_to_remove = before_cursor
                        .chars()
                        .rev()
                        .take_while(|&c| c == ' ')
                        .take(4)
                        .count();

                    if spaces_to_remove > 0 {
                        let line_start = self.buffer.line_to_char(self.cursor.row);
                        let delete_start = line_start + self.cursor.col - spaces_to_remove;
                        self.buffer
                            .delete_range(delete_start, delete_start + spaces_to_remove);
                        self.cursor.col -= spaces_to_remove;
                        self.cursor.desired_visual_col = self.cursor.col;
                    }
                }
            }

            // 跳轉到行
            Command::GoToLine => {
                if let Ok(Some(line_str)) =
                    crate::dialog::prompt("Go to line:", self.terminal.size())
                {
                    if let Ok(line_num) = line_str.trim().parse::<usize>() {
                        if line_num > 0 && line_num <= self.buffer.line_count() {
                            self.cursor.row = line_num - 1;
                            self.cursor.col = 0;
                            self.cursor.desired_visual_col = 0;
                            self.message = Some(format!("Jumped to line {}", line_num));
                        } else {
                            self.message = Some(format!("Invalid line number: {}", line_num));
                        }
                    } else {
                        self.message = Some("Please enter a valid number".to_string());
                    }
                }
            }
        }

        Ok(())
    }

    fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    fn get_selected_text(&self) -> String {
        if let Some(sel) = self.selection {
            let (start_row, start_col) = sel.start.min(sel.end);
            let (end_row, end_col) = sel.start.max(sel.end);

            let mut text = String::new();

            for row in start_row..=end_row {
                let line = self.buffer.get_line_content(row);
                let line = line.trim_end_matches(['\n', '\r']);

                if row == start_row && row == end_row {
                    // 單行選擇
                    let chars: Vec<char> = line.chars().collect();
                    text.push_str(
                        &chars[start_col..end_col.min(chars.len())]
                            .iter()
                            .collect::<String>(),
                    );
                } else if row == start_row {
                    // 第一行
                    let chars: Vec<char> = line.chars().collect();
                    text.push_str(&chars[start_col..].iter().collect::<String>());
                    text.push('\n');
                } else if row == end_row {
                    // 最後一行
                    let chars: Vec<char> = line.chars().collect();
                    text.push_str(&chars[..end_col.min(chars.len())].iter().collect::<String>());
                } else {
                    // 中間行
                    text.push_str(line);
                    text.push('\n');
                }
            }

            text
        } else {
            String::new()
        }
    }

    fn delete_selection(&mut self) {
        if let Some(sel) = self.selection {
            let (start_row, start_col) = sel.start.min(sel.end);
            let (end_row, end_col) = sel.start.max(sel.end);

            let start_pos = self.buffer.line_to_char(start_row) + start_col;
            let end_pos = self.buffer.line_to_char(end_row) + end_col;

            self.buffer.delete_range(start_pos, end_pos);

            self.cursor
                .set_position(&self.buffer, &self.view, start_row, start_col);
            self.selection = None;
        }
    }

    fn get_debug_info(&self) -> String {
        let total_lines = self.buffer.line_count();
        let screen_rows = self.view.screen_rows;
        let logical_row = self.cursor.row;
        let logical_col = self.cursor.col;
        let visual_line_index = self.cursor.visual_line_index;

        // 計算可用列寬度
        let available_width = self.view.get_available_width(&self.buffer);

        // 計算當前行的視覺列位置和總字符數
        let (
            visual_col_in_line,
            line_char_count,
            line_visual_width,
            total_visual_lines,
            current_visual_line_width,
        ) = if let Some(line) = self.buffer.line(logical_row) {
            let line_str = line.to_string();
            let line_str = line_str.trim_end_matches(['\n', '\r']);
            let visual_col = self.view.logical_col_to_visual_col(line_str, logical_col);
            let char_count = line_str.chars().count();

            // 計算在當前視覺行內的列位置
            let visual_lines = self
                .view
                .calculate_visual_lines_for_row(&self.buffer, logical_row);
            let total_visual_lines = visual_lines.len();
            let mut accumulated = 0;
            for line in visual_lines
                .iter()
                .take(visual_line_index.min(visual_lines.len()))
            {
                accumulated += visual_width(line);
            }
            let col_in_visual_line = visual_col.saturating_sub(accumulated);

            // 計算整行的視覺寬度
            let line_visual_width = visual_width(line_str);

            // 計算當前視覺行的寬度
            let current_visual_line_width = if visual_line_index < visual_lines.len() {
                visual_width(&visual_lines[visual_line_index])
            } else {
                0
            };

            (
                col_in_visual_line,
                char_count,
                line_visual_width,
                total_visual_lines,
                current_visual_line_width,
            )
        } else {
            (0, 0, 0, 0, 0)
        };

        // 計算選取的邏輯字數和顯示寬度
        let (selection_char_count, selection_visual_width) = if self.selection.is_some() {
            let selected_text = self.get_selected_text();
            let char_count = selected_text.chars().count();
            let visual_width = visual_width(&selected_text);
            (char_count, visual_width)
        } else {
            (0, 0)
        };

        format!(
            "DEBUG | AA:{}x{} LL:L{}/{}:C{}/{}:{} VL:L{}/{}:C{}/{} SC:{}:{}",
            screen_rows,
            available_width,
            logical_row + 1,
            total_lines,
            logical_col,
            line_char_count,
            line_visual_width,
            visual_line_index + 1,
            total_visual_lines,
            visual_col_in_line,
            current_visual_line_width,
            selection_char_count,
            selection_visual_width
        )
    }
}
