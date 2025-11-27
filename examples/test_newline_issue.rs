/// 測試換行符對語法高亮的影響
///
/// 執行：cargo run --example test_newline_issue

#[cfg(feature = "syntax-highlighting")]
use wedi::highlight::{HighlightConfig, HighlightEngine};

#[cfg(not(feature = "syntax-highlighting"))]
fn main() {
    println!("語法高亮功能未啟用！");
}

#[cfg(feature = "syntax-highlighting")]
fn main() {
    use std::path::Path;

    println!("=== 測試換行符對語法高亮的影響 ===\n");

    let config = HighlightConfig::default();
    let mut engine = HighlightEngine::new(Some(&config.theme), config.true_color)
        .expect("Failed to create engine");

    // 測試 Bash 語法
    engine.set_file(Some(Path::new("test.sh")));

    let test_code = vec![
        "#!/bin/bash",
        "# This is a comment",
        "echo \"Hello World\"",
        "if [ -f file.txt ]; then",
        "    cat file.txt",
        "fi",
    ];

    println!("測試 1: 無換行符（當前實作）");
    println!("{}", "-".repeat(60));
    let mut highlighter1 = engine.create_highlighter().unwrap();
    for (i, line) in test_code.iter().enumerate() {
        let highlighted = highlighter1.highlight_line(line);
        println!("{:2}: {}", i + 1, highlighted);
    }

    println!("\n測試 2: 有換行符（cate 方案）");
    println!("{}", "-".repeat(60));
    let mut highlighter2 = engine.create_highlighter().unwrap();
    for (i, line) in test_code.iter().enumerate() {
        let line_with_newline = format!("{}\n", line);
        let highlighted = highlighter2.highlight_line(&line_with_newline);
        println!("{:2}: {}", i + 1, highlighted);
    }

    println!("\n=== 結論 ===");
    println!("比較兩者的輸出，看看顏色是否有差異");
}
