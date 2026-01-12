// 範例：使用 wedi-core 的基本功能
// 展示如何使用 RopeBuffer、Cursor 和 Keymap

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use wedi_core::{Cursor, Keymap, RopeBuffer};

fn main() {
    println!("=== wedi-core 基本使用範例 ===\n");

    // 1. 建立文字緩衝區
    let mut buffer = RopeBuffer::new();
    buffer.insert(0, "Hello, World!\n");
    buffer.insert(buffer.len_chars(), "This is wedi-core.");

    println!("初始內容:");
    for i in 0..buffer.line_count() {
        println!("{}", buffer.get_line_content(i));
    }

    // 2. 使用游標
    let cursor = Cursor::new();
    println!("\n游標位置: ({}, {})", cursor.row, cursor.col);

    // 3. 編輯文字
    buffer.insert_char(13, '!');
    println!("\n插入字元後:");
    for i in 0..buffer.line_count() {
        println!("{}", buffer.get_line_content(i));
    }

    // 4. 使用快捷鍵對映
    let keymap = Keymap::default();

    println!("\n快捷鍵測試:");
    let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    if let Some(cmd) = keymap.get_command(event, false) {
        println!("  Ctrl+S (選擇模式關閉) -> {:?}", cmd);
    }

    let event2 = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
    if let Some(cmd) = keymap.get_command(event2, false) {
        println!("  Ctrl+W -> {:?}", cmd);
    }

    // 5. 顯示統計資訊
    println!("\n文字緩衝區統計:");
    println!("  行數: {}", buffer.line_count());
    println!("  字元數: {}", buffer.len_chars());

    println!("\n✅ wedi-core 可以成功作為 library 使用！");
}
