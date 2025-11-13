mod buffer;
mod clipboard;
mod comment;
mod config;
mod cursor;
mod dialog;
mod editor;
mod highlight;
mod input;
mod search;
mod terminal;
mod utils;
mod view;

use anyhow::Result;
use clap::Parser;
use editor::Editor;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "wedi")]
#[command(author = "wen")]
#[command(version = "0.1.12")]
#[command(about = "A lightweight, easy-to-use console text editor.")]
#[command(long_about = "
wedi - A easy-to-use text editor

KEYBOARD SHORTCUTS:
  
  Basic Editing:
    Ctrl+W              Save file
    Ctrl+Q              Quit (press twice if modified)
    Ctrl+Z              Undo
    Ctrl+Y              Redo
    Backspace           Delete character before cursor or selected text
    Delete              Delete character under cursor or selected text
    Ctrl+D              Delete current line or selected lines
    Tab                 Indent (insert 4 spaces or indent selected lines)
    Shift+Tab           Unindent (remove up to 4 leading spaces)

  Navigation:
    Arrow Keys            Move cursor
    Ctrl+Left/Ctrl+H/Home Move to line start
    Ctrl+Right/Ctrl+E/End Move to line end
    Ctrl+Up/Ctrl+Home     Move to first line
    Ctrl+Down/Ctrl+End    Move to last line
    Page Up/Down          Scroll page up/down
    Ctrl+G                Go to line number
    
  Selection:
    Ctrl+S              Toggle selection mode (for terminals without Shift support)
    Shift+Arrows        Select text
    Shift+Ctrl+Arrows   Quick select to line/file boundaries
    Shift+Home/End      Select to line boundaries
    Shift+Ctrl+Home/End Quick select to file boundaries
    Shift+Ctrl+H/E      Quick select to line boundaries
    Shift+PgUp/Dn       Select page up/down
    Ctrl+A              Select all
    ESC                 Clear selection and messages

  Clipboard:
    Ctrl+C              Copy (selection or current line)
    Ctrl+X              Cut (selection or current line)
    Ctrl+V              Paste
    Alt+C               Internal Copy (selection or current line)
    Alt+X               Internal Cut (selection or current line)
    Alt+V               Internal Paste
    
  Search:
    Ctrl+F              Find text
    F3                  Find next match
    Shift+F3            Find previous match
    
  Code:
    Ctrl+/ \\ K         Toggle line comment
    Ctrl+L              Toggle line numbers

SUPPORTED COMMENT STYLES:
  //  - Rust, C/C++, Java, JavaScript, TypeScript, Go, C#
  #   - Python, Shell, PowerShell, Ruby, YAML, TOML
  --  - SQL, Lua, Haskell
  REM - Batch, CMD
  \"   - Vim
")]
struct Args {
    /// File to open or create (default: Untitled)
    #[arg(default_value = "Untitled")]
    file: PathBuf,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 初始化日誌
    utils::init_logger(args.debug);

    // 創建並運行編輯器
    let mut editor = Editor::new(Some(&args.file), args.debug)?;

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
