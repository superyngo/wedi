use anyhow::{Context, Result};
use ropey::{Rope, RopeSlice};
use std::fs;
use std::path::{Path, PathBuf};

use super::history::{Action, History};
use super::EncodingConfig;
use crate::debug_log;

pub struct RopeBuffer {
    rope: Rope,
    file_path: Option<PathBuf>,
    modified: bool,
    history: History,
    in_undo_redo: bool,                            // 防止在撤銷/重做時記錄歷史
    read_encoding: &'static encoding_rs::Encoding, // 讀取編碼
    save_encoding: &'static encoding_rs::Encoding, // 存檔編碼
}

impl RopeBuffer {
    pub fn new() -> Self {
        // 新建文件默认使用系统 ANSI 编码
        // 可通过 --dec 或 --en 参数覆盖
        let system_enc = Self::get_system_ansi_encoding();

        // Debug 模式：显示新建文件的默认编码
        if cfg!(debug_assertions) {
            eprintln!("[DEBUG] RopeBuffer::new()");
            eprintln!("[DEBUG]   System default encoding: {}", system_enc.name());
        }

        Self {
            rope: Rope::new(),
            file_path: None,
            modified: false,
            history: History::default(),
            in_undo_redo: false,
            read_encoding: system_enc,
            save_encoding: system_enc,
        }
    }

    /// 根據系統區域設置獲取 ANSI 編碼
    pub fn get_system_ansi_encoding() -> &'static encoding_rs::Encoding {
        // 跨平台編碼檢測策略
        // Windows: 使用 WinAPI 讀取 CodePage
        // Linux/macOS: 讀取 locale，解析 charset（大多是 UTF-8）
        // 若無法判斷 → fallback = UTF-8

