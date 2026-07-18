// 幫助訊息模組 - 提供統一的幫助文本

/// 獲取鍵盤快捷鍵幫助內容
#[allow(clippy::vec_init_then_push)]
pub fn get_keyboard_shortcuts() -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("Basic Editing:".to_string());
    lines.push("  Ctrl+W / Alt+W      Save file".to_string());
    lines.push("  Ctrl+Q              Quit (press twice if modified)".to_string());
    lines.push("  Ctrl+Z              Undo".to_string());
    lines.push("  Ctrl+Y              Redo".to_string());
    lines.push("  Backspace           Delete character before cursor or selected text".to_string());
    lines.push("  Delete              Delete character under cursor or selected text".to_string());
    lines.push("  Ctrl+D              Delete current line or selected lines".to_string());
    lines.push(
        "  Tab                 Indent (insert 4 spaces or indent selected lines)".to_string(),
    );
    lines.push("  Shift+Tab           Unindent (remove up to 4 leading spaces)".to_string());
    lines.push(String::new());

    lines.push("Navigation:".to_string());
    lines.push("  Arrow Keys          Move cursor".to_string());
    lines.push("  Ctrl+Left/Home      Move to line start".to_string());
    lines.push("  Ctrl+Right/End      Move to line end".to_string());
    lines.push("  Ctrl+Up/Ctrl+Home   Move to first line".to_string());
    lines.push("  Ctrl+Down/Ctrl+End  Move to last line".to_string());
    lines.push("  Page Up/Down        Scroll page up/down".to_string());
    lines.push("  Ctrl+PageUp/Down    Jump 1/10 of file".to_string());
    lines.push("  Ctrl+G              Go to line number".to_string());
    #[cfg(feature = "mouse-support")]
    lines.push("  Mouse Wheel         Scroll up/down (moves cursor)".to_string());
    lines.push(String::new());

    lines.push("Selection:".to_string());
    lines.push(
        "  Ctrl+S / Alt+S      Toggle selection mode (for terminals without Shift support)"
            .to_string(),
    );
    lines.push("  Shift+Arrows        Select text".to_string());
    lines.push("  Shift+Ctrl+Arrows   Quick select to line/file boundaries".to_string());
    lines.push("  Shift+Home/End      Select to line boundaries".to_string());
    lines.push("  Shift+Ctrl+Home/End Quick select to file boundaries".to_string());
    lines.push("  Shift+PgUp/Dn       Select page up/down".to_string());
    lines.push("  Ctrl+A              Select all".to_string());
    lines.push(
        "  ESC                 Dismiss one layer: message, then selection, then search mode"
            .to_string(),
    );
    lines.push(String::new());

    lines.push("Clipboard:".to_string());
    lines.push("  Ctrl+C              Copy (selection or current line)".to_string());
    lines.push("  Ctrl+X              Cut (selection or current line)".to_string());
    lines.push("  Ctrl+V              Paste".to_string());
    lines.push("  Alt+C               Internal Copy (selection or current line)".to_string());
    lines.push("  Alt+X               Internal Cut (selection or current line)".to_string());
    lines.push("  Alt+V               Internal Paste".to_string());
    lines.push(String::new());

    lines.push("Search:".to_string());
    lines.push("  Ctrl+F                 Find text (with last search term pre-filled)".to_string());
    lines.push("  Ctrl+N                 Find next match (PageDown if no active search)".to_string());
    lines.push("  Ctrl+P                 Find previous match (PageUp if no active search)".to_string());
    lines.push(String::new());

    lines.push("Code:".to_string());
    lines.push("  Ctrl+/ \\ K         Toggle line comment".to_string());
    lines.push("  Ctrl+L              Toggle line numbers (& display mode)".to_string());
    lines.push("  Ctrl+O              Toggle display mode (wrap/scroll)".to_string());
    #[cfg(feature = "syntax-highlighting")]
    lines.push("  Ctrl+T              Toggle syntax highlight".to_string());
    lines.push(String::new());

    lines.push("Other:".to_string());
    lines.push(
        "  Ctrl+E              Change file encoding (utf-8, gbk, big5, shift-jis, etc.)"
            .to_string(),
    );
    lines.push("  Ctrl+H              Show this help".to_string());
    lines.push(String::new());

    lines.push("SUPPORTED COMMENT STYLES:".to_string());
    lines.push("  //  - Rust, C/C++, Java, JavaScript, TypeScript, Go, C#".to_string());
    lines.push("  #   - Python, Shell, PowerShell, Ruby, YAML, TOML".to_string());
    lines.push("  --  - SQL, Lua, Haskell".to_string());
    lines.push("  REM - Batch, CMD".to_string());
    lines.push("  \"   - Vim".to_string());

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
