mod buffer;
mod clipboard;
mod comment;
mod config;
mod cursor;
mod dialog;
mod editor;
mod help;
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
    #[cfg(feature = "syntax-highlighting")]
    theme: Option<String>,
    #[cfg(feature = "syntax-highlighting")]
    language: Option<String>,
    #[cfg(feature = "syntax-highlighting")]
    #[allow(dead_code)]
    list_themes: bool,
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

        // 檢查是否有 --list-themes
        #[cfg(feature = "syntax-highlighting")]
        if pargs.contains("--list-themes") {
            Self::print_themes();
            std::process::exit(0);
        }

        // 檢查是否有 --list-languages
        #[cfg(feature = "syntax-highlighting")]
        if pargs.contains("--list-languages") {
            Self::print_languages();
            std::process::exit(0);
        }

        let debug = pargs.contains("--debug");

        // 解析主題參數
        #[cfg(feature = "syntax-highlighting")]
        let theme = pargs.opt_value_from_str("--theme")?;
        #[cfg(feature = "syntax-highlighting")]
        let language = pargs.opt_value_from_str(["-l", "--language"])?;
        #[cfg(feature = "syntax-highlighting")]
        let list_themes = false; // 已在上面處理

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
            #[cfg(feature = "syntax-highlighting")]
            theme,
            #[cfg(feature = "syntax-highlighting")]
            language,
            #[cfg(feature = "syntax-highlighting")]
            list_themes,
        })
    }

    fn print_version() {
        println!("wedi {}", env!("CARGO_PKG_VERSION"));
    }

    #[cfg(feature = "syntax-highlighting")]
    fn print_themes() {
        use highlight::HighlightEngine;

        println!("Available syntax highlighting themes:\n");

        let themes = HighlightEngine::available_themes();
        let mut sorted_themes = themes.clone();
        sorted_themes.sort();

        for (i, theme) in sorted_themes.iter().enumerate() {
            println!("  {:<2}. {}", i + 1, theme);
        }

        println!("\nUsage: wedi --theme <THEME_NAME> <FILE>");
        println!("Example: wedi --theme \"Solarized (dark)\" myfile.rs");
        println!("\nDefault theme: base16-eighties.dark");
    }

    #[cfg(feature = "syntax-highlighting")]
    fn print_languages() {
        use highlight::HighlightEngine;

        println!("Available syntax highlighting languages:\n");

        let mut syntaxes = HighlightEngine::available_syntaxes();
        syntaxes.sort();

        // 顯示為多欄格式
        let max_len = syntaxes.iter().map(|s| s.len()).max().unwrap_or(20);
        let col_width = max_len + 2;
        let cols = 3;

        for (i, syntax) in syntaxes.iter().enumerate() {
            print!("{:<width$}", syntax, width = col_width);
            if (i + 1) % cols == 0 {
                println!();
            }
        }
        if syntaxes.len() % cols != 0 {
            println!();
        }

        println!("\nTotal: {} languages", syntaxes.len());
        println!("\nUsage: wedi -l <LANGUAGE> <FILE>");
        println!("       wedi --language <LANGUAGE> <FILE>");
        println!("Example: wedi -l rust script");
        println!("         wedi --language python myfile.txt");
    }

    fn print_help() {
        help::print_help();
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
    let mut editor = Editor::new(
        Some(&args.file),
        args.debug,
        &encoding_config,
        #[cfg(feature = "syntax-highlighting")]
        args.theme.as_deref(),
        #[cfg(feature = "syntax-highlighting")]
        args.language.as_deref(),
    )?;

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
