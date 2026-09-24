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
                "Backspace / Ctrl+H",
                "Delete character before cursor or selected text",
            ),
            ("Delete", "Delete character under cursor or selected text"),
            ("Ctrl+D", "Delete current line or selected lines"),
            ("Tab", "Indent (insert 4 spaces or indent selected lines)"),
            ("Shift+Tab", "Unindent (remove up to 4 leading spaces)"),
            ("Enter / Ctrl+J / Ctrl+M", "Insert newline"),
        ],
    ));

    let navigation = vec![
        ("Arrow Keys", "Move cursor"),
        ("Ctrl+Left/Home", "Move to line start"),
        ("Ctrl+Right/End", "Move to line end"),
        ("Ctrl+Up/Ctrl+Home", "Move to first line"),
        ("Ctrl+Down/Ctrl+End", "Move to last line"),
        ("Page Up/Down", "Scroll page (cycle matches in search mode)"),
        ("Ctrl+PageUp/Down", "Jump 1/10 of file"),
        ("Ctrl+G", "Go to line number"),
        // 終端在 alternate screen 中把滾輪轉為方向鍵
        ("Mouse Wheel", "Scroll up/down (moves cursor)"),
    ];
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
            ("Shift+Ctrl+PgUp/Dn", "Select 1/10 of file up/down"),
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
            ("Ctrl+F", "Find from cursor (last search term pre-filled)"),
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

    #[cfg_attr(not(feature = "syntax-highlighting"), allow(unused_mut))]
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
            ("F1", "Show this help"),
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
pub fn print_help() {
    println!("wedi - A easy-to-use text editor");
    println!();
    println!("USAGE:");
    println!("    wedi [OPTIONS] [FILE]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help                         Show this help message");
    println!("    -v, --version                      Show version information");
    println!(
        "    --debug                            Debug mode; logs to wedi-debug.log in the temp dir"
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::collections::BTreeSet;
    use wedi_core::keymap::handle_key_event;

    type Key = (String, u8);

    const NONE: KeyModifiers = KeyModifiers::NONE;
    const CTRL: KeyModifiers = KeyModifiers::CONTROL;
    const ALT: KeyModifiers = KeyModifiers::ALT;
    const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

    fn key(code: KeyCode, m: KeyModifiers) -> Key {
        (format!("{:?}", code), m.bits())
    }

    fn arrows() -> [KeyCode; 4] {
        [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right]
    }

    /// 文件標籤 → 按鍵。標籤格式不規則（如 "Ctrl+Left/Home" 與 "Ctrl+PageUp/Down"），
    /// 故用明確對照表；未知標籤回傳 None 讓測試失敗，新增標籤時必須補上。
    fn label_keys(label: &str) -> Option<Vec<(KeyCode, KeyModifiers)>> {
        use KeyCode::*;
        let ctrl = |c| (Char(c), CTRL);
        let alt = |c| (Char(c), ALT);
        let mut shift_moves: Vec<_> = arrows().into_iter().map(|c| (c, SHIFT)).collect();
        shift_moves.extend([Home, End, PageUp, PageDown].map(|c| (c, SHIFT)));
        let ctrl_shift = |c| (c, CTRL | SHIFT);
        let keys = match label {
            "Ctrl+S / Ctrl+W / Alt+W" => vec![ctrl('s'), ctrl('w'), alt('w')],
            "Ctrl+Q" => vec![ctrl('q')],
            "Ctrl+Z" => vec![ctrl('z')],
            "Ctrl+Y" => vec![ctrl('y')],
            "Ctrl+Z / Ctrl+Y" => vec![ctrl('z'), ctrl('y')],
            "Backspace / Ctrl+H" => vec![(Backspace, NONE), ctrl('h')],
            "Delete" => vec![(Delete, NONE)],
            "Backspace (or Ctrl+H) / Delete" => vec![(Backspace, NONE), ctrl('h'), (Delete, NONE)],
            "Ctrl+D" => vec![ctrl('d')],
            "Tab" => vec![(Tab, NONE)],
            "Shift+Tab" => vec![(BackTab, SHIFT), (Tab, SHIFT)],
            "Tab / Shift+Tab" => vec![(Tab, NONE), (BackTab, SHIFT), (Tab, SHIFT)],
            "Enter / Ctrl+J / Ctrl+M" | "Enter, Ctrl+J, Ctrl+M" => {
                vec![(Enter, NONE), ctrl('j'), ctrl('m')]
            }
            "Arrow Keys" | "Arrows" | "Mouse Wheel" | "Mouse wheel" => {
                arrows().into_iter().map(|c| (c, NONE)).collect()
            }
            "Ctrl+Left/Home" | "Home / Ctrl+Left" => vec![(Left, CTRL), (Home, NONE)],
            "Ctrl+Right/End" | "End / Ctrl+Right" => vec![(Right, CTRL), (End, NONE)],
            "Ctrl+Up/Ctrl+Home" | "Ctrl+Up / Ctrl+Home" => vec![(Up, CTRL), (Home, CTRL)],
            "Ctrl+Down/Ctrl+End" | "Ctrl+Down / Ctrl+End" => vec![(Down, CTRL), (End, CTRL)],
            "Page Up/Down" | "PageUp / PageDown" => vec![(PageUp, NONE), (PageDown, NONE)],
            "Ctrl+PageUp/Down" | "Ctrl+PageUp / Ctrl+PageDown" => {
                vec![(PageUp, CTRL), (PageDown, CTRL)]
            }
            "Ctrl+G" => vec![ctrl('g')],
            "Alt+S" => vec![alt('s')],
            "Shift+Arrows" => arrows().into_iter().map(|c| (c, SHIFT)).collect(),
            "Shift+Ctrl+Arrows" => arrows().into_iter().map(ctrl_shift).collect(),
            "Shift+Home/End" => vec![(Home, SHIFT), (End, SHIFT)],
            "Shift+Ctrl+Home/End" => vec![ctrl_shift(Home), ctrl_shift(End)],
            "Shift+PgUp/Dn" => vec![(PageUp, SHIFT), (PageDown, SHIFT)],
            "Shift+Ctrl+PgUp/Dn" => vec![ctrl_shift(PageUp), ctrl_shift(PageDown)],
            "Shift+movement" => {
                let mut v = shift_moves;
                v.extend(arrows().map(ctrl_shift));
                v.extend([Home, End, PageUp, PageDown].map(ctrl_shift));
                v
            }
            "Ctrl+A" => vec![ctrl('a')],
            "ESC" => vec![(Esc, NONE)],
            "Ctrl+C" => vec![ctrl('c')],
            "Ctrl+X" => vec![ctrl('x')],
            "Ctrl+V" => vec![ctrl('v')],
            "Alt+C" => vec![alt('c')],
            "Alt+X" => vec![alt('x')],
            "Alt+V" => vec![alt('v')],
            "Ctrl+C / Ctrl+X / Ctrl+V" => vec![ctrl('c'), ctrl('x'), ctrl('v')],
            "Alt+C / Alt+X / Alt+V" => vec![alt('c'), alt('x'), alt('v')],
            "Ctrl+F" => vec![ctrl('f')],
            "Ctrl+N / F3 / PgDn" | "Ctrl+N / F3 / PageDown" => {
                vec![ctrl('n'), (F(3), NONE), (PageDown, NONE)]
            }
            "Ctrl+P / Shift+F3 / PgUp" | "Ctrl+P / Shift+F3 / PageUp" => {
                vec![ctrl('p'), (F(3), SHIFT), (PageUp, NONE)]
            }
            "Ctrl+/ \\ K" | "Ctrl+/ / Ctrl+\\\\ / Ctrl+K" => vec![ctrl('/'), ctrl('\\'), ctrl('k')],
            "Ctrl+L" => vec![ctrl('l')],
            "Ctrl+O" => vec![ctrl('o')],
            // 未編入語法高亮時 Ctrl+T 沒有綁定，只有 KEYMAP.md 註明需要該功能
            "Ctrl+T" if cfg!(feature = "syntax-highlighting") => vec![ctrl('t')],
            "Ctrl+T" => vec![],
            "Ctrl+E" => vec![ctrl('e')],
            "F1" => vec![(F(1), NONE)],
            _ => return None,
        };
        Some(keys)
    }

    fn keys_of(labels: &[&str], source: &str) -> BTreeSet<Key> {
        let mut set = BTreeSet::new();
        for label in labels {
            let keys = label_keys(label).unwrap_or_else(|| {
                panic!("{source}: unknown label {label:?}; add it to label_keys")
            });
            for (code, m) in keys {
                let bound = handle_key_event(KeyEvent::new(code, m), false);
                assert!(
                    bound.is_some(),
                    "{source}: {label:?} documents {code:?}+{m:?}, which is unbound"
                );
                set.insert(key(code, m));
            }
        }
        set
    }

    fn help_keys() -> BTreeSet<Key> {
        let labels: Vec<&str> = get_help_sections()
            .into_iter()
            .filter(|(title, _)| *title != "Supported Comment Styles")
            .flat_map(|(_, rows)| rows.into_iter().map(|(k, _)| k))
            .collect();
        keys_of(&labels, "help panel")
    }

    fn keymap_md_keys() -> BTreeSet<Key> {
        let md = include_str!("../docs/reference/KEYMAP.md");
        let labels: Vec<&str> = md
            .split("\n## ")
            .filter(|s| !s.starts_with("Comment styles"))
            .flat_map(|s| s.lines())
            .filter(|l| l.starts_with("| ") && !l.starts_with("| Key |"))
            .filter_map(|l| l.split(" | ").next())
            .map(|k| k.trim_start_matches("| "))
            .collect();
        keys_of(&labels, "KEYMAP.md")
    }

    #[test]
    fn test_help_panel_and_keymap_md_agree() {
        // 稽核 B2：說明面板與 KEYMAP.md 列出同一組按鍵，且每個都有綁定
        let help = help_keys();
        let md = keymap_md_keys();
        let only_help: Vec<_> = help.difference(&md).collect();
        let only_md: Vec<_> = md.difference(&help).collect();
        assert!(
            only_help.is_empty() && only_md.is_empty(),
            "only in help: {only_help:?}; only in KEYMAP.md: {only_md:?}"
        );
    }

    #[test]
    fn test_every_binding_is_documented() {
        // 稽核 B2：列舉按鍵組合，凡有綁定者都必須出現在說明面板
        let documented = help_keys();
        let mut codes: Vec<KeyCode> = ('a'..='z')
            .chain('0'..='9')
            .chain("/\\[];',.-=`".chars())
            .map(KeyCode::Char)
            .collect();
        codes.extend((1..=12).map(KeyCode::F));
        codes.extend(arrows());
        codes.extend([
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Tab,
            KeyCode::Insert,
        ]);
        let mut undocumented = Vec::new();
        for code in codes {
            for m in [NONE, CTRL, ALT, SHIFT, CTRL | SHIFT, ALT | SHIFT] {
                // 一般輸入（無修飾或 Shift 的字元）不是快捷鍵
                if matches!(code, KeyCode::Char(_)) && (m == NONE || m == SHIFT) {
                    continue;
                }
                if handle_key_event(KeyEvent::new(code, m), false).is_some()
                    && !documented.contains(&key(code, m))
                {
                    undocumented.push(format!("{code:?}+{m:?}"));
                }
            }
        }
        // Enter / Backspace / Delete / Esc / BackTab 刻意接受任何修飾鍵，不另列舉
        assert!(
            undocumented.is_empty(),
            "bound but not in help panel: {undocumented:?}"
        );
    }
}
