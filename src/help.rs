// 幫助訊息模組 - 提供統一的幫助文本

/// 幫助面板的一個區段：(區段標題, [(按鍵, 說明)])
pub type HelpSection = (&'static str, Vec<(&'static str, &'static str)>);

/// 獲取結構化的鍵盤快捷鍵資料（供 TUI 面板著色與對齊使用）
pub fn get_help_sections() -> Vec<HelpSection> {
    let mut sections: Vec<HelpSection> = Vec::new();

    sections.push((
        "Basic Editing",
        vec![
            ("Ctrl+S / Ctrl+W / Alt+W", "Save file"),
            ("Ctrl+Q", "Quit (press twice if modified)"),
            ("Ctrl+Z", "Undo"),
            ("Ctrl+Y", "Redo"),
            (
                "Backspace",
                "Delete character before cursor or selected text",
            ),
            ("Delete", "Delete character under cursor or selected text"),
            ("Ctrl+D", "Delete current line or selected lines"),
            ("Tab", "Indent (insert 4 spaces or indent selected lines)"),
            ("Shift+Tab", "Unindent (remove up to 4 leading spaces)"),
        ],
    ));

    let mut navigation = vec![
        ("Arrow Keys", "Move cursor"),
        ("Ctrl+Left/Home", "Move to line start"),
        ("Ctrl+Right/End", "Move to line end"),
        ("Ctrl+Up/Ctrl+Home", "Move to first line"),
        ("Ctrl+Down/Ctrl+End", "Move to last line"),
        ("Page Up/Down", "Scroll page (cycle matches in search mode)"),
        ("Ctrl+PageUp/Down", "Jump 1/10 of file"),
        ("Ctrl+G", "Go to line number"),
    ];
    #[cfg(feature = "mouse-support")]
    navigation.push(("Mouse Wheel", "Scroll up/down (moves cursor)"));
    sections.push(("Navigation", navigation));

    sections.push((
        "Selection",
        vec![
            (
                "Alt+S",
                "Toggle selection mode (for terminals without Shift support)",
            ),
            ("Shift+Arrows", "Select text"),
            ("Shift+Ctrl+Arrows", "Quick select to line/file boundaries"),
            ("Shift+Home/End", "Select to line boundaries"),
            ("Shift+Ctrl+Home/End", "Quick select to file boundaries"),
            ("Shift+PgUp/Dn", "Select page up/down"),
            ("Ctrl+A", "Select all"),
            (
                "ESC",
                "Dismiss one layer: message, then selection, then search mode",
            ),
        ],
    ));

    sections.push((
        "Clipboard",
        vec![
            ("Ctrl+C", "Copy (selection or current line)"),
            ("Ctrl+X", "Cut (selection or current line)"),
            ("Ctrl+V", "Paste"),
            ("Alt+C", "Internal Copy (selection or current line)"),
            ("Alt+X", "Internal Cut (selection or current line)"),
            ("Alt+V", "Internal Paste"),
        ],
    ));

    sections.push((
        "Search",
        vec![
            ("Ctrl+F", "Find text (with last search term pre-filled)"),
            (
                "Ctrl+N / F3 / PgDn",
                "Find next match (PageDown if no active search)",
            ),
            (
                "Ctrl+P / Shift+F3 / PgUp",
                "Find previous match (PageUp if no active search)",
            ),
        ],
    ));

    let mut code = vec![
        ("Ctrl+/ \\ K", "Toggle line comment"),
        ("Ctrl+L", "Toggle line numbers (& display mode)"),
        ("Ctrl+O", "Toggle display mode (wrap/scroll)"),
    ];
    #[cfg(feature = "syntax-highlighting")]
    code.push(("Ctrl+T", "Toggle syntax highlight"));
    sections.push(("Code", code));

    sections.push((
        "Other",
        vec![
            (
                "Ctrl+E",
                "Change file encoding (utf-8, gbk, big5, shift-jis, etc.)",
            ),
            ("Ctrl+H / F1", "Show this help"),
        ],
    ));

    sections.push((
        "Supported Comment Styles",
        vec![
            ("//", "Rust, C/C++, Java, JavaScript, TypeScript, Go, C#"),
            ("#", "Python, Shell, PowerShell, Ruby, YAML, TOML"),
            ("--", "SQL, Lua, Haskell"),
            ("REM", "Batch, CMD"),
            ("\"", "Vim"),
        ],
    ));

    sections
}

/// About 面板的結構化內容：(標籤, 內容)；標籤為空字串表示純文字行
pub fn get_about_entries() -> Vec<(&'static str, String)> {
    vec![
        ("", format!("wedi v{}", env!("CARGO_PKG_VERSION"))),
        ("", env!("CARGO_PKG_DESCRIPTION").to_string()),
        ("", String::new()),
        ("Author", "wen (superyngo)".to_string()),
        ("License", "MIT".to_string()),
        ("GitHub", "https://github.com/superyngo/wedi".to_string()),
        ("", String::new()),
        ("", "Privacy".to_string()),
        (
            "",
            "  wedi runs entirely on your machine. It does not collect,".to_string(),
        ),
        (
            "",
            "  store, or transmit any data. The clipboard feature only".to_string(),
        ),
        (
            "",
            "  accesses your system clipboard when you copy or paste.".to_string(),
        ),
    ]
}

/// 獲取鍵盤快捷鍵幫助內容（純文字，用於 --help 輸出）
pub fn get_keyboard_shortcuts() -> Vec<String> {
    let sections = get_help_sections();
    let key_width = sections
        .iter()
        .flat_map(|(_, items)| items.iter())
        .map(|(key, _)| key.chars().count())
        .max()
        .unwrap_or(0);

    let mut lines = Vec::new();
    for (i, (title, items)) in sections.iter().enumerate() {
        if i > 0 {
            lines.push(String::new());
        }
        lines.push(format!("{}:", title));
        for (key, desc) in items {
            lines.push(format!("  {:<width$}  {}", key, desc, width = key_width));
        }
    }
    lines
}

/// 打印完整的幫助訊息到標準輸出 (用於 --help)
#[allow(dead_code)]
pub fn print_help() {
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
    #[cfg(feature = "syntax-highlighting")]
    println!("    --theme <THEME>                    Set syntax highlighting theme");
    #[cfg(feature = "syntax-highlighting")]
    println!("    --list-themes                      List all available themes");
    #[cfg(feature = "syntax-highlighting")]
    println!("    -l, --language <LANG>              Set syntax highlighting language");
    #[cfg(feature = "syntax-highlighting")]
    println!("    --list-languages                   List all available languages");
    println!();
    println!("KEYBOARD SHORTCUTS:");
    println!();

    for line in get_keyboard_shortcuts() {
        println!("  {}", line);
    }
}
