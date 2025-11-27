/// 測試 syntect API，驗證 ParseState 存取方式
///
/// 執行方式：cargo run --example test_syntect
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

const SERIALIZED_SYNTAX_SET: &[u8] = include_bytes!("../assets/syntaxes.bin");

fn load_syntax_set() -> SyntaxSet {
    bincode::deserialize(SERIALIZED_SYNTAX_SET).expect("Failed to load syntax set")
}

fn main() {
    println!("=== syntect API 測試 ===\n");

    // 1. 載入語法集
    println!("1. 載入 syntaxes.bin...");
    let syntax_set = load_syntax_set();
    println!("   ✓ 成功載入 {} 種語法", syntax_set.syntaxes().len());

    // 2. 載入主題
    println!("\n2. 載入主題...");
    let theme_set = ThemeSet::load_defaults();
    let theme = theme_set
        .themes
        .get("base16-ocean.dark")
        .expect("Theme not found");
    println!("   ✓ 主題載入成功");

    // 3. 測試語法檢測
    println!("\n3. 測試語法檢測...");
    let rust_syntax = syntax_set
        .find_syntax_by_extension("rs")
        .expect("Rust syntax not found");
    println!("   ✓ 檢測到 Rust 語法：{}", rust_syntax.name);

    // 4. 測試高亮（重點：ParseState 存取）
    println!("\n4. 測試高亮與 ParseState 存取...");
    let mut highlighter = HighlightLines::new(rust_syntax, theme);

    let test_code = "fn main() {\n    println!(\"Hello, world!\");\n}";
    let lines: Vec<&str> = test_code.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        // 執行高亮
        let ranges = highlighter
            .highlight_line(line, &syntax_set)
            .expect("Highlight failed");

        // ✨ 關鍵測試：檢查 parse_state 是否可以存取
        println!("   第 {} 行高亮完成", i + 1);

        // 嘗試存取 parse_state（這是關鍵測試）
        test_parse_state_access(&highlighter);
    }

    println!("\n=== 所有測試通過 ✓ ===");
}

/// 測試 ParseState 的存取方式
fn test_parse_state_access(highlighter: &HighlightLines) {
    // 測試方式 1：直接存取 parse_state 欄位（如果是 pub）
    #[allow(dead_code)]
    fn test_direct_access(hl: &HighlightLines) {
        let _state = &hl.parse_state; // 測試是否可以直接存取
        println!("      ✓ ParseState 可以直接存取");
    }

    // 測試方式 2：嘗試 clone（如果實作了 Clone）
    #[allow(dead_code)]
    fn test_clone(hl: &HighlightLines) {
        let _state_clone = hl.parse_state.clone();
        println!("      ✓ ParseState 可以 clone");
    }

    // 執行測試
    test_direct_access(highlighter);
    test_clone(highlighter);
}
