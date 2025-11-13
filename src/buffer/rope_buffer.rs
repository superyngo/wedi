use anyhow::{Context, Result};
use ropey::{Rope, RopeSlice};
use std::fs;
use std::path::{Path, PathBuf};

use super::history::{Action, History};

pub struct RopeBuffer {
    rope: Rope,
    file_path: Option<PathBuf>,
    modified: bool,
    history: History,
    in_undo_redo: bool,                       // 防止在撤銷/重做時記錄歷史
    encoding: &'static encoding_rs::Encoding, // 文件編碼
}

impl RopeBuffer {
    /// 特殊的 ANSI 編碼標記
    const ANSI_ENCODING_MARKER: &'static encoding_rs::Encoding = &encoding_rs::UTF_8; // 臨時使用 UTF-8 作為標記

    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            modified: false,
            history: History::default(),
            in_undo_redo: false,
            encoding: encoding_rs::UTF_8,
        }
    }

    /// 根據系統區域設置獲取 ANSI 編碼
    fn get_system_ansi_encoding() -> &'static encoding_rs::Encoding {
        // 在 Windows 中，ANSI 編碼取決於系統代碼頁
        // 這裡簡化處理：檢查環境變數或使用平台特定的邏輯

        #[cfg(target_os = "windows")]
        {
            use std::env;
            use std::process::Command;

            // 檢查 LANG 或 LC_ALL 環境變數
            if let Ok(lang) = env::var("LANG") {
                if lang.to_lowercase().contains("zh_tw") || lang.to_lowercase().contains("zh-hk") {
                    // 繁體中文 - Big5
                    if let Some(enc) = encoding_rs::Encoding::for_label(b"big5") {
                        return enc;
                    }
                } else if lang.to_lowercase().contains("zh_cn") {
                    // 簡體中文 - GBK
                    return encoding_rs::GBK;
                } else if lang.to_lowercase().contains("ja") {
                    // 日文 - Shift-JIS
                    return encoding_rs::SHIFT_JIS;
                }
            }

            // 檢查系統代碼頁 (如果可用)
            if let Ok(codepage) = env::var("ACP") {
                match codepage.as_str() {
                    "936" => return encoding_rs::GBK, // 中文(簡體)
                    "950" => {
                        // 中文(繁體)
                        if let Some(enc) = encoding_rs::Encoding::for_label(b"big5") {
                            return enc;
                        }
                    }
                    "932" => return encoding_rs::SHIFT_JIS, // 日文
                    "949" => {
                        // 韓文
                        if let Some(enc) = encoding_rs::Encoding::for_label(b"euc-kr") {
                            return enc;
                        }
                    }
                    "1252" => return encoding_rs::WINDOWS_1252, // 西歐
                    _ => {}
                }
            }

            // 嘗試使用 chcp 命令獲取當前代碼頁
            if let Ok(output) = Command::new("chcp").output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    // chcp 輸出格式如: "Active code page: 936"
                    if let Some(cp_start) = output_str.find(": ") {
                        let cp_str = &output_str[cp_start + 2..].trim();
                        if let Ok(cp) = cp_str.parse::<u32>() {
                            match cp {
                                936 => return encoding_rs::GBK, // 中文(簡體)
                                950 => {
                                    // 中文(繁體)
                                    if let Some(enc) = encoding_rs::Encoding::for_label(b"big5") {
                                        return enc;
                                    }
                                }
                                932 => return encoding_rs::SHIFT_JIS, // 日文
                                949 => {
                                    // 韓文
                                    if let Some(enc) = encoding_rs::Encoding::for_label(b"euc-kr") {
                                        return enc;
                                    }
                                }
                                1252 => return encoding_rs::WINDOWS_1252, // 西歐
                                _ => {}
                            }
                        }
                    }
                }
            }

            // 預設使用 GBK (因為用戶環境可能是中文)
            encoding_rs::GBK
        }

        #[cfg(not(target_os = "windows"))]
        {
            // 在非 Windows 系統上，ANSI 通常是 Latin-1
            encoding_rs::WINDOWS_1252
        }
    }

    /// 檢測文件編碼，基於 BOM
    fn detect_encoding(bytes: &[u8]) -> (&'static encoding_rs::Encoding, usize) {
        if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
            // UTF-8 BOM
            (encoding_rs::UTF_8, 3)
        } else if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
            // UTF-16LE BOM
            (encoding_rs::UTF_16LE, 2)
        } else if bytes.len() >= 2 && bytes[0..2] == [0xFE, 0xFF] {
            // UTF-16BE BOM
            (encoding_rs::UTF_16BE, 2)
        } else {
            // 無 BOM，預設 UTF-8
            (encoding_rs::UTF_8, 0)
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        Self::from_file_with_encoding(path, None)
    }

    pub fn from_file_with_encoding(
        path: &Path,
        encoding: Option<&'static encoding_rs::Encoding>,
    ) -> Result<Self> {
        // 如果文件存在，讀取內容；否則創建空緩衝區
        let (rope, detected_encoding, modified) = if path.exists() {
            let bytes = fs::read(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;

            // 編碼處理邏輯
            let (encoding_to_use, bom_length) = if let Some(specified_enc) = encoding {
                // 檢查是否是 ANSI 標記
                if std::ptr::eq(specified_enc, Self::ANSI_ENCODING_MARKER) {
                    // ANSI：先檢查 BOM，如果沒有 BOM 再使用系統 ANSI
                    let (bom_encoding, bom_len) = Self::detect_encoding(&bytes);
                    if bom_len > 0 {
                        // 有 BOM，使用 BOM 檢測到的編碼
                        (bom_encoding, bom_len)
                    } else {
                        // 無 BOM，使用系統 ANSI 編碼
                        (Self::get_system_ansi_encoding(), 0)
                    }
                } else if specified_enc.name() == "Big5"
                    || specified_enc.name() == "GBK"
                    || specified_enc.name() == "Windows-1252"
                    || specified_enc.name() == "Shift_JIS"
                {
                    // 對於系統 ANSI 編碼，先檢查 BOM
                    let (bom_encoding, bom_len) = Self::detect_encoding(&bytes);
                    if bom_len > 0 {
                        // 有 BOM，使用 BOM 檢測到的編碼
                        (bom_encoding, bom_len)
                    } else {
                        // 無 BOM，使用指定的編碼
                        (specified_enc, 0)
                    }
                } else {
                    // 非 ANSI 編碼，直接使用指定編碼
                    (specified_enc, 0)
                }
            } else {
                // 沒有指定編碼，檢測 BOM
                Self::detect_encoding(&bytes)
            };

            // 解碼為 UTF-8
            let (decoded, _, had_errors) = encoding_to_use.decode(&bytes[bom_length..]);
            if had_errors {
                log::warn!("Encoding errors detected in file: {}", path.display());
            }

            (Rope::from_str(&decoded), encoding_to_use, false)
        } else {
            // 文件不存在，創建空緩衝區
            let encoding_to_use = encoding.unwrap_or(encoding_rs::UTF_8);
            (Rope::new(), encoding_to_use, true)
        };

        Ok(Self {
            rope,
            file_path: Some(path.to_path_buf()),
            modified,
            history: History::default(),
            in_undo_redo: false,
            encoding: detected_encoding,
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
            let contents = self.rope.to_string();
            // 使用指定編碼編碼內容
            let (encoded, _, had_errors) = self.encoding.encode(&contents);
            if had_errors {
                log::warn!(
                    "Encoding errors occurred while saving file: {}",
                    path.display()
                );
            }
            std::fs::write(path, encoded)?;
            self.modified = false;
            Ok(())
        } else {
            anyhow::bail!("No file path set")
        }
    }

    #[allow(dead_code)]
    pub fn save_to(&mut self, path: &Path) -> Result<()> {
        let contents = self.rope.to_string();
        // 使用指定編碼編碼內容
        let (encoded, _, had_errors) = self.encoding.encode(&contents);
        if had_errors {
            log::warn!(
                "Encoding errors occurred while saving file: {}",
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
        let (encoded, _, had_errors) = self.encoding.encode(&contents);
        if had_errors {
            log::warn!(
                "Encoding errors occurred while saving file: {}",
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
                    self.rope.remove(pos..pos + text.len());
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
                    Some(pos + text.len())
                }
                Action::Delete { pos, text } => {
                    // 重做刪除
                    self.rope.remove(pos..pos + text.len());
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

    /// 設置文件編碼
    pub fn set_encoding(&mut self, encoding: &'static encoding_rs::Encoding) {
        self.encoding = encoding;
        // 設置編碼後標記為已修改，因為編碼改變了
        self.modified = true;
    }

    /// 獲取當前編碼
    pub fn encoding(&self) -> &'static encoding_rs::Encoding {
        self.encoding
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

        let buffer = RopeBuffer::from_file(&file_path).unwrap();
        assert_eq!(buffer.encoding().name(), "UTF-8");
    }

    #[test]
    fn test_utf8_bom_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_utf8_bom.txt");

        // 創建 UTF-8 文件（有 BOM）
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice("Hello, 世界!".as_bytes());
        fs::write(&file_path, content).unwrap();

        let buffer = RopeBuffer::from_file(&file_path).unwrap();
        assert_eq!(buffer.encoding().name(), "UTF-8");
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

        let buffer = RopeBuffer::from_file(&file_path).unwrap();
        assert_eq!(buffer.encoding().name(), "UTF-16LE");
    }

    #[test]
    fn test_gbk_encoding_save() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_gbk.txt");

        // 創建 buffer 並設置 GBK 編碼
        let mut buffer = RopeBuffer::new();
        buffer.set_encoding(encoding_rs::GBK);
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
        let mut buffer = RopeBuffer::from_file(&file_path).unwrap();
        buffer.set_encoding(encoding_rs::GBK);

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
        buffer.set_encoding(encoding_rs::WINDOWS_1252);
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
        buffer.set_encoding(big5_encoding);
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
