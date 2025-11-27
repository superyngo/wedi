/// 手動測試語法高亮功能
///
/// 執行：cargo run --example manual_highlight_test

#[cfg(feature = "syntax-highlighting")]
use wedi::highlight::{supports_true_color, HighlightConfig, HighlightEngine};

#[cfg(not(feature = "syntax-highlighting"))]
fn main() {
    println!("語法高亮功能未啟用！");
    println!("請使用: cargo run --features syntax-highlighting --example manual_highlight_test");
}

#[cfg(feature = "syntax-highlighting")]
fn main() {
    println!("=== wedi 語法高亮測試 ===\n");

    // 檢測真彩色支援
    let true_color = supports_true_color();
    println!(
        "終端真彩色支援: {}",
        if true_color { "是" } else { "否（256色）" }
    );

    // 建立高亮引擎
    let config = HighlightConfig::default();
    println!("使用主題: {}", config.theme);

    // 測試四種語言
    test_language("Rust", "test_highlight.rs");
    test_language("Python", "test_highlight.py");
    test_language("JavaScript", "test_highlight.js");
    test_language("Bash", "test_install.sh");

    println!("\n=== 測試完成 ===");
}

#[cfg(feature = "syntax-highlighting")]
fn test_language(lang_name: &str, file_path: &str) {
    use std::fs;
    use std::path::Path;

    println!("\n--- {} 語法高亮 ---", lang_name);

    // 建立引擎
    let config = HighlightConfig::default();
    let mut engine = HighlightEngine::new(Some(&config.theme), config.true_color)
        .expect("Failed to create highlight engine");

    // 設定檔案類型
    engine.set_file(Some(Path::new(file_path)));

    if !engine.is_enabled() {
        println!("⚠️ 語法檢測失敗，請檢查檔案 {}", file_path);
        return;
    }

    println!("檢測到語法: {}", engine.syntax_name().unwrap_or("Unknown"));

    // 讀取檔案
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️ 無法讀取檔案 {}: {}", file_path, e);
            return;
        }
    };

    // 建立高亮器
    let mut highlighter = engine
        .create_highlighter()
        .expect("Failed to create highlighter");

    // 逐行高亮並顯示
    for (i, line) in content.lines().enumerate() {
        // ⚠️ 重要：syntect 需要換行符才能正確解析語法狀態
        let line_with_newline = format!("{}\n", line);
        let highlighted = highlighter.highlight_line(&line_with_newline);
        // 移除尾部的換行符以避免多餘空行
        let highlighted = highlighted.trim_end_matches('\n');
        println!("{:3}: {}", i + 1, highlighted);
    }
}
