# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**wedi** is a lightweight, cross-platform console text editor written in Rust. It supports basic text editing, clipboard operations, undo/redo, search, comment toggling, syntax highlighting (219+ languages), and multiple character encodings (UTF-8, GBK, Big5, Shift-JIS, etc.).

## Build & Development Commands

### Building
```bash
# Development build
cargo build

# Release build (optimized for size)
cargo build --release
```

### Running
```bash
# Run with a file
cargo run -- <filename>

# Run with debug mode
cargo run -- --debug <filename>

# Run with encoding options
cargo run -- <filename> -f utf-8 -t gbk
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture
```

### Linting
```bash
# Check for common mistakes
cargo clippy

# Apply automatic fixes
cargo clippy --fix
```

## Architecture

### Core Components

The editor follows a modular architecture with clear separation of concerns:

**Editor (`src/editor.rs`)**: Main orchestrator that owns and coordinates all components:
- `RopeBuffer` - text content storage and manipulation
- `Cursor` - cursor position tracking
- `View` - viewport and rendering logic
- `Terminal` - terminal control abstraction
- `ClipboardManager` - system clipboard integration
- `Search` - search functionality
- `CommentHandler` - language-aware comment toggling
- `Selection` - text selection state

**Text Buffer (`src/buffer/`)**:
- Uses `ropey` crate for efficient text storage
- `RopeBuffer` wraps Rope with file I/O and encoding support
- `History` implements undo/redo with action stack pattern
- Supports multiple encodings via `encoding_rs` (UTF-8, GBK, Big5, Shift-JIS, etc.)
- Auto-detects system ANSI encoding on Windows using WinAPI

**View & Rendering (`src/view.rs`)**:
- `View` manages viewport (offset_row, screen dimensions)
- `LineLayout` handles tab expansion and line wrapping
- Caches line layouts for performance
- Supports CJK characters using `unicode-width`

**Input Handling (`src/input/`)**:
- `keymap.rs` - maps keyboard events to commands
- `handler.rs` - defines `Command` enum and execution logic
- Uses `crossterm` for cross-platform terminal event handling

**Clipboard (`src/clipboard.rs`)**:
- Custom implementation using platform-specific APIs
- Windows: WinAPI (GlobalAlloc, SetClipboardData)
- macOS: pbcopy/pbpaste via Command
- Linux: xclip/xsel via Command
- Falls back to internal clipboard if system clipboard unavailable

**Syntax Highlighting (`src/highlight/`)**:
- Uses bat project's `syntaxes.bin` (219+ language definitions)
- `engine.rs` - core highlighting engine using syntect
- `cache.rs` - caching system for highlighted lines
- `mod.rs` - public API and configuration
- Three modes: Disabled, Fast (visible range only), Accurate (full file)
- Supports both 24-bit true color and 256-color terminals

**Other Components**:
- `comment.rs` - detects file type and applies appropriate comment syntax
- `search.rs` - implements find/find-next functionality
- `cursor.rs` - cursor position management
- `terminal.rs` - wraps crossterm for terminal control
- `dialog.rs` - user input dialogs (e.g., Go to Line, Find)

### Key Design Patterns

**Encoding Strategy**:
- Priority: `--to-encoding` (`-t`) > `--from-encoding` (`-f`) > detected encoding
- New files default to system ANSI encoding
- `EncodingConfig` struct holds read/save encoding preferences

**Undo/Redo**:
- Action-based history with Insert/Delete/DeleteRange actions
- `in_undo_redo` flag prevents recording during undo/redo operations
- Stacks cleared when new action is performed

**Selection**:
- Two modes: Shift-based selection and Ctrl+S toggle mode
- Selection stored as start/end (row, col) tuples
- Supports multi-line selection and quick selection to boundaries

**Terminal Management**:
- Raw mode enabled during editor session
- Panic hook ensures terminal cleanup on crash
- Cursor visibility toggled appropriately

**Syntax Highlighting**:
- Three modes for different use cases:
  - **Disabled**: No highlighting
  - **Fast**: Process only visible lines from initial state (efficient for large files)
  - **Accurate**: Process from line 0 to maintain full syntax state (accurate multi-line constructs)
- Syntax definitions from bat project (`assets/syntaxes.bin`)
- Caching system to optimize repeated rendering
- Graceful degradation on syntax errors (falls back to plain text)
- Auto-detection from file extension, filename, and shebang
- Supports 219+ programming languages

## Encoding Support

When working with encoding features:
- `RopeBuffer::from_file_with_encoding()` handles reading with specific encoding
- `get_system_ansi_encoding()` detects Windows code page or Unix locale
- Encoding detection uses BOM or charset analysis
- Save encoding can differ from read encoding (useful for transcoding)

