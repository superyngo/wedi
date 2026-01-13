# AGENTS.md - Guidelines for AI Coding Agents

This document provides guidelines for AI coding agents working on the **wedi** codebase - a cross-platform minimalist lightweight CLI text editor written in Rust.

## Project Overview

- **Language**: Rust (2021 Edition)
- **Type**: CLI text editor with syntax highlighting (219+ languages)
- **Architecture**: Workspace with 3 crates:
  - `wedi` (main binary)
  - `wedi-core` (core editor primitives)
  - `wedi-widget` (TUI widget for embedding)

## Build & Development Commands

### Building
```bash
cargo build                    # Debug build
cargo build --release          # Release build (optimized for size)
cargo build --no-default-features  # Build without syntax highlighting
```

### Running
```bash
cargo run -- <file>            # Run with a file
cargo run -- --help            # Show help
cargo run -- --debug <file>    # Run with debug output
```

### Testing
```bash
cargo test                     # Run all tests
cargo test --verbose           # Verbose test output
cargo test <test_name>         # Run a single test by name
cargo test --package wedi-core # Test specific crate
cargo test --package wedi-core -- test_utf8_file_detection  # Single test in crate
```

### Linting & Formatting
```bash
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting (CI mode)
cargo clippy                   # Run linter
cargo clippy -- -D warnings    # Treat warnings as errors (CI mode)
```

## Code Style Guidelines

### Imports
- Group imports in this order: std, external crates, internal modules
- Use `use crate::` for internal module imports
- Prefer explicit imports over glob imports (`*`)
- Re-export commonly used types in `lib.rs` for convenience

```rust
use anyhow::{Context, Result};
use ropey::Rope;
use std::path::{Path, PathBuf};

use crate::buffer::RopeBuffer;
use crate::debug_log;
```

### Naming Conventions
- **Structs/Enums/Traits**: `PascalCase` (e.g., `RopeBuffer`, `HighlightEngine`)
- **Functions/Methods**: `snake_case` (e.g., `get_line_content`, `move_cursor`)
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case` (e.g., `rope_buffer`, `ansi_slice`)
- **Feature flags**: `kebab-case` (e.g., `syntax-highlighting`)

### Error Handling
- Use `anyhow::Result` for functions that can fail
- Use `anyhow::Context` to add context to errors
- Use `anyhow::bail!` for early returns with errors
- Prefer `?` operator over explicit `match` for error propagation

```rust
fn load_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .context("Failed to read file")?
}
```

### Documentation
- Use Chinese (Traditional) for inline comments
- Use English for public API documentation
- Module-level docs use `//!` comments
- Add `# Examples` section for complex public functions

```rust
//! wedi-core - 核心編輯器元件
//!
//! 提供文字緩衝區、游標管理、快捷鍵對映和語法高亮等基礎功能。
```

### Conditional Compilation
- Use `#[cfg(feature = "...")]` for optional features
- Use `cfg!(debug_assertions)` for debug-only code
- Platform-specific code uses `#[cfg(target_os = "...")]`

```rust
#[cfg(feature = "syntax-highlighting")]
pub mod highlight;

if cfg!(debug_assertions) {
    eprintln!("[DEBUG] Loading file: {:?}", path);
}
```

### Type Patterns
- Prefer explicit types over `impl Trait` in public APIs
- Use `&str` for input strings, `String` for owned output
- Use `Option<T>` for optional values, never null-like patterns
- Lifetime annotations only when necessary

### Struct Design
- Fields should be private by default
- Use `pub(crate)` for internal visibility
- Group related fields with comments
- Use `#[allow(dead_code)]` sparingly, document why

```rust
pub struct Editor {
    buffer: RopeBuffer,
    cursor: Cursor,
    // 語法高亮（可選功能）
    #[cfg(feature = "syntax-highlighting")]
    highlight_engine: Option<HighlightEngine>,
}
```

## Testing Guidelines

### Test Structure
- Tests are inline within source files using `#[cfg(test)]` modules
- Use `tempfile::TempDir` for file system tests
- Test names follow `test_<what_is_tested>` pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_utf8_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        // ... test implementation
    }
}
```

### Test Assertions
- Use `assert_eq!` for equality checks
- Use `assert!` for boolean conditions
- Include descriptive messages for complex assertions

## Project Structure

```
wedi/
├── src/
│   ├── main.rs          # CLI entry point, argument parsing
│   ├── editor.rs        # Main editor loop and state
│   ├── dialog.rs        # Dialog components
│   └── help.rs          # Help system
├── crates/
│   ├── wedi-core/       # Core primitives
│   │   └── src/
│   │       ├── buffer/  # Text buffer (rope_buffer, history)
│   │       ├── cursor.rs
│   │       ├── keymap/  # Key bindings
│   │       ├── highlight/ # Syntax highlighting
│   │       └── utils/   # Utilities (ansi_slice, line_wrapper)
│   └── wedi-widget/     # Embeddable TUI widget
└── assets/
    └── syntaxes.bin     # Precompiled syntax definitions
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `ropey` | Efficient rope-based text buffer |
| `crossterm` | Cross-platform terminal control |
| `syntect` | Syntax highlighting engine |
| `anyhow` | Error handling |
| `encoding_rs` | Multi-encoding support |
| `arboard` | Cross-platform clipboard |

## Feature Flags

- `syntax-highlighting` (default): Enable syntax highlighting via syntect

## Common Patterns

### Debug Logging
```rust
use crate::debug_log;

debug_log!("Starting wedi with file: {:?}", args.file);
```

### Cache Invalidation
When modifying the buffer, invalidate relevant caches:
```rust
self.view.invalidate_cache();       // Layout cache
#[cfg(feature = "syntax-highlighting")]
self.highlight_cache.clear();       // Syntax cache
```

### Selection Handling
```rust
if self.has_selection() {
    self.delete_selection();
}
```

## Performance Considerations

- Release builds use aggressive optimizations (`opt-level = "z"`, LTO, strip)
- Avoid allocations in hot paths (rendering, key handling)
- Use `invalidate_line()` over `invalidate_cache()` when possible
- Syntax highlighting uses caching to avoid re-parsing

## CI/CD Notes

- CI runs on Ubuntu, Windows, and macOS
- Clippy warnings are treated as errors
- Format checks are enforced
- Tests run with `cargo test --verbose`
