/// 測試 syntect API
///
/// 執行方式：cargo run --example test_syntect --features syntax-highlighting
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

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

    // 4. 測試高亮
    println!("\n4. 測試高亮...");
    let mut highlighter = HighlightLines::new(rust_syntax, theme);

    let test_code = "fn main() {\n    println!(\"Hello, world!\");\n}";
    let lines: Vec<&str> = test_code.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let ranges = highlighter
            .highlight_line(line, &syntax_set)
            .expect("Highlight failed");
        println!("   第 {} 行: {} tokens", i + 1, ranges.len());
    }

    // 5. 測試跨行語法狀態
    println!("\n5. 測試跨行語法狀態（多行註解）...");
    let mut highlighter2 = HighlightLines::new(rust_syntax, theme);

    let multiline = vec!["/* 開始註解", "   中間", "   結束 */", "fn test() {}"];

    for (i, line) in multiline.iter().enumerate() {
        let ranges = highlighter2
            .highlight_line(line, &syntax_set)
            .expect("Highlight failed");
        println!("   第 {} 行: {} tokens", i + 1, ranges.len());
    }

    println!("\n=== 所有測試通過 ✓ ===");
}