## Syntax Highlighting Architecture

### Components

**HighlightEngine** (`src/highlight/engine.rs`):
- Loads syntax definitions from embedded `syntaxes.bin`
- Creates `LineHighlighter` instances for stateful highlighting
- Detects file type from extension, filename, or shebang
- Supports both true color (24-bit) and 256-color terminals
- Graceful error handling (degrades to plain text on syntax errors)

**HighlightCache** (`src/highlight/cache.rs`):
- Caches highlighted lines for performance
- Simplified caching (no ParseState, as it's private in syntect)
- Invalidation strategies for different edit types
- Absolute indexing with HashMap storage

**SyntaxHighlightMode** (`src/editor.rs`):
- **Disabled**: No highlighting
- **Fast**: Highlights only visible range from initial state
  - May have color differences for multi-line constructs
  - Ideal for large files and fast scrolling
- **Accurate**: Processes from line 0 to maintain full syntax state
  - Ensures accurate colors for heredocs, multi-line strings, etc.
  - May have delay for very large files

### Execution Order

Critical: In the render loop, `scroll_if_needed()` must be called BEFORE calculating `highlighted_lines` to ensure the correct `offset_row` is used. This fixes the "page jump causes highlighting to disappear" bug.

### Syntax Definitions

wedi uses the [bat](https://github.com/sharkdp/bat) project's syntax definitions:
- File: `assets/syntaxes.bin`
- Languages: 219+ programming languages
- License: MIT License / Apache License 2.0 (dual licensed)
- Original sources: Sublime Text packages (MIT License)

The syntax definitions are embedded in the binary during compilation using `include_bytes!()`.

## Platform-Specific Code

Windows-only code is gated with `#[cfg(target_os = "windows")]`:
- Clipboard implementation uses WinAPI
- System encoding detection uses GetACP()
- True color terminal detection checks for ENABLE_VIRTUAL_TERMINAL_PROCESSING
- Dependencies include `winapi` crate features

**Syntax highlighting** uses `#[cfg(feature = "syntax-highlighting")]`:
- Default feature, enabled unless explicitly disabled
- Can be disabled with `--no-default-features` for minimal builds

## Testing

Tests are located in module files with `#[cfg(test)]` blocks. Currently tests exist in:
- `src/buffer/rope_buffer.rs` - buffer operation tests
- `src/highlight/engine.rs` - syntax highlighting tests
- `src/highlight/cache.rs` - cache invalidation tests

When adding tests, use the `tempfile` crate for temporary file operations (already in dev-dependencies).

**Syntax highlighting tests**:
- Verify engine creation and syntax detection
- Test multi-line constructs (comments, strings)
- Verify graceful degradation on syntax errors
- Ensure 219+ syntaxes are loaded from bat's syntaxes.bin

## Binary Size Optimization

Release profile is heavily optimized for size:
- Strip symbols: `strip = true`
- LTO enabled: `lto = true`
- Optimize for size: `opt-level = "z"`
- Single codegen unit: `codegen-units = 1`
- Abort on panic: `panic = "abort"`

When adding dependencies, prefer minimal crates. The project uses `pico-args` instead of `clap` for CLI parsing to reduce binary size.

## Common Patterns

**Debug Logging**:
```rust
if cfg!(debug_assertions) || self.debug_mode {
    eprintln!("[DEBUG] message");
}
```

**Error Handling**:
- Use `anyhow::Result` for functions that can fail
- Use `.context()` to add error context
- Return `anyhow::bail!()` for custom errors

**Cursor Movement**:
- Always update cursor position through `Cursor` methods
- Validate bounds against buffer line count and line length
- Consider visual width for CJK characters

**Rendering**:
- Clear selection/messages with ESC
- Always refresh view after buffer modifications
- Use `View::scroll_to_cursor()` to keep cursor visible

**Syntax Highlighting**:
- Call `scroll_if_needed()` BEFORE calculating highlighted lines
- Use Fast mode for large files (> 10,000 lines recommended)
- Cache is automatically invalidated on edits
- Syntax detection happens on file load (extension → filename → shebang)

## Third-Party Acknowledgments

wedi uses syntax definitions from the [bat](https://github.com/sharkdp/bat) project:
- **File**: `assets/syntaxes.bin` embedded in binary
- **Languages**: 219+ programming language definitions
- **License**: MIT License / Apache License 2.0 (dual licensed)
- **Original Sources**: Sublime Text packages (MIT License)
- **Repository**: https://github.com/sharkdp/bat

The syntax definitions are used under the MIT License terms. We are grateful to the bat project maintainers and the Sublime Text community for maintaining these excellent syntax definitions.
