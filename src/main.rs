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
use buffer::EncodingConfig;
use clap::Parser;
use editor::Editor;
use std::path::PathBuf;

fn parse_encoding(dec: Option<&str>, en: Option<&str>) -> Result<EncodingConfig> {
    // 解析讀取編碼
    let read_encoding = if let Some(enc_str) = dec {
        Some(parse_single_encoding(enc_str)?)
    } else {
        // 沒有指定讀取編碼自動檢測
        None
    };

    // 解析存檔編碼
    let save_encoding = if let Some(enc_str) = en {
        // 用戶指定了存檔編碼
        Some(parse_single_encoding(enc_str)?)
    } else if let Some(enc_str) = dec {
        // 沒有指定存檔編碼，但有讀取編碼，使用讀取編碼
        Some(parse_single_encoding(enc_str)?)
    } else {
        // 都沒有指定，存檔編碼將在讀取後動態決定
        None
    };

    Ok(EncodingConfig {
        read_encoding,
        save_encoding,
    })
}

fn parse_single_encoding(enc_str: &str) -> Result<&'static encoding_rs::Encoding> {
    match enc_str.to_lowercase().as_str() {
        "utf-8" | "utf8" => Ok(encoding_rs::UTF_8),
        "utf-16le" | "utf16le" => Ok(encoding_rs::UTF_16LE),
        "utf-16be" | "utf16be" => Ok(encoding_rs::UTF_16BE),
        "gbk" | "cp936" => Ok(encoding_rs::GBK),
        "shift-jis" | "shift_jis" | "sjis" => Ok(encoding_rs::SHIFT_JIS),
        "big5" | "cp950" => {
            // Big5 編碼用於繁體中文
            if let Some(enc) = encoding_rs::Encoding::for_label(b"big5") {
                Ok(enc)
            } else {
                anyhow::bail!("Big5 encoding not supported");
            }
        }
        "cp1252" | "windows-1252" => Ok(encoding_rs::WINDOWS_1252),
        _ => {
            // 嘗試查找其他編碼
            if let Some(enc) = encoding_rs::Encoding::for_label(enc_str.as_bytes()) {
                Ok(enc)
            } else {
                anyhow::bail!("Unsupported encoding: {}", enc_str);
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "wedi")]
#[command(author = "wen")]
#[command(version = "0.1.13")]
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

    /// Decode encoding for reading files (utf-8, utf-16le, utf-16be, gbk, shift-jis, big5, cp1252, etc.)
    /// If not specified, uses automatic detection or system default
    #[arg(long)]
    dec: Option<String>,

    /// Encode encoding for saving files (utf-8, utf-16le, utf-16be, gbk, shift-jis, big5, cp1252, etc.)
    /// If not specified, uses --dec encoding or the encoding used for reading
    #[arg(long)]
    en: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 初始化日誌
    utils::init_logger(args.debug);

    // 解析編碼
    let encoding_config = parse_encoding(args.dec.as_deref(), args.en.as_deref())?;

    // 創建並運行編輯器
    let mut editor = Editor::new(Some(&args.file), args.debug, &encoding_config)?;

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
