use anyhow::Result;
use std::path::Path;
use wedi_core::{
    buffer::{parse_encoding_label, EncodingConfig, RopeBuffer},
    clipboard::ClipboardManager,
    comment::CommentHandler,
    cursor::Cursor,
    keymap::{handle_key_event, Command, Direction},
    search::Search,
    terminal::{InputEvent, Terminal},
    utils::visual_width,
    view::{Selection, View},
};

#[cfg(feature = "syntax-highlighting")]
use wedi_core::highlight::{HighlightCache, HighlightConfig, HighlightEngine};

pub struct Editor {
    buffer: RopeBuffer,
    cursor: Cursor,
    view: View,
    terminal: Terminal,
    clipboard: ClipboardManager,
    internal_clipboard: String, // 內部剪貼簿作為後備
    line_copy: Option<String>,  // 最近一次「無選取複製整行」的內容；只有貼上相同文字時才整行貼上
    search: Search,
    search_mode: bool, // 搜尋模式開關（Ctrl+F 開啟，ESC 關閉）
    comment_handler: CommentHandler,
    should_quit: bool,
    selection: Option<Selection>,
    selection_mode: bool, // F1 選擇模式開關
    message: Option<String>,
    quit_pending: bool,     // 有未存修改時已按過一次 Ctrl+Q
    lossy_save_armed: bool, // 解碼有損的檔案：第一次 Ctrl+S 僅警告，再按一次才存檔
    debug_mode: bool,

    // 語法高亮（可選功能）
    #[cfg(feature = "syntax-highlighting")]
    pub(crate) highlight_engine: Option<HighlightEngine>,
    #[cfg(feature = "syntax-highlighting")]
    pub(crate) highlight_cache: HighlightCache,
    #[cfg(feature = "syntax-highlighting")]
    highlight_enabled: bool,
}

/// 解碼有損時的狀態列警告
fn lossy_warning(buffer: &RopeBuffer) -> String {
    format!(
        "Invalid {} bytes shown as \u{FFFD}; saving loses them (Ctrl+E: encoding).",
        buffer.read_encoding().name()
    )
}

impl Editor {
    pub fn new(
        file_path: Option<&Path>,
        debug_mode: bool,
        encoding_config: &EncodingConfig,
        #[cfg(feature = "syntax-highlighting")] theme: Option<&str>,
        #[cfg(feature = "syntax-highlighting")] language: Option<&str>,
    ) -> Result<Self> {
        let terminal = Terminal::new()?;
        #[cfg(feature = "syntax-highlighting")]
        return Self::with_terminal(
            terminal,
            file_path,
            debug_mode,
            encoding_config,
            theme,
            language,
        );
        #[cfg(not(feature = "syntax-highlighting"))]
        Self::with_terminal(terminal, file_path, debug_mode, encoding_config)
    }