        #[cfg(target_os = "windows")]
        {
            use winapi::um::consoleapi::{GetConsoleCP, GetConsoleOutputCP};
            use winapi::um::winnls::GetACP;

            // 檢查多個代碼頁來源
            let console_input_cp = unsafe { GetConsoleCP() };
            let console_output_cp = unsafe { GetConsoleOutputCP() };
            let system_acp = unsafe { GetACP() };

            if cfg!(debug_assertions) {
                eprintln!("[DEBUG] Detecting system encoding on Windows:");
                eprintln!(
                    "[DEBUG]   Console Input CP (GetConsoleCP): {}",
                    console_input_cp
                );
                eprintln!(
                    "[DEBUG]   Console Output CP (GetConsoleOutputCP): {}",
                    console_output_cp
                );
                eprintln!("[DEBUG]   System ANSI CP (GetACP): {}", system_acp);
            }

            // 優先使用控制台輸出代碼頁，如果是 0 則回退到系統 ANSI 代碼頁
            let cp = if console_output_cp != 0 {
                if cfg!(debug_assertions) {
                    eprintln!("[DEBUG]   Using Console Output CP: {}", console_output_cp);
                }
                console_output_cp
            } else {
                if cfg!(debug_assertions) {
                    eprintln!(
                        "[DEBUG]   Console CP is 0, using System ANSI CP: {}",
                        system_acp
                    );
                }
                system_acp
            };

            let encoding = match cp {
                65001 => {
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   Using UTF-8 (CP 65001)");
                    }
                    encoding_rs::UTF_8
                }
                936 => {
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   Using GBK (CP 936)");
                    }
                    encoding_rs::GBK
                }
                950 => {
                    // 中文(繁體) - Big5
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   Using Big5 (CP 950)");
                    }
                    if let Some(enc) = encoding_rs::Encoding::for_label(b"big5") {
                        enc
                    } else {
                        encoding_rs::UTF_8
                    }
                }
                932 => {
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   Using Shift_JIS (CP 932)");
                    }
                    encoding_rs::SHIFT_JIS
                }
                949 => {
                    // 韓文 - EUC-KR
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   Using EUC-KR (CP 949)");
                    }
                    if let Some(enc) = encoding_rs::Encoding::for_label(b"euc-kr") {
                        enc
                    } else {
                        encoding_rs::UTF_8
                    }
                }
                1252 => {
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   Using Windows-1252 (CP 1252)");
                    }
                    encoding_rs::WINDOWS_1252
                }
                _ => {
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   Unknown code page, using UTF-8 as fallback");
                    }
                    encoding_rs::UTF_8
                }
            };

            encoding
        }

        #[cfg(not(target_os = "windows"))]
        {
            use std::env;

            // 在 Unix-like 系統上，讀取 locale 設置
            // 優先級: LC_ALL > LC_CTYPE > LANG
            let locale_vars = ["LC_ALL", "LC_CTYPE", "LANG"];

            if cfg!(debug_assertions) {
                eprintln!("[DEBUG] Detecting system encoding on Unix-like system:");
            }

            for var in &locale_vars {
                if let Ok(locale) = env::var(var) {
                    if cfg!(debug_assertions) {
                        eprintln!("[DEBUG]   {} = {}", var, locale);
                    }

                    // 解析 locale 字符串，提取 charset 部分
                    // 格式如: zh_CN.UTF-8, en_US.UTF-8, zh_TW.Big5 等
                    if let Some(charset_start) = locale.find('.') {
                        let charset = &locale[charset_start + 1..];

                        if cfg!(debug_assertions) {
                            eprintln!("[DEBUG]   Detected charset: {}", charset);
                        }

                        match charset.to_uppercase().as_str() {
                            "UTF-8" => {
                                if cfg!(debug_assertions) {
                                    eprintln!("[DEBUG]   Using UTF-8");
                                }
                                return encoding_rs::UTF_8;
                            }
                            "GBK" | "GB2312" | "GB18030" => {
                                if cfg!(debug_assertions) {
                                    eprintln!("[DEBUG]   Using GBK");
                                }
                                return encoding_rs::GBK;
                            }
                            "BIG5" => {
                                if let Some(enc) = encoding_rs::Encoding::for_label(b"big5") {
                                    if cfg!(debug_assertions) {
                                        eprintln!("[DEBUG]   Using Big5");
                                    }
                                    return enc;
                                }
                            }
                            "SHIFT_JIS" | "SJIS" => {
                                if cfg!(debug_assertions) {
                                    eprintln!("[DEBUG]   Using Shift_JIS");
                                }
                                return encoding_rs::SHIFT_JIS;
                            }
                            "EUC-KR" => {
                                if let Some(enc) = encoding_rs::Encoding::for_label(b"euc-kr") {
                                    if cfg!(debug_assertions) {
                                        eprintln!("[DEBUG]   Using EUC-KR");
                                    }
                                    return enc;
                                }
                            }
                            _ => {
                                if cfg!(debug_assertions) {
                                    eprintln!("[DEBUG]   Unknown charset, continuing...");
                                }
                            } // 繼續檢查其他變數
                        }
                    }
                }
            }

            // 若無法從 locale 判斷，預設使用 UTF-8
            if cfg!(debug_assertions) {
                eprintln!("[DEBUG]   No valid locale found, using UTF-8 as fallback");
            }
            encoding_rs::UTF_8
        }
    }

    /// 檢測 Unicode BOM 或 UTF-8 編碼，返回編碼和 BOM 長度，如果都不是則返回 None
    fn detect_unicode(bytes: &[u8]) -> Option<(&'static encoding_rs::Encoding, usize)> {
        if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
            // UTF-8 BOM
            Some((encoding_rs::UTF_8, 3))
        } else if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
            // UTF-16LE BOM
            Some((encoding_rs::UTF_16LE, 2))
        } else if bytes.len() >= 2 && bytes[0..2] == [0xFE, 0xFF] {
            // UTF-16BE BOM
            Some((encoding_rs::UTF_16BE, 2))
        } else {
            // 沒有 BOM，檢查是否為有效的 UTF-8
            let (_, _, had_errors) = encoding_rs::UTF_8.decode(bytes);
            if !had_errors {
                // 如果是有效的 UTF-8，使用 UTF-8
                Some((encoding_rs::UTF_8, 0))
            } else {
                None
            }
        }
    }

    // pub fn from_file(path: &Path) -> Result<Self> {
    //     let encoding_config = EncodingConfig {
    //         read_encoding: None,
    //         save_encoding: None,
    //     };
    //     Self::from_file_with_encoding(path, &encoding_config)
    // }

    pub fn from_file_with_encoding(path: &Path, encoding_config: &EncodingConfig) -> Result<Self> {
        // 如果文件存在，讀取內容；否則創建空緩衝區
        let (rope, detected_encoding, modified) = if path.exists() {
            let bytes = fs::read(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;

            // 編碼處理邏輯 - 簡化版本
            // 優先級：BOM > 用戶指定 > 系統預設
            let (read_encoding, bom_length, detected_encoding_info) =
                if let Some((bom_encoding, bom_len)) = Self::detect_unicode(&bytes) {
                    // 檢測到 BOM 或 UTF-8，使用檢測到的編碼
                    let detected_info = if bom_len > 0 {
                        format!("BOM detected: {}", bom_encoding.name())
                    } else {
                        "UTF-8 detected (no BOM)".to_string()
                    };
                    (bom_encoding, bom_len, Some((detected_info, bom_encoding)))
                } else if let Some(specified_enc) = encoding_config.read_encoding {
                    // 沒有檢測到，使用用戶指定的編碼
                    (specified_enc, 0, None)
                } else {
                    // 沒有檢測到也沒有用戶指定，使用系統編碼
                    let system_enc = Self::get_system_ansi_encoding();
                    (system_enc, 0, None)
                };

            // Debug 模式：顯示編碼選擇信息
            // if cfg!(debug_assertions) {
            debug_log!("  File: {}", path.display());
            if let Some((detected_info, detected_enc)) = &detected_encoding_info {
                debug_log!("  Detected: {}", detected_info);
                if let Some(specified_enc) = encoding_config.read_encoding {
                    if detected_enc.name() != specified_enc.name() {
                        debug_log!("  User specified: {} (bypassed)", specified_enc.name());
                    }
                }
            } else if let Some(specified_enc) = encoding_config.read_encoding {
                debug_log!("  User specified: {}", specified_enc.name());
            } else {
                debug_log!("  System default: {}", read_encoding.name());
            }
            debug_log!("  Using decoding: {}", read_encoding.name());
            // }

            // 解碼為 UTF-8
            let (decoded, _, had_errors) = read_encoding.decode(&bytes[bom_length..]);
            if had_errors {
                eprintln!(
                    "[WARN] Encoding errors detected in file: {}",
                    path.display()
                );
            }

            (Rope::from_str(&decoded), read_encoding, false)
        } else {
            // 文件不存在，創建空緩衝區
            // 使用用戶指定編碼，否則使用系統默認編碼
            let encoding_to_use = encoding_config
                .read_encoding
                .unwrap_or_else(|| Self::get_system_ansi_encoding());

            if cfg!(debug_assertions) {
                eprintln!("[DEBUG]   File does not exist, creating new buffer");
                if encoding_config.read_encoding.is_some() {
                    eprintln!(
                        "[DEBUG]   Using user-specified encoding: {}",
                        encoding_to_use.name()
                    );
                } else {
                    eprintln!(
                        "[DEBUG]   Using system default encoding: {}",
                        encoding_to_use.name()
                    );
                }
            }

            (Rope::new(), encoding_to_use, true)
        };

        // 確定存檔編碼：優先級 --en > --dec > 實際讀取編碼
        let save_encoding = encoding_config
            .save_encoding
            .or(encoding_config.read_encoding)
            .unwrap_or(detected_encoding);

        // Debug 模式：顯示存檔編碼選擇信息
        // if cfg!(debug_assertions) {
        debug_log!("  Using encoding: {}", save_encoding.name());
        // }

        Ok(Self {
            rope,
            file_path: Some(path.to_path_buf()),
            modified,
            history: History::default(),
            in_undo_redo: false,
            read_encoding: detected_encoding,
            save_encoding,
        })
    }

    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let pos = pos.min(self.rope.len_chars());

        // 記錄到歷史
        if !self.in_undo_redo {
            self.history.push(Action::Insert {
                pos,
                text: ch.to_string(),
            });
        }

        self.rope.insert_char(pos, ch);
        self.modified = true;
    }

    pub fn insert(&mut self, pos: usize, text: &str) {
        let pos = pos.min(self.rope.len_chars());

        // 記錄到歷史
        if !self.in_undo_redo {
            self.history.push(Action::Insert {
                pos,
                text: text.to_string(),
            });
        }

        self.rope.insert(pos, text);
        self.modified = true;
    }

    pub fn delete_char(&mut self, pos: usize) {
        if pos < self.rope.len_chars() {
            // 獲取要刪除的字符
            let deleted_char = self.rope.char(pos).to_string();

            // 記錄到歷史
            if !self.in_undo_redo {
                self.history.push(Action::Delete {
                    pos,
                    text: deleted_char,
                });
            }

            self.rope.remove(pos..pos + 1);
            self.modified = true;
        }
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        if start < end && start < self.rope.len_chars() {
            let end = end.min(self.rope.len_chars());

            // 獲取要刪除的文本
            let deleted_text = self.rope.slice(start..end).to_string();

            // 記錄到歷史
            if !self.in_undo_redo {
                self.history.push(Action::DeleteRange {
                    start,
                    end,
                    text: deleted_text,
                });
            }

            self.rope.remove(start..end);
            self.modified = true;
        }
    }

    pub fn delete_line(&mut self, row: usize) {
        if row < self.line_count() {
            let start = self.rope.line_to_char(row);
            let end = if row + 1 < self.line_count() {
                self.rope.line_to_char(row + 1)
            } else {
                self.rope.len_chars()
            };

            // 獲取要刪除的行
            let deleted_line = self.rope.slice(start..end).to_string();

            // 記錄到歷史
            if !self.in_undo_redo {
                self.history.push(Action::DeleteRange {
                    start,
                    end,
                    text: deleted_line,
                });
            }

            self.rope.remove(start..end);
            self.modified = true;
        }
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, idx: usize) -> Option<RopeSlice<'_>> {
        if idx < self.line_count() {
            Some(self.rope.line(idx))
        } else {
            None
        }
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx.min(self.line_count()))
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx.min(self.rope.len_chars()))
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.file_path.clone() {
            if cfg!(debug_assertions) {
                eprintln!("[DEBUG] Saving file: {}", path.display());
                eprintln!("[DEBUG]   save_encoding: {}", self.save_encoding.name());
            }

            let contents = self.rope.to_string();
            // 使用指定編碼編碼內容
            let (encoded, _, had_errors) = self.save_encoding.encode(&contents);
            if had_errors {
                eprintln!(
                    "[WARN] Encoding errors occurred while saving file: {}",
                    path.display()
                );
            }
            std::fs::write(path, encoded)?;
            self.modified = false;

            if cfg!(debug_assertions) {
                eprintln!(
                    "[DEBUG]   File saved successfully with {} encoding",
                    self.save_encoding.name()
                );
            }

            Ok(())
        } else {
            anyhow::bail!("No file path set")
        }
    }

    #[allow(dead_code)]
    pub fn save_to(&mut self, path: &Path) -> Result<()> {
        let contents = self.rope.to_string();
        // 使用指定編碼編碼內容
        let (encoded, _, had_errors) = self.save_encoding.encode(&contents);
        if had_errors {
            eprintln!(
                "[WARN] Encoding errors occurred while saving file: {}",
                path.display()
            );
        }
        std::fs::write(path, encoded)?;
        self.modified = false;
        self.file_path = Some(path.to_path_buf());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_as(&mut self, path: &Path) -> Result<()> {
        let contents = self.rope.to_string();
        // 使用指定編碼編碼內容
        let (encoded, _, had_errors) = self.save_encoding.encode(&contents);
        if had_errors {
            eprintln!(
                "[WARN] Encoding errors occurred while saving file: {}",
                path.display()
            );
        }
        fs::write(path, encoded)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
        self.file_path = Some(path.to_path_buf());
        self.modified = false;
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    #[allow(dead_code)]
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]")
            .to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn get_line_content(&self, line_idx: usize) -> String {
        if let Some(line) = self.line(line_idx) {
            line.to_string()
        } else {
            String::new()
        }
    }

    /// 獲取完整行內容（包括尾部空格和換行符）
    pub fn get_line_full(&self, line_idx: usize) -> String {
        let line_start = self.line_to_char(line_idx);
        let line_end = if line_idx + 1 < self.line_count() {
            self.line_to_char(line_idx + 1)
        } else {
            self.rope.len_chars()
        };
        self.rope.slice(line_start..line_end).to_string()
    }

    // 撤銷/重做方法
    pub fn undo(&mut self) -> Option<usize> {
        if let Some(action) = self.history.undo() {
            self.in_undo_redo = true;

            let result_pos = match action {
                Action::Insert { pos, text } => {
                    // 撤銷插入 = 刪除
                    let char_count = text.chars().count();
                    self.rope.remove(pos..pos + char_count);
                    self.modified = true;
                    Some(pos)
                }
                Action::Delete { pos, text } => {
                    // 撤銷刪除 = 插入
                    self.rope.insert(pos, &text);
                    self.modified = true;
                    Some(pos)
                }
                Action::DeleteRange { start, text, .. } => {
                    // 撤銷範圍刪除 = 插入
                    self.rope.insert(start, &text);
                    self.modified = true;
                    Some(start)
                }
            };

            self.in_undo_redo = false;
            result_pos
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<usize> {
        if let Some(action) = self.history.redo() {
            self.in_undo_redo = true;

            let result_pos = match action {
                Action::Insert { pos, text } => {
                    // 重做插入
                    self.rope.insert(pos, &text);
                    self.modified = true;
                    Some(pos + text.chars().count())
                }
                Action::Delete { pos, text } => {
                    // 重做刪除
                    let char_count = text.chars().count();
                    self.rope.remove(pos..pos + char_count);
                    self.modified = true;
                    Some(pos)
                }
                Action::DeleteRange { start, end, .. } => {
                    // 重做範圍刪除
                    self.rope.remove(start..end);
                    self.modified = true;
                    Some(start)
                }
            };

            self.in_undo_redo = false;
            result_pos
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    #[allow(dead_code)]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    // 設置讀取編碼
    pub fn set_read_encoding(&mut self, encoding: &'static encoding_rs::Encoding) {
        self.read_encoding = encoding;
    }

    /// 設置存檔編碼
    pub fn set_save_encoding(&mut self, encoding: &'static encoding_rs::Encoding) {
        self.save_encoding = encoding;
        // 設置編碼後標記為已修改，因為編碼改變了
        self.modified = true;
    }

    // 獲取存檔編碼
    #[allow(dead_code)]
    pub fn save_encoding(&self) -> &'static encoding_rs::Encoding {
        self.save_encoding
    }

    /// 使用指定編碼重新載入檔案
    pub fn reload_with_encoding(&mut self, encoding: &'static encoding_rs::Encoding) -> Result<()> {
        if let Some(path) = &self.file_path.clone() {
            let encoding_config = EncodingConfig {
                read_encoding: Some(encoding),
                save_encoding: Some(encoding),
            };
            let new_buffer = Self::from_file_with_encoding(path, &encoding_config)?;

            // 重置內容但保留檔案路徑
            self.rope = new_buffer.rope;
            self.read_encoding = new_buffer.read_encoding;
            self.save_encoding = new_buffer.save_encoding;
            self.modified = false;
            self.history.clear(); // 清除 undo/redo 歷史

            Ok(())
        } else {
            anyhow::bail!("No file to reload")
        }
    }

    /// 為新建檔案設定編碼（無需重新載入）
    pub fn change_encoding(&mut self, encoding: &'static encoding_rs::Encoding) {
        self.read_encoding = encoding;
        self.save_encoding = encoding;
        // 不標記為已修改，因為只是改變未來的編碼設定
    }

    /// 檢查是否有檔案路徑
    pub fn has_file_path(&self) -> bool {
        self.file_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_utf8_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_utf8.txt");

        // 創建 UTF-8 文件（無 BOM）
        fs::write(&file_path, "Hello, 世界!").unwrap();

        let buffer = RopeBuffer::from_file_with_encoding(
            &file_path,
            &EncodingConfig {
                read_encoding: None,
                save_encoding: None,
            },
        )
        .unwrap();
        assert_eq!(buffer.save_encoding().name(), "UTF-8");
    }

    #[test]
    fn test_utf8_bom_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_utf8_bom.txt");

        // 創建 UTF-8 文件（有 BOM）
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice("Hello, 世界!".as_bytes());
        fs::write(&file_path, content).unwrap();

        let buffer = RopeBuffer::from_file_with_encoding(
            &file_path,
            &EncodingConfig {
                read_encoding: None,
                save_encoding: None,
            },
        )
        .unwrap();
        assert_eq!(buffer.save_encoding().name(), "UTF-8");
    }

    #[test]
    fn test_utf16le_bom_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_utf16le.txt");

        // 創建 UTF-16LE 文件（有 BOM）
        let mut content = vec![0xFF, 0xFE]; // UTF-16LE BOM
        let utf16_bytes: Vec<u8> = "Hello"
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        content.extend_from_slice(&utf16_bytes);
        fs::write(&file_path, content).unwrap();

        let buffer = RopeBuffer::from_file_with_encoding(
            &file_path,
            &EncodingConfig {
                read_encoding: None,
                save_encoding: None,
            },
        )
        .unwrap();
        assert_eq!(buffer.save_encoding().name(), "UTF-16LE");
    }

    #[test]
    fn test_gbk_encoding_save() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_gbk.txt");

        // 創建 buffer 並設置 GBK 編碼
        let mut buffer = RopeBuffer::new();
        buffer.set_save_encoding(encoding_rs::GBK);
        buffer.insert(0, "Hello, 世界!");

        // 保存文件
        buffer.save_to(&file_path).unwrap();

        // 讀取文件內容，應該是 GBK 編碼
        let saved_bytes = fs::read(&file_path).unwrap();
        let (decoded, _, _) = encoding_rs::GBK.decode(&saved_bytes);
        assert_eq!(decoded, "Hello, 世界!");
    }

    #[test]
    fn test_encoding_override() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_override.txt");

        // 創建 UTF-8 文件
        fs::write(&file_path, "Hello, 世界!").unwrap();

        // 讀取時指定 GBK 編碼
        let mut buffer = RopeBuffer::from_file_with_encoding(
            &file_path,
            &EncodingConfig {
                read_encoding: Some(encoding_rs::GBK),
                save_encoding: None,
            },
        )
        .unwrap();
        buffer.set_save_encoding(encoding_rs::GBK);

        // 保存時應該使用 GBK
        buffer.save_to(&file_path).unwrap();

        let saved_bytes = fs::read(&file_path).unwrap();
        let (decoded, _, _) = encoding_rs::GBK.decode(&saved_bytes);
        assert_eq!(decoded, "Hello, 世界!");
    }

    #[test]
    fn test_ansi_encoding_save() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_ansi.txt");

        // 創建 buffer 並設置 ANSI (Windows-1252) 編碼
        let mut buffer = RopeBuffer::new();
        buffer.set_save_encoding(encoding_rs::WINDOWS_1252);
        buffer.insert(0, "Hello, world! ©");

        // 保存文件
        buffer.save_to(&file_path).unwrap();

        // 讀取文件內容，應該是 Windows-1252 編碼
        let saved_bytes = fs::read(&file_path).unwrap();
        let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&saved_bytes);
        assert_eq!(decoded, "Hello, world! ©");
    }

    #[test]
    fn test_big5_encoding_save() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_big5.txt");

        // 獲取 Big5 編碼
        let big5_encoding = encoding_rs::Encoding::for_label(b"big5").unwrap();

        // 創建 buffer 並設置 Big5 編碼
        let mut buffer = RopeBuffer::new();
        buffer.set_save_encoding(big5_encoding);
        buffer.insert(0, "Hello, 世界!"); // 這裡會有一些字符無法用 Big5 表示

        // 保存文件
        buffer.save_to(&file_path).unwrap();

        // 讀取文件內容，應該是 Big5 編碼
        let saved_bytes = fs::read(&file_path).unwrap();
        let (decoded, _, _) = big5_encoding.decode(&saved_bytes);
        // 注意：Big5 無法表示簡體中文字符，所以會有替換字符
        assert!(decoded.contains("Hello"));
    }
}

impl Default for RopeBuffer {
    fn default() -> Self {
        Self::new()
    }
}
