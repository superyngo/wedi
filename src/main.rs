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
use editor::Editor;
use pico_args::Arguments;
use std::path::PathBuf;

fn parse_encoding(
    from_encoding: Option<&str>,
    to_encoding: Option<&str>,
) -> Result<EncodingConfig> {
    // 解析讀取編碼
    let read_encoding = if let Some(enc_str) = from_encoding {
        Some(parse_single_encoding(enc_str)?)
    } else {
        // 沒有指定讀取編碼自動檢測
        None
    };

    // 解析存檔編碼
    let save_encoding = if let Some(enc_str) = to_encoding {
        // 用戶指定了存檔編碼
        Some(parse_single_encoding(enc_str)?)
    } else if let Some(enc_str) = from_encoding {
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

#[derive(Debug)]
struct Args {
    file: PathBuf,
    debug: bool,
    from_encoding: Option<String>,
    to_encoding: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut pargs = Arguments::from_env();

        // 檢查是否有 --help
        if pargs.contains(["-h", "--help"]) {
            Self::print_help();
            std::process::exit(0);
        }

        // 檢查是否有 --version
        if pargs.contains(["-v", "--version"]) {
            Self::print_version();
            std::process::exit(0);
        }

        let debug = pargs.contains("--debug");

        // -e/--encoding 同時設定讀取和保存編碼
        let encoding = pargs.opt_value_from_str(["-e", "--encoding"])?;

        // -f/--from-encoding 和 -t/--to-encoding 可以覆蓋 -e 的設定
        let from_encoding = pargs
            .opt_value_from_str(["-f", "--from-encoding"])?
            .or(encoding.clone());
        let to_encoding = pargs
            .opt_value_from_str(["-t", "--to-encoding"])?
            .or(encoding);

        let file = pargs
            .free_from_str()
            .unwrap_or_else(|_| PathBuf::from("Untitled"));

        // 檢查未處理的參數
        let remaining = pargs.finish();
        if !remaining.is_empty() {
            eprintln!("Warning: unused arguments {:?}", remaining);
        }

        Ok(Self {
            file,
            debug,
            from_encoding,
            to_encoding,
        })
    }

    fn print_version() {
        println!("wedi {}", env!("CARGO_PKG_VERSION"));
    }

    fn print_help() {
        println!("wedi - A easy-to-use text editor");
        println!();
        println!("USAGE:");
        println!("    wedi [OPTIONS] [FILE]");
        println!();
        println!("OPTIONS:");
        println!("    -h, --help                         Show this help message");
        println!("    -v, --version                      Show version information");
        println!("    --debug                            Enable debug mode");
        println!("    -e, --encoding <ENCODING>          Encoding for both reading and saving");
        println!("                                       (utf-8, utf-16le, utf-16be, gbk, shift-jis, big5, cp1252, etc.)");
        println!("    -f, --from-encoding <ENCODING>     Encoding for reading files (overrides -e)");
        println!("    -t, --to-encoding <ENCODING>       Encoding for saving files (overrides -e)");
        println!();
        println!("KEYBOARD SHORTCUTS:");
        println!();
        println!("  Basic Editing:");
        println!("    Ctrl+W              Save file");
        println!("    Ctrl+Q              Quit (press twice if modified)");
        println!("    Ctrl+Z              Undo");
        println!("    Ctrl+Y              Redo");
        println!("    Backspace           Delete character before cursor or selected text");
        println!("    Delete              Delete character under cursor or selected text");
        println!("    Ctrl+D              Delete current line or selected lines");
        println!("    Tab                 Indent (insert 4 spaces or indent selected lines)");
        println!("    Shift+Tab           Unindent (remove up to 4 leading spaces)");
        println!();
        println!("  Navigation:");
        println!("    Arrow Keys          Move cursor");
        println!("    Ctrl+Left/Home      Move to line start");
        println!("    Ctrl+Right/End      Move to line end");
        println!("    Ctrl+Up/Ctrl+Home   Move to first line");
        println!("    Ctrl+Down/Ctrl+End  Move to last line");
        println!("    Page Up/Down        Scroll page up/down");
        println!("    Ctrl+PageUp/Down    Jump 1/10 of file");
        println!("    Ctrl+G              Go to line number");
        println!();
        println!("  Selection:");
        println!(
            "    Ctrl+S              Toggle selection mode (for terminals without Shift support)"
        );
        println!("    Shift+Arrows        Select text");
        println!("    Shift+Ctrl+Arrows   Quick select to line/file boundaries");
        println!("    Shift+Home/End      Select to line boundaries");
        println!("    Shift+Ctrl+Home/End Quick select to file boundaries");
        println!("    Shift+PgUp/Dn       Select page up/down");
        println!("    Ctrl+A              Select all");
        println!("    ESC                 Clear selection and messages");
        println!();
        println!("  Clipboard:");
        println!("    Ctrl+C              Copy (selection or current line)");
        println!("    Ctrl+X              Cut (selection or current line)");
        println!("    Ctrl+V              Paste");
        println!("    Alt+C               Internal Copy (selection or current line)");
        println!("    Alt+X               Internal Cut (selection or current line)");
        println!("    Alt+V               Internal Paste");
        println!();
        println!("  Search:");
        println!("    Ctrl+F              Find text");
        println!("    F3                  Find next match");
        println!("    F4                  Find previous match");
        println!();
        println!("  Code:");
        println!("    Ctrl+/ \\ K         Toggle line comment");
        println!("    Ctrl+L              Toggle line numbers");
        println!();
        println!("  Encoding:");
        println!(
            "    Ctrl+E              Change file encoding (utf-8, gbk, big5, shift-jis, etc.)"
        );
        println!();
        println!("SUPPORTED COMMENT STYLES:");
        println!("  //  - Rust, C/C++, Java, JavaScript, TypeScript, Go, C#");
        println!("  #   - Python, Shell, PowerShell, Ruby, YAML, TOML");
        println!("  --  - SQL, Lua, Haskell");
        println!("  REM - Batch, CMD");
        println!("  \"   - Vim");
    }
}

fn main() -> Result<()> {
    let args = Args::parse()?;

    // 設置全局調試模式（支持 release 版本通過 --debug 參數啟用）
    utils::set_debug_mode(args.debug);

    // 使用 debug_log! 宏輸出調試信息
    debug_log!("Starting wedi with file: {:?}", args.file);
    debug_log!("Debug mode enabled");

    let encoding_config =
        parse_encoding(args.from_encoding.as_deref(), args.to_encoding.as_deref())?;

    debug_log!(
        "Read encoding: {:?}",
        encoding_config.read_encoding.map(|e| e.name())
    );
    debug_log!(
        "Save encoding: {:?}",
        encoding_config.save_encoding.map(|e| e.name())
    );

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