    /// 以指定的 Terminal 建立編輯器（測試時可傳入固定尺寸的 Terminal）
    fn with_terminal(
        terminal: Terminal,
        file_path: Option<&Path>,
        debug_mode: bool,
        encoding_config: &EncodingConfig,
        #[cfg(feature = "syntax-highlighting")] theme: Option<&str>,
        #[cfg(feature = "syntax-highlighting")] language: Option<&str>,
    ) -> Result<Self> {
        let buffer = if let Some(path) = file_path {
            // 使用新的方法，支持指定編碼
            RopeBuffer::from_file_with_encoding(path, encoding_config)?
        } else {
            let mut buffer = RopeBuffer::new();
            // 如果指定了讀取編碼，設置編碼
            if let Some(enc) = encoding_config.read_encoding {
                wedi_core::debug_log!(
                    "Editor::new() - Setting read_encoding from config: {}",
                    enc.name()
                );
                buffer.set_read_encoding(enc);
            }
            // 如果指定了存檔編碼，設置存檔編碼
            if let Some(enc) = encoding_config.save_encoding {
                wedi_core::debug_log!(
                    "Editor::new() - Setting save_encoding from config: {}",
                    enc.name()
                );
                buffer.set_save_encoding(enc);
            }

            wedi_core::debug_log!(
                "Editor::new() - Final buffer save_encoding: {}",
                if let Some(enc) = encoding_config.save_encoding {
                    enc.name()
                } else if let Some(enc) = encoding_config.read_encoding {
                    enc.name()
                } else {
                    "system default"
                }
            );

            buffer
        };

        let mut view = View::new(&terminal);
        view.set_debug_ruler(debug_mode);
        let clipboard = ClipboardManager::new()?;

        let mut comment_handler = CommentHandler::new();
        if let Some(path) = file_path {
            comment_handler.detect_from_path(path);
        }

        // 啟動時的警告改顯示在狀態列（stderr 會被 TUI 蓋掉）
        #[allow(unused_mut)]
        let mut startup_warning: Option<String> = None;

        // 語法高亮初始化
        #[cfg(feature = "syntax-highlighting")]
        let (highlight_engine, highlight_cache) = {
            let mut config = HighlightConfig::default();

            // 如果提供了自定義主題，使用它；否則使用默認主題
            if let Some(custom_theme) = theme {
                config.theme = custom_theme.to_string();
            }

            let mut engine = if config.enabled {
                HighlightEngine::new(Some(&config.theme), config.true_color).ok()
            } else {
                None
            };

            // 設定語法類型：優先使用命令列指定的語言，否則從檔案路徑自動檢測
            if let Some(ref mut eng) = engine.as_mut() {
                if let Some(lang) = language {
                    // 使用者指定了語言，嘗試設定
                    if let Err(e) = eng.set_syntax_by_name(lang) {
                        startup_warning = Some(format!("Warning: {}", e));
                        // 失敗時回退到自動檢測
                        if let Some(path) = file_path {
                            eng.set_file(Some(path));
                        }
                    }
                } else if let Some(path) = file_path {
                    // 沒有指定語言，從檔案路徑自動檢測
                    eng.set_file(Some(path));
                }
            }

            (engine, HighlightCache::new())
        };

        // 解碼有無效位元組：開檔即在狀態列警告
        let message = buffer
            .is_lossy()
            .then(|| lossy_warning(&buffer))
            .or(startup_warning);

        Ok(Self {
            buffer,
            cursor: Cursor::new(),
            view,
            terminal,
            clipboard,
            internal_clipboard: String::new(), // 初始化內部剪貼簿
            line_copy: None,
            search: Search::new(),
            search_mode: false, // 預設關閉搜尋模式
            comment_handler,
            should_quit: false,
            selection: None,
            selection_mode: false, // 預設關閉選擇模式
            message,
            quit_pending: false,
            lossy_save_armed: false,
            debug_mode,

            #[cfg(feature = "syntax-highlighting")]
            highlight_engine,
            #[cfg(feature = "syntax-highlighting")]
            highlight_cache,
            #[cfg(feature = "syntax-highlighting")]
            highlight_enabled: true, // 預設啟用語法高亮
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

            // ⚠️ 重要：在計算高亮之前先更新 offset_row
            // 避免跳頁後 highlighted_lines 使用舊的 offset_row
            let has_debug_ruler = self.debug_mode;
            self.view
                .scroll_if_needed(&self.cursor, &self.buffer, has_debug_ruler);

            // 獲取語法高亮行
            #[cfg(feature = "syntax-highlighting")]
            let highlighted_lines = {
                if self.highlight_enabled {
                    let start_row = self.view.offset_row;
                    let end_row = start_row + self.view.screen_rows;
                    self.get_highlighted_lines(start_row, end_row)
                } else {
                    std::collections::HashMap::new()
                }
            };

            // 搜尋模式下傳入匹配位置供渲染高亮
            let search_highlight = if self.search_mode && self.search.match_count() > 0 {
                Some(wedi_core::SearchHighlight {
                    matches: self.search.get_matches(),
                    query_len: self.search.get_query().len(),
                    current: self.search.current_index(),
                })
            } else {
                None
            };

            self.view.render(
                &self.buffer,
                &self.cursor,
                self.selection.as_ref(),
                search_highlight.as_ref(),
                if self.debug_mode {
                    debug_info.as_deref()
                } else {
                    self.message.as_deref()
                },
                #[cfg(feature = "syntax-highlighting")]
                Some(&highlighted_lines),
            )?;

            // 使用 read_input() 支援 Bracketed Paste
            match Terminal::read_input()? {
                InputEvent::Key(key_event) => {
                    if let Some(command) = handle_key_event(key_event, self.selection_mode) {
                        self.handle_command(command)?;
                    }
                }
                InputEvent::Paste(text) => {
                    // 直接處理貼上的文字
                    self.handle_command(Command::PasteText(text))?;
                }
                InputEvent::Resize => self.handle_command(Command::Resize)?,
            }

            // 對話框取走了 Resize 事件：同步編輯器尺寸
            if crate::dialog::take_resized() {
                self.handle_command(Command::Resize)?;
            }
        }

        Terminal::exit_raw_mode()?;
        Ok(())
    }

    /// 執行一個命令；命令內的所有編輯構成一個撤銷群組
    fn handle_command(&mut self, command: Command) -> Result<()> {
        self.buffer.begin_group();
        let result = self.dispatch_command(command);
        self.buffer.end_group();
        result
    }

    fn dispatch_command(&mut self, command: Command) -> Result<()> {
        // 任何非 Quit 的命令都取消待確認的退出
        if !matches!(command, Command::Quit) {
            self.quit_pending = false;
        }
        if !matches!(command, Command::Save) {
            self.lossy_save_armed = false;
        }

        // 修改緩衝區的命令會使搜尋匹配位置失效：先退出搜尋模式（保留查詢字串）
        if self.search_mode
            && matches!(
                command,
                Command::Insert(_)
                    | Command::PasteText(_)
                    | Command::Backspace
                    | Command::Delete
                    | Command::DeleteLine
                    | Command::Indent
                    | Command::Unindent
                    | Command::Cut
                    | Command::CutInternal
                    | Command::Paste
                    | Command::PasteInternal
                    | Command::Undo
                    | Command::Redo
                    | Command::ToggleComment
            )
        {
            self.search_mode = false;
        }

        match command {
            // 字符輸入
            Command::Insert(ch) => {
                if self.has_selection() {
                    self.delete_selection();
                }

                let pos = self.cursor.char_position(&self.buffer);
                if ch == '\n' {
                    // 沿用檔案的行尾符號（CRLF 檔案插入 \r\n）
                    let eol = self.buffer.line_ending();
                    self.buffer.insert(pos, eol);
                } else {
                    self.buffer.insert_char(pos, ch);
                }

                // 優化：僅失效當前行（除非是換行符，需要重建整個緩存）
                if ch == '\n' {
                    self.view.invalidate_cache(); // 換行影響多行佈局
                    self.cursor.row += 1;
                    self.cursor.reset_to_line_start();
                } else {
                    self.view.invalidate_line(self.cursor.row); // 僅失效當前行
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
                    // 行內刪除
                    let new_col = self.cursor.col - 1;
                    let pos = self.buffer.line_to_char(self.cursor.row) + new_col;
                    self.buffer.delete_char(pos);
                    self.view.invalidate_line(self.cursor.row); // 僅失效當前行
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

                    // 一次刪除整個換行符（\n 或 \r\n）
                    let pos = self.buffer.line_to_char(new_row) + prev_line_len;
                    let end = self.buffer.line_to_char(self.cursor.row);
                    self.buffer.delete_range(pos, end);
                    self.view.invalidate_cache(); // 行合併影響多行

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
                    let line_content = self.buffer.get_line_content(self.cursor.row);
                    let at_line_end = self.cursor.col
                        >= line_content.trim_end_matches(['\n', '\r']).chars().count();

                    if at_line_end {
                        // 一次刪除整個換行符（\n 或 \r\n）
                        let end = if self.cursor.row + 1 < self.buffer.line_count() {
                            self.buffer.line_to_char(self.cursor.row + 1)
                        } else {
                            self.buffer.len_chars()
                        };
                        self.buffer.delete_range(pos, end);
                    } else {
                        self.buffer.delete_char(pos);
                    }

                    // 優化：如果在行尾刪除（會合併下一行），需要完全失效；否則僅失效當前行
                    if at_line_end {
                        self.view.invalidate_cache(); // 行合併影響多行
                    } else {
                        self.view.invalidate_line(self.cursor.row); // 僅失效當前行
                    }
                }
                self.selection_mode = false; // 刪除後關閉選擇模式
            }

            Command::DeleteLine => {
                if self.has_selection() {
                    // 選取模式下刪除所有包含選取的整行
                    if let Some(sel) = self.selection {
                        let (start_row, _) = sel.start.min(sel.end);
                        let (end_row, _) = sel.start.max(sel.end);

                        // 從後往前刪除，避免行號變化影響
                        for row in (start_row..=end_row).rev() {
                            if row < self.buffer.line_count() {
                                self.buffer.delete_line(row);
                            }
                        }

                        self.view.invalidate_cache();

                        // 確保光標在有效範圍內
                        self.cursor.row = start_row.min(self.buffer.line_count().saturating_sub(1));
                        self.cursor.reset_to_line_start();
                        self.selection = None;
                    }
                } else {
                    // 記錄是否在最後一行
                    let was_last_line = self.cursor.row == self.buffer.line_count() - 1;

                    self.buffer.delete_line(self.cursor.row);
                    self.view.invalidate_cache();

                    // 如果刪除的是最後一行且不是唯一一行，光標上移
                    if was_last_line && self.cursor.row > 0 {
                        self.cursor.row -= 1;
                    }

                    // 確保光標在有效範圍內
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

            Command::JumpTenthUp => {
                self.jump_tenth(true);
                self.selection = None;
            }

            Command::JumpTenthDown => {
                self.jump_tenth(false);
                self.selection = None;
            }

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
                    Direction::TenthUp => self.jump_tenth(true),
                    Direction::TenthDown => self.jump_tenth(false),
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

            Command::ClearMessage => {
                // ESC 一次只剝一層：訊息 → 選取/選擇模式 → 搜尋模式
                if self.message.is_some() {
                    self.message = None;
                } else if self.selection.is_some() || self.selection_mode {
                    self.selection = None;
                    self.selection_mode = false;
                } else {
                    self.search_mode = false; // 關閉搜尋模式（保留搜尋結果）
                }
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
                let text = self.get_copy_text();
                self.set_clipboard_text(text, true);
                // 複製後關閉選擇模式並清除選擇範圍
                self.selection_mode = false;
                self.selection = None;
            }

            Command::Cut => {
                self.do_cut(true);
            }

            Command::Paste => {
                let text = self.get_clipboard_text(true);
                self.paste_text(text);
                self.selection_mode = false; // 貼上後關閉選擇模式
            }

            // 內部剪貼板操作（僅使用內部剪貼簿）
            Command::CopyInternal => {
                let text = self.get_copy_text();
                self.set_clipboard_text(text, false);
                self.selection_mode = false; // 複製後關閉選擇模式
                self.selection = None; // 複製後清除選擇範圍
            }

            Command::CutInternal => {
                self.do_cut(false);
            }

            Command::PasteInternal => {
                let text = self.get_clipboard_text(false);
                self.paste_text(text);
                self.selection_mode = false; // 貼上後關閉選擇模式
            }

            // Bracketed Paste 直接貼上文字
            Command::PasteText(text) => {
                if self.has_selection() {
                    self.delete_selection();
                }
                self.paste_text(text);
                self.selection_mode = false; // 貼上後關閉選擇模式
            }

            // 文件操作
            Command::Save => {
                if self.buffer.is_lossy() && !self.lossy_save_armed {
                    self.lossy_save_armed = true;
                    self.message = Some(format!(
                        "{} Ctrl+S again: save anyway",
                        lossy_warning(&self.buffer)
                    ));
                } else if let Err(e) = self.buffer.save() {
                    self.message = Some(format!("Save failed: {}", e));
                } else {
                    self.message = Some("File saved".to_string());
                }
            }

            Command::Quit => {
                if self.buffer.is_modified() {
                    if self.quit_pending {
                        // 第二次按 Ctrl+Q，強制退出
                        self.should_quit = true;
                    } else {
                        // 第一次按 Ctrl+Q，顯示警告
                        self.quit_pending = true;
                        self.message = Some(
                            "Unsaved changes! Press Ctrl+Q again to force quit, or Ctrl+S to save"
                                .to_string(),
                        );
                    }
                } else {
                    self.should_quit = true;
                }
            }

            // 視窗調整
            Command::Resize => {
                self.terminal.update_size()?; // 對話框依賴 terminal.size()，必須同步更新
                self.view.update_size();
                self.resync_cursor();
            }

            // 撤銷/重做
            Command::Undo => {
                if let Some(pos) = self.buffer.undo() {
                    self.view.invalidate_cache();
                    // 將光標移動到撤銷操作的位置
                    let row = self.buffer.char_to_line(pos);
                    let line_start = self.buffer.line_to_char(row);
                    let col = pos - line_start;

                    self.cursor.set_position(&self.buffer, &self.view, row, col);
                    self.selection = None;
                    self.message = Some("Undo".to_string());
                } else {
                    self.message = Some("Nothing to undo".to_string());
                }
            }

            Command::Redo => {
                if let Some(pos) = self.buffer.redo() {
                    self.view.invalidate_cache();
                    // 將光標移動到重做操作的位置
                    let row = self.buffer.char_to_line(pos);
                    let line_start = self.buffer.line_to_char(row);
                    let col = pos - line_start;

                    self.cursor.set_position(&self.buffer, &self.view, row, col);
                    self.selection = None;
                    self.message = Some("Redo".to_string());
                } else {
                    self.message = Some("Nothing to redo".to_string());
                }
            }

            // 搜索
            Command::Find => {
                // 獲取搜索查詢，使用上次的搜索詞作為預設值
                let default_query = self.search.get_query();
                if let Ok(Some(query)) = crate::dialog::prompt_with_default(
                    "Search:",
                    default_query,
                    self.terminal.size(),
                ) {
                    if !query.is_empty() {
                        self.search.set_query(query.clone());
                        self.search.find_matches(&self.buffer);
                        self.search_mode = true; // 開啟搜尋模式

                        if self.search.match_count() > 0 {
                            // 跳到第一個匹配（不推進 current，避免 off-by-one）
                            if let Some((row, byte_col)) = self.search.first_match() {
                                self.jump_to_match(row, byte_col);
                                self.message = Some(format!(
                                    "Found {} matches (ESC to exit search mode)",
                                    self.search.match_count()
                                ));
                            }
                        } else {
                            self.message = Some(format!("No matches found for '{}'", query));
                            self.search_mode = false; // 沒有結果就關閉搜尋模式
                        }
                    }
                }
            }

            Command::FindNext => {
                if self.search_mode && self.search.match_count() > 0 {
                    if let Some((row, byte_col)) = self.search.next_match() {
                        self.jump_to_match(row, byte_col);
                        self.message = Some(format!(
                            "Match {}/{} (ESC to exit search mode)",
                            self.search.current_index() + 1,
                            self.search.match_count()
                        ));
                    }
                } else {
                    // 沒有搜尋模式時，執行 PageDown
                    return self.handle_command(Command::PageDown);
                }
            }

            Command::FindPrev => {
                if self.search_mode && self.search.match_count() > 0 {
                    if let Some((row, byte_col)) = self.search.prev_match() {
                        self.jump_to_match(row, byte_col);
                        self.message = Some(format!(
                            "Match {}/{} (ESC to exit search mode)",
                            self.search.current_index() + 1,
                            self.search.match_count()
                        ));
                    }
                } else {
                    // 沒有搜尋模式時，執行 PageUp
                    return self.handle_command(Command::PageUp);
                }
            }

            // 視圖控制
            Command::ToggleLineNumbers => {
                self.view.toggle_line_numbers();
                self.resync_cursor();
            }

            // 切換顯示模式（單行/多行）
            Command::ToggleDisplayMode => {
                self.view.toggle_display_mode();
                self.resync_cursor();
                self.message = Some(format!(
                    "Display Mode: {}",
                    self.view.get_display_mode_name()
                ));
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

                                // 插入新行（保留原行尾符號）
                                let body = line_content.trim_end_matches(['\n', '\r']);
                                let eol = &line_content[body.len()..];
                                let new_line_with_newline =
                                    format!("{}{}", new_line.trim_end_matches(['\n', '\r']), eol);
                                self.buffer.insert(line_start, &new_line_with_newline);
                            }
                        }

                        self.view.invalidate_cache();

                        // 保留選擇狀態，但改為整行範圍（舊欄位在改寫後已失效）
                        self.select_whole_lines(start_row, end_row);
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

                        // 插入新行（保留原行尾符號）
                        let body = line_content.trim_end_matches(['\n', '\r']);
                        let eol = &line_content[body.len()..];
                        let new_line_with_newline =
                            format!("{}{}", new_line.trim_end_matches(['\n', '\r']), eol);
                        self.buffer.insert(line_start, &new_line_with_newline);

                        self.view.invalidate_cache();

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

                        self.view.invalidate_cache();

                        // 保留選擇狀態，但改為整行範圍（舊欄位在改寫後已失效）
                        self.select_whole_lines(start_row, end_row);
                        self.cursor.row = start_row;
                        self.cursor.col = 0;
                        self.cursor.desired_visual_col = 0;
                    }
                } else {
                    // 單行：在光標位置插入 4 個空格
                    let pos = self.cursor.char_position(&self.buffer);
                    self.buffer.insert(pos, "    ");
                    self.view.invalidate_cache();
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

                        self.view.invalidate_cache();

                        // 保留選擇狀態，但改為整行範圍（舊欄位在改寫後已失效）
                        self.select_whole_lines(start_row, end_row);
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
                        self.view.invalidate_cache();
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

            // 編碼切換
            Command::ChangeEncoding => {
                if let Ok(Some(encoding_str)) =
                    crate::dialog::prompt("Change encoding to:", self.terminal.size())
                {
                    if let Some(encoding) = parse_encoding_label(&encoding_str) {
                        // 檢查是否有檔案路徑（區分已存在檔案和新建檔案）
                        if self.buffer.has_file_path() {
                            // 已存在的檔案：需要重新載入
                            if self.buffer.is_modified() {
                                // 有未保存的修改，顯示確認對話框
                                if let Ok(confirmed) = crate::dialog::confirm(
                                    "Unsaved changes will be lost. Continue?",
                                    self.terminal.size(),
                                ) {
                                    if confirmed {
                                        match self.buffer.reload_with_encoding(encoding) {
                                            Ok(_) => {
                                                // 重新載入成功，重置游標
                                                self.cursor.row = 0;
                                                self.cursor.col = 0;
                                                self.cursor.desired_visual_col = 0;
                                                self.cursor.visual_line_index = 0;
                                                self.view.invalidate_cache();
                                                self.message = Some(if self.buffer.is_lossy() {
                                                    lossy_warning(&self.buffer)
                                                } else {
                                                    format!(
                                                        "Encoding changed to {} (file reloaded)",
                                                        encoding.name()
                                                    )
                                                });
                                            }
                                            Err(e) => {
                                                self.message =
                                                    Some(format!("Failed to reload file: {}", e));
                                            }
                                        }
                                    }
                                }
                            } else {
                                // 沒有未保存的修改，直接重新載入
                                match self.buffer.reload_with_encoding(encoding) {
                                    Ok(_) => {
                                        self.cursor.row = 0;
                                        self.cursor.col = 0;
                                        self.cursor.desired_visual_col = 0;
                                        self.cursor.visual_line_index = 0;
                                        self.view.invalidate_cache();
                                        self.message = Some(if self.buffer.is_lossy() {
                                            lossy_warning(&self.buffer)
                                        } else {
                                            format!(
                                                "Encoding changed to {} (file reloaded)",
                                                encoding.name()
                                            )
                                        });
                                    }
                                    Err(e) => {
                                        self.message =
                                            Some(format!("Failed to reload file: {}", e));
                                    }
                                }
                            }
                        } else {
                            // 新建檔案：只設定編碼，不重新載入
                            self.buffer.change_encoding(encoding);
                            self.message = Some(format!(
                                "Encoding set to {} (will be used on save)",
                                encoding.name()
                            ));
                        }
                    } else {
                        self.message = Some(format!("Unsupported encoding: {}", encoding_str));
                    }
                }
            }

            // 切換語法高亮
            #[cfg(feature = "syntax-highlighting")]
            Command::ToggleSyntaxHighlight => {
                self.highlight_enabled = !self.highlight_enabled;
                self.message = Some(format!(
                    "Syntax Highlight: {}",
                    if self.highlight_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                ));
            }

            // 顯示幫助
            Command::ShowHelp => {
                // 保存當前終端狀態
                if let Err(e) = crate::dialog::show_help(self.terminal.size()) {
                    self.message = Some(format!("Failed to show help: {}", e));
                }
                // 重新繪製編輯器畫面
                self.view.invalidate_cache();
            }
        }

        Ok(())
    }

    fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    /// 跳躍檔案 1/10 的距離（Ctrl+PageUp/PageDown 及其選取版本共用）
    fn jump_tenth(&mut self, up: bool) {
        let total_lines = self.buffer.line_count();
        let jump_distance = total_lines.max(10) / 10; // 至少跳 1 行
        self.cursor.row = if up {
            self.cursor.row.saturating_sub(jump_distance)
        } else {
            self.cursor
                .row
                .saturating_add(jump_distance)
                .min(total_lines.saturating_sub(1))
        };
        self.cursor
            .set_position(&self.buffer, &self.view, self.cursor.row, self.cursor.col);
    }

    /// 剪切：複製選取（或整行）後刪除
    /// use_system: true 表示使用系統剪貼簿，false 表示僅使用內部剪貼簿
    fn do_cut(&mut self, use_system: bool) {
        let text = self.get_copy_text();
        self.set_clipboard_text(text, use_system);

        // 剪切後刪除內容
        if self.has_selection() {
            self.delete_selection();
        } else {
            // 記錄是否在最後一行
            let was_last_line = self.cursor.row == self.buffer.line_count() - 1;

            self.buffer.delete_line(self.cursor.row);
            self.view.invalidate_cache();

            // 如果刪除的是最後一行且不是唯一一行，光標上移
            if was_last_line && self.cursor.row > 0 {
                self.cursor.row -= 1;
            }

            // 確保光標在有效範圍內
            if self.cursor.row >= self.buffer.line_count() && self.buffer.line_count() > 0 {
                self.cursor.row = self.buffer.line_count() - 1;
            }

            self.cursor.col = 0;
            self.cursor.desired_visual_col = 0;
        }

        // 剪切後關閉選擇模式
        self.selection_mode = false;
    }

    /// 獲取要複製/剪切的文本
    /// 如果有選擇範圍，返回選擇的文本；否則返回當前整行（帶換行符）
    fn get_copy_text(&mut self) -> String {
        if self.has_selection() {
            self.line_copy = None;
            self.get_selected_text()
        } else {
            // 複製當前整行（完整內容，包括尾部空格和換行符）
            let line_text = self.buffer.get_line_full(self.cursor.row);
            // 確保以換行符結尾（用於識別整行貼上）
            let text = if line_text.ends_with('\n') {
                line_text
            } else {
                format!("{}\n", line_text)
            };
            self.line_copy = Some(text.replace("\r\n", "\n"));
            text
        }
    }

    /// 設置剪貼簿內容
    /// use_system: true 表示使用系統剪貼簿，false 表示僅使用內部剪貼簿
    fn set_clipboard_text(&mut self, text: String, use_system: bool) {
        if use_system {
            // 嘗試系統剪貼簿，失敗則回退到內部剪貼簿
            if self.clipboard.set_text(&text).is_err() {
                self.message = Some("System clipboard unavailable; copied internally".to_string());
            }
            self.internal_clipboard = text; // 同步到內部剪貼簿
        } else {
            // 僅使用內部剪貼簿
            self.internal_clipboard = text;
            self.message = Some("Copied (internal clipboard)".to_string());
        }
    }

    /// 獲取剪貼簿內容
    /// use_system: true 表示優先使用系統剪貼簿，false 表示僅使用內部剪貼簿
    fn get_clipboard_text(&mut self, use_system: bool) -> String {
        if use_system {
            // 嘗試從系統剪貼簿獲取，失敗則使用內部剪貼簿
            self.clipboard.get_text().unwrap_or_else(|_| {
                if self.internal_clipboard.is_empty() {
                    self.message =
                        Some("Nothing to paste (system clipboard unavailable)".to_string());
                    String::new()
                } else {
                    self.internal_clipboard.clone()
                }
            })
        } else {
            // 僅使用內部剪貼簿
            if self.internal_clipboard.is_empty() {
                self.message = Some("Nothing to paste (internal clipboard)".to_string());
                String::new()
            } else {
                self.internal_clipboard.clone()
            }
        }
    }

    /// 執行貼上操作
    fn paste_text(&mut self, text: String) {
        if text.is_empty() {
            return;
        }

        if self.has_selection() {
            self.delete_selection();
        }
        // 統一為 \n 計算游標，插入時轉為檔案的行尾符號
        let text = text.replace("\r\n", "\n");
        let eol = self.buffer.line_ending();
        let insert_text = if eol == "\n" {
            text.clone()
        } else {
            text.replace('\n', eol)
        };

        // 只有貼上 wedi 自己「複製整行」的內容才整行貼上；外部多行文字插入在游標處
        let is_whole_line = self.line_copy.as_deref() == Some(text.as_str());

        if is_whole_line {
            // 整行貼上：在光標所在行的開始處插入
            let line_start = self.buffer.line_to_char(self.cursor.row);
            self.buffer.insert(line_start, &insert_text);
            self.view.invalidate_cache();

            // 計算插入了多少行
            let inserted_lines = text.chars().filter(|&c| c == '\n').count();

            // 光標移動到被擠下去的原行首
            self.cursor.row += inserted_lines;
            self.cursor.col = 0;
            self.cursor.desired_visual_col = 0;
        } else {
            // 普通貼上：在光標位置插入
            let pos = self.cursor.char_position(&self.buffer);
            self.buffer.insert(pos, &insert_text);
            self.view.invalidate_cache();
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

    /// 版面改變後（換行模式、行號、視窗大小）重新同步游標的視覺行
    fn resync_cursor(&mut self) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        self.cursor.set_position(&self.buffer, &self.view, row, col);
    }

    /// 跳到搜尋匹配：Search 以行內 byte 位置記錄，游標需要字元欄位
    fn jump_to_match(&mut self, row: usize, byte_col: usize) {
        let line = self.buffer.get_line_content(row);
        let col = line
            .char_indices()
            .take_while(|&(i, _)| i < byte_col)
            .count();
        self.cursor.set_position(&self.buffer, &self.view, row, col);
    }

    /// 將選取範圍設為 start_row..=end_row 的整行
    fn select_whole_lines(&mut self, start_row: usize, end_row: usize) {
        self.selection = Some(Selection {
            start: (start_row, 0),
            end: (end_row, self.line_len(end_row)),
        });
    }

    /// 行的字元數（不含換行符）
    fn line_len(&self, row: usize) -> usize {
        self.buffer
            .get_line_content(row)
            .trim_end_matches(['\n', '\r'])
            .chars()
            .count()
    }

    /// 已排序且夾限在緩衝區範圍內的選取端點
    fn selection_bounds(&self) -> Option<((usize, usize), (usize, usize))> {
        let sel = self.selection?;
        let last_row = self.buffer.line_count().saturating_sub(1);
        let clamp = |(row, col): (usize, usize)| {
            let row = row.min(last_row);
            (row, col.min(self.line_len(row)))
        };
        let (a, b) = (clamp(sel.start), clamp(sel.end));
        Some((a.min(b), a.max(b)))
    }

    fn get_selected_text(&self) -> String {
        if let Some(((start_row, start_col), (end_row, end_col))) = self.selection_bounds() {
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
        if let Some(((start_row, start_col), (end_row, end_col))) = self.selection_bounds() {
            let start_pos = self.buffer.line_to_char(start_row) + start_col;
            let end_pos = self.buffer.line_to_char(end_row) + end_col;

            self.buffer.delete_range(start_pos, end_pos);
            self.view.invalidate_cache();

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

    /// 獲取語法高亮後的行：先依緩衝區的修改記錄讓快取失效，再從最近的檢查點解析
    #[cfg(feature = "syntax-highlighting")]
    pub fn get_highlighted_lines(
        &mut self,
        start_row: usize,
        end_row: usize,
    ) -> std::collections::HashMap<usize, String> {
        // 單一失效入口：涵蓋輸入、貼上、撤銷、重新載入等所有修改
        if let Some(row) = self.buffer.take_changed_from() {
            self.highlight_cache.invalidate_from(row);
        }
        let Some(ref engine) = self.highlight_engine else {
            return std::collections::HashMap::new();
        };
        let buffer = &self.buffer;
        self.highlight_cache.highlight_rows(
            engine,
            buffer.line_count(),
            start_row,
            end_row,
            |row| buffer.line(row).map(|line| line.to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wedi_core::keymap::Direction;

    /// 以固定尺寸的 Terminal 建立無頭編輯器，內容寫入暫存檔
    fn editor_with(dir: &TempDir, name: &str, bytes: &[u8]) -> Editor {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        let config = EncodingConfig {
            read_encoding: None,
            save_encoding: None,
        };
        #[cfg(feature = "syntax-highlighting")]
        let editor = Editor::with_terminal(
            Terminal::with_size((80, 24)),
            Some(&path),
            false,
            &config,
            None,
            None,
        );
        #[cfg(not(feature = "syntax-highlighting"))]
        let editor =
            Editor::with_terminal(Terminal::with_size((80, 24)), Some(&path), false, &config);
        editor.unwrap()
    }

    fn run(editor: &mut Editor, commands: Vec<Command>) {
        for command in commands {
            editor.handle_command(command).unwrap();
        }
    }

    fn text(editor: &Editor) -> String {
        (0..editor.buffer.line_count())
            .map(|i| editor.buffer.get_line_content(i))
            .collect()
    }

    #[test]
    fn test_headless_typing() {
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "a.txt", b"bc\n");
        run(&mut editor, vec![Command::Insert('a')]);
        assert_eq!(text(&editor), "abc\n");
        assert_eq!((editor.cursor.row, editor.cursor.col), (0, 1));
    }

    #[test]
    fn test_lossy_file_warns_and_needs_second_save() {
        // 稽核 F5：無效位元組被替換為 U+FFFD，開檔要警告，存檔需再確認一次
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "bad.txt", b"a\xff\n");
        let path = dir.path().join("bad.txt");
        assert!(editor.message.as_deref().unwrap().contains('\u{FFFD}'));
        run(&mut editor, vec![Command::Save]);
        assert_eq!(std::fs::read(&path).unwrap(), b"a\xff\n");
        run(&mut editor, vec![Command::Save]);
        assert_eq!(editor.message.as_deref(), Some("File saved"));
        // 中間夾其他命令則重新要求確認
        let mut editor = editor_with(&dir, "bad2.txt", b"a\xff\n");
        run(
            &mut editor,
            vec![Command::Save, Command::MoveEnd, Command::Save],
        );
        assert_eq!(
            std::fs::read(dir.path().join("bad2.txt")).unwrap(),
            b"a\xff\n"
        );
    }

    #[test]
    fn test_copy_after_unindent_does_not_panic() {
        // 稽核 R1：選取範圍在 Unindent 改寫行後仍保留舊欄位
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "sel.txt", b"    ab\nline2\n");
        run(
            &mut editor,
            vec![
                Command::MoveEnd,
                Command::ExtendSelection(Direction::Down),
                Command::Unindent,
                Command::CopyInternal,
            ],
        );
        assert_eq!(editor.internal_clipboard, "ab\nline2");
    }

    #[test]
    fn test_undo_clears_selection() {
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "u.txt", b"abcdef\n");
        run(
            &mut editor,
            vec![
                Command::MoveEnd,
                Command::Insert('g'),
                Command::ExtendSelection(Direction::Left),
                Command::Undo,
            ],
        );
        assert!(editor.selection.is_none());
        run(&mut editor, vec![Command::CopyInternal]);
    }

    #[test]
    fn test_find_next_lands_on_char_column() {
        // 稽核 R2：CJK 行上匹配的 byte 位置曾被當成字元欄位
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "s.txt", "中文abc\nxyz\n".as_bytes());
        editor.search.set_query("abc".to_string());
        editor.search.find_matches(&editor.buffer);
        editor.search_mode = true;
        run(&mut editor, vec![Command::FindNext, Command::Insert('X')]);
        assert_eq!(text(&editor), "中文Xabc\nxyz\n");
    }

    #[test]
    fn test_toggle_display_mode_resyncs_visual_line() {
        // 稽核 R6：切換換行模式後 visual_line_index 仍停在舊值
        let dir = TempDir::new().unwrap();
        let long = format!("{}\n", "x".repeat(250));
        let mut editor = editor_with(&dir, "w.txt", long.as_bytes());
        run(&mut editor, vec![Command::MoveEnd]);
        assert!(editor.cursor.visual_line_index > 0);
        run(&mut editor, vec![Command::ToggleDisplayMode]);
        assert!(!editor.view.wrap_mode);
        assert_eq!(editor.cursor.visual_line_index, 0);
    }

    #[test]
    fn test_crlf_join_and_edit_keep_crlf() {
        // 稽核 R3：CRLF 檔案 Backspace/Delete 一次合併；Enter、貼上、註解保留 CRLF
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "c.rs", b"ab\r\ncd\r\n");
        run(&mut editor, vec![Command::MoveDown, Command::Backspace]);
        assert_eq!(text(&editor), "abcd\r\n");
        assert_eq!((editor.cursor.row, editor.cursor.col), (0, 2));
        run(&mut editor, vec![Command::Insert('\n')]);
        assert_eq!(text(&editor), "ab\r\ncd\r\n");
        run(
            &mut editor,
            vec![Command::MoveUp, Command::MoveEnd, Command::Delete],
        );
        assert_eq!(text(&editor), "abcd\r\n");
        run(&mut editor, vec![Command::PasteText("x\ny".to_string())]);
        assert_eq!(text(&editor), "abx\r\nycd\r\n");
        run(&mut editor, vec![Command::ToggleComment]);
        assert_eq!(text(&editor), "abx\r\n// ycd\r\n");
    }

    #[test]
    fn test_comment_toggle_keeps_tabs_and_skips_unknown_types() {
        // 稽核 F12：Tab 縮排被改成空格；F13：未知類型誤用 #
        let dir = TempDir::new().unwrap();
        let src = b"all:\n\tcc -o a a.c\n";
        let mut editor = editor_with(&dir, "Makefile", src);
        run(&mut editor, vec![Command::MoveDown, Command::ToggleComment]);
        assert_eq!(text(&editor), "all:\n\t# cc -o a a.c\n");
        run(&mut editor, vec![Command::ToggleComment]);
        assert_eq!(text(&editor).as_bytes(), src);

        let mut editor = editor_with(&dir, "a.json", b"{}\n");
        run(&mut editor, vec![Command::ToggleComment]);
        assert_eq!(text(&editor), "{}\n");
        assert_eq!(
            editor.message.as_deref(),
            Some("No comment style for this file type")
        );
    }

    #[test]
    fn test_undo_groups_words_and_commands_and_tracks_save_point() {
        // 稽核 F17：每個字元一次撤銷；多行命令需 N–2N 次；回到存檔點不清除 [modified]
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "u.txt", b"a\nb\nc\n");
        let typing: Vec<Command> = "hi yo".chars().map(Command::Insert).collect();
        run(&mut editor, typing);
        assert_eq!(text(&editor), "hi yoa\nb\nc\n");
        run(&mut editor, vec![Command::Undo]);
        assert_eq!(text(&editor), "hi a\nb\nc\n");
        run(&mut editor, vec![Command::Undo]);
        assert_eq!(text(&editor), "a\nb\nc\n");
        assert!(!editor.buffer.is_modified());

        // 三行縮排：一次撤銷
        run(
            &mut editor,
            vec![
                Command::ExtendSelection(Direction::Down),
                Command::ExtendSelection(Direction::Down),
                Command::Indent,
            ],
        );
        assert_ne!(text(&editor), "a\nb\nc\n");
        run(&mut editor, vec![Command::Undo]);
        assert_eq!(text(&editor), "a\nb\nc\n");

        // 存檔後輸入再撤銷：回到存檔點清除 [modified]；重做則再次標記
        run(&mut editor, vec![Command::Insert('x'), Command::Save]);
        assert!(!editor.buffer.is_modified());
        run(&mut editor, vec![Command::Insert('y')]);
        assert!(editor.buffer.is_modified());
        run(&mut editor, vec![Command::Undo]);
        assert_eq!(text(&editor), "xa\nb\nc\n");
        assert!(!editor.buffer.is_modified());
        run(&mut editor, vec![Command::Redo]);
        assert!(editor.buffer.is_modified());
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn test_highlight_follows_undo() {
        // 稽核 F17：撤銷後高亮快取也要失效（先前只有部分編輯路徑清除快取）
        let dir = TempDir::new().unwrap();
        let body = "int a = 1;\nint b = 2;\nint c = 3;\n";
        let mut editor = editor_with(&dir, "a.c", body.as_bytes());
        let code = editor.get_highlighted_lines(0, 2);
        run(
            &mut editor,
            vec![Command::Insert('/'), Command::Insert('*')],
        );
        let commented = editor.get_highlighted_lines(0, 2);
        assert_ne!(commented[&2], code[&2]);
        run(&mut editor, vec![Command::Undo]);
        assert_eq!(text(&editor), body);
        assert_eq!(editor.get_highlighted_lines(0, 2), code);
    }

    #[test]
    fn test_external_paste_ending_in_newline_inserts_at_cursor() {
        // 稽核 F21：以換行結尾的外部貼上不應被當成整行貼上
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "a.txt", b"hello world\nsecond\n");
        run(&mut editor, vec![Command::MoveRight; 5]);
        run(
            &mut editor,
            vec![Command::PasteText("AA\nBB\n".to_string())],
        );
        assert_eq!(text(&editor), "helloAA\nBB\n world\nsecond\n");
        assert_eq!((editor.cursor.row, editor.cursor.col), (2, 0));
        // 自己複製的整行仍貼在行首
        let mut editor = editor_with(&dir, "b.txt", b"one\ntwo\n");
        run(&mut editor, vec![Command::CopyInternal, Command::MoveDown]);
        run(
            &mut editor,
            vec![Command::MoveRight, Command::PasteInternal],
        );
        assert_eq!(text(&editor), "one\none\ntwo\n");
    }

    #[test]
    fn test_quit_with_unsaved_changes_needs_two_presses() {
        let dir = TempDir::new().unwrap();
        let mut editor = editor_with(&dir, "a.txt", b"x\n");
        run(&mut editor, vec![Command::Insert('a'), Command::Quit]);
        assert!(!editor.should_quit);
        // 中間夾其他命令就要重新確認
        run(&mut editor, vec![Command::MoveEnd, Command::Quit]);
        assert!(!editor.should_quit);
        run(&mut editor, vec![Command::Quit]);
        assert!(editor.should_quit);
    }
}
