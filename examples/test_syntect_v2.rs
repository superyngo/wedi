/// 測試 syntect 的替代方案（不使用 parse_state）
///
/// 執行方式：cargo run --example test_syntect_v2
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

const SERIALIZED_SYNTAX_SET: &[u8] = include_bytes!("../assets/syntaxes.bin");

fn load_syntax_set() -> SyntaxSet {
    bincode::deserialize(SERIALIZED_SYNTAX_SET).expect("Failed to load syntax set")
}

fn main() {
    println!("=== syntect 替代方案測試 ===\n");

    let syntax_set = load_syntax_set();
    let theme_set = ThemeSet::load_defaults();
    let theme = theme_set.themes.get("base16-ocean.dark").unwrap();
    let rust_syntax = syntax_set.find_syntax_by_extension("rs").unwrap();

    println!("語法集載入：{} 種語法", syntax_set.syntaxes().len());
    println!("測試語法：{}\n", rust_syntax.name);

    // 測試多行程式碼（包含跨行註解）
    let test_code = r#"fn main() {
    /* 這是一個
       跨行註解 */
    println!("Hello, world!");
    let x = 42; // 行末註解
}"#;

    println!("方案 1: 每次渲染都建立新的 HighlightLines（簡單但效能較差）");
    println!("---------------------------------------------------------------");

    for (i, line) in test_code.lines().enumerate() {
        // 每行都建立新的 highlighter，從頭開始解析到當前行
        let mut highlighter = HighlightLines::new(rust_syntax, theme);

        // 需要先處理前面所有行，才能正確處理當前行的狀態
        for prev_line in test_code.lines().take(i) {
            let _ = highlighter.highlight_line(prev_line, &syntax_set);
        }

        // 處理當前行
        let ranges = highlighter.highlight_line(line, &syntax_set).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        println!("{:2}: {}", i + 1, escaped);
    }

    println!("\n方案 2: 單個 HighlightLines 循序處理所有行（推薦）");
    println!("---------------------------------------------------------------");

    let mut highlighter = HighlightLines::new(rust_syntax, theme);
    for (i, line) in test_code.lines().enumerate() {
        let ranges = highlighter.highlight_line(line, &syntax_set).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        println!("{:2}: {}", i + 1, escaped);
    }

    println!("\n結論：");
    println!("✓ HighlightLines 內部維護狀態，循序處理即可");
    println!("✓ 快取策略：快取「已高亮的字串」而非 ParseState");
    println!("✓ 編輯時：從修改行開始重新高亮到可見區域結束");
}
