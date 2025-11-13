mod buffer;
mod clipboard;
mod comment;
mod config;
mod cursor;
mod dialog;
mod editor;
mod highlight;
mod input;
mod search;
mod terminal;
mod utils;
mod view;

use anyhow::Result;
use clap::Parser;
use editor::Editor;
use std::path::PathBuf;

/// 特殊的 ANSI 編碼標記
static ANSI_ENCODING_MARKER: &encoding_rs::Encoding = &encoding_rs::UTF_8; // 臨時使用 UTF-8 作為標記

fn parse_encoding(encoding_str: &str) -> Result<Option<&'static encoding_rs::Encoding>> {
    match encoding_str.to_lowercase().as_str() {
        "utf-8" | "utf8" => Ok(Some(encoding_rs::UTF_8)),
        "utf-16le" | "utf16le" => Ok(Some(encoding_rs::UTF_16LE)),
        "utf-16be" | "utf16be" => Ok(Some(encoding_rs::UTF_16BE)),
        "gbk" | "cp936" => Ok(Some(encoding_rs::GBK)),
        "shift-jis" | "shift_jis" | "sjis" => Ok(Some(encoding_rs::SHIFT_JIS)),
        "big5" | "cp950" => {
            // Big5 編碼用於繁體中文
            if let Some(enc) = encoding_rs::Encoding::for_label(b"big5") {
                Ok(Some(enc))
            } else {
                anyhow::bail!("Big5 encoding not supported");
            }
        }
        "ansi" => {
            // 返回特殊的 ANSI 標記
            Ok(Some(ANSI_ENCODING_MARKER))
        }
        "cp1252" | "windows-1252" => Ok(Some(encoding_rs::WINDOWS_1252)),
        _ => {
            // 嘗試查找其他編碼
            if let Some(enc) = encoding_rs::Encoding::for_label(encoding_str.as_bytes()) {
                Ok(Some(enc))
            } else {
                anyhow::bail!("Unsupported encoding: {}", encoding_str);
            }
        }
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

#[derive(Parser, Debug)]
#[command(name = "wedi")]
#[command(author = "wen")]
#[command(version = "0.1.12")]
#[command(about = "A lightweight, easy-to-use console text editor.")]
#[command(long_about = "
wedi - A easy-to-use text editor

KEYBOARD SHORTCUTS:
  
  Basic Editing:
    Ctrl+W              Save file
    Ctrl+Q              Quit (press twice if modified)
    Ctrl+Z              Undo
    Ctrl+Y              Redo
    Backspace           Delete character before cursor or selected text
    Delete              Delete character under cursor or selected text
    Ctrl+D              Delete current line or selected lines
    Tab                 Indent (insert 4 spaces or indent selected lines)
    Shift+Tab           Unindent (remove up to 4 leading spaces)

  Navigation:
    Arrow Keys            Move cursor
    Ctrl+Left/Ctrl+H/Home Move to line start
    Ctrl+Right/Ctrl+E/End Move to line end
    Ctrl+Up/Ctrl+Home     Move to first line
    Ctrl+Down/Ctrl+End    Move to last line
    Page Up/Down          Scroll page up/down
    Ctrl+G                Go to line number
    
  Selection:
    Ctrl+S              Toggle selection mode (for terminals without Shift support)
    Shift+Arrows        Select text
    Shift+Ctrl+Arrows   Quick select to line/file boundaries
    Shift+Home/End      Select to line boundaries
    Shift+Ctrl+Home/End Quick select to file boundaries
    Shift+Ctrl+H/E      Quick select to line boundaries
    Shift+PgUp/Dn       Select page up/down
    Ctrl+A              Select all
    ESC                 Clear selection and messages

  Clipboard:
    Ctrl+C              Copy (selection or current line)
    Ctrl+X              Cut (selection or current line)
    Ctrl+V              Paste
    Alt+C               Internal Copy (selection or current line)
    Alt+X               Internal Cut (selection or current line)
    Alt+V               Internal Paste
    
  Search:
    Ctrl+F              Find text
    F3                  Find next match
    Shift+F3            Find previous match
    
  Code:
    Ctrl+/ \\ K         Toggle line comment
    Ctrl+L              Toggle line numbers

SUPPORTED COMMENT STYLES:
  //  - Rust, C/C++, Java, JavaScript, TypeScript, Go, C#
  #   - Python, Shell, PowerShell, Ruby, YAML, TOML
  --  - SQL, Lua, Haskell
  REM - Batch, CMD
  \"   - Vim
")]
struct Args {
    /// File to open or create (default: Untitled)
    #[arg(default_value = "Untitled")]
    file: PathBuf,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,

    /// File encoding (utf-8, utf-16le, utf-16be, gbk, shift-jis, big5, ansi, cp1252, etc.)
    #[arg(long, default_value = "utf-8")]
    encoding: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 初始化日誌
    utils::init_logger(args.debug);

    // 解析編碼
    let encoding = parse_encoding(&args.encoding)?;

    // 如果啟用了調試模式，打印編碼信息
    if args.debug {
        if let Some(enc) = encoding {
            eprintln!("Using encoding: {}", enc.name());
        } else {
            eprintln!("Using default encoding: UTF-8");
        }
    }

    // 創建並運行編輯器
    let mut editor = Editor::new(Some(&args.file), args.debug, encoding)?;

    // 設置 panic hook 以確保終端正常恢復
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = terminal::Terminal::exit_raw_mode();
        let _ = terminal::Terminal::show_cursor();
        original_hook(panic_info);
    }));

    editor.run()?;

    Ok(())
}
