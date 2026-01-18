# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.5] - 2026-01-18

### Fixed
- **Multi-line Paste (Termius/iOS)**: Fixed paste issue in terminals that send Ctrl+J for newlines
  - Added `Ctrl+J` (LF, ASCII 10) and `Ctrl+M` (CR, ASCII 13) handling as newline input
  - Terminals like Termius on iOS send newlines as control characters instead of key events
  - This complements the previous fix for Bracketed Paste Mode

### Changed
- **Keybinding**: Changed "Toggle Syntax Highlight" shortcut from `Ctrl+J` to `Ctrl+T`
  - `Ctrl+J` is now used for newline input (required for Termius compatibility)

## [0.8.4] - 2026-01-18

### Fixed
- **Multi-line Paste**: Fixed issue where pasting multi-line text from terminal would merge all lines into one
  - Enabled Bracketed Paste Mode for proper terminal paste detection
  - Added `InputEvent` type and `read_input()` method to distinguish keyboard events from paste events
  - Added `PasteText` command for direct text pasting with proper line break handling
  - Normalized `\r` characters to `\n` in both paste events and keyboard input

## [0.8.3] - 2026-01-15

### Fixed
- **Mouse Interaction**: Disabled mouse capture in raw mode to allow text selection and better terminal interaction

## [0.8.2] - 2026-01-15

### Fixed
- Fixed winapi dependency issue for Windows builds

## [0.8.0] - 2026-01-12

### Added
- **Workspace refactoring**: Split into `wedi-core` and `wedi-widget` crates for reusability
- **crates.io ready**: Both library crates now include proper metadata for publishing
  - `wedi-core`: Core editor primitives (buffer, cursor, keymap, syntax highlighting)
  - `wedi-widget`: TUI widget for embedding wedi editor in applications
- Added `repository`, `homepage`, `documentation`, `keywords`, `categories` to Cargo.toml

### Changed
- **Architecture**: Refactored from monolithic to workspace structure
- `Keymap` now implements `Default` trait via derive macro
- Conditional compilation for syntax-highlighting related imports

### Fixed
- Fixed `test_partial_wide_char` test - now correctly checks visual width instead of byte length
- Fixed unused import warnings in `view.rs` with proper `#[cfg]` attributes

## [0.7.0] - 2026-01-08

### Added
- **Ctrl+O shortcut**: Toggle display mode (wrap/scroll) independently from line numbers
- Separated display mode toggle from line number toggle for more flexible control

### Changed
- **Ctrl+L behavior**: Now only toggles line numbers (previously also toggled display mode)
- **Delete line in selection mode**: Now deletes all lines that contain the selection (previously only deleted selected text)

### Removed
- Removed development documentation files (moved to devs/ folder)
- Removed install scripts (install.ps1, install.sh)

## [0.6.0] - 2026-01-06

### Added
- **Syntax highlighting theme parameter**: Use `--theme <THEME>` to set theme via command line
- **Installation options**: Added support for Wenget and Winget package managers

### Fixed
- Resolved cargo fmt and clippy warnings for cleaner codebase

## [0.5.2] - 2025-12-27

### Added
- **Single-line/Multi-line display mode toggle**: Press Ctrl+L to switch between modes
  - **Multi-line mode (default)**: Shows line numbers, long lines wrap to next visual line
  - **Single-line mode**: Hides line numbers, long lines scroll horizontally
- **Horizontal scrolling** in single-line mode with automatic cursor following
- **Full syntax highlighting support** in single-line mode using ANSI escape code slicing
- New `src/utils/ansi_slice.rs` module for proper ANSI escape sequence handling

### Fixed
- Fixed syntax highlighting display bug in wrap mode where characters were duplicated at visual line boundaries
- Syntax highlighting now correctly renders across all visual lines (previously only worked on first visual line)

### Technical
- Added `offset_col` and `wrap_mode` fields to View struct
- Added `slice_ansi_text()` function for slicing text with ANSI codes by visual width
- Modified `LineLayout::new()` to support wrap parameter
- Updated cursor movement (`move_up`/`move_down`) to handle single-line mode correctly

## [0.5.0] - 2025-12-24

### Added
- **In-editor help dialog**: Press Ctrl+H to display keyboard shortcuts and usage guide
  - Scrollable full-screen help dialog with all keyboard shortcuts
  - Supports navigation with arrow keys, Page Up/Down, Home/End
  - Press ESC to close and return to editing
- Centralized help content system - both `--help` and Ctrl+H use the same source

### Changed
- Ctrl+H now opens help dialog (previously used as alternative for Home key)
- Refactored help message system for consistency between CLI and in-editor help

## [0.4.0] - 2025-12-24

### Added
- Smart search mode with state management (Ctrl+F to enter, ESC to exit)
- Search query persistence - previous search term is automatically filled in next search
- Cursor movement support in search dialog (Left/Right arrows, Home/End, Delete)
- Smart navigation shortcuts:
  - Ctrl+N/P: Jump to next/previous search result when in search mode, otherwise PageDown/PageUp
  - PageUp/Down: Jump to search results when in search mode, otherwise normal paging

### Changed
- **BREAKING**: Changed syntax highlighting toggle from Ctrl+H to Ctrl+J
- **BREAKING**: Removed F3/F4 keybindings (replaced by smart PageUp/Down and Ctrl+N/P)
- Search mode now has explicit state - ESC exits search mode while preserving search results
- Improved search dialog with full cursor editing capabilities

### Fixed
- Search mode now properly toggles between search navigation and normal paging
- Search results are preserved until a new search is initiated

## [0.3.0] - 2025-12-04

### Changed
- **BREAKING**: Simplified syntax highlighting modes from three modes (Disabled/Fast/Accurate) to simple on/off toggle
- Ctrl+H now toggles syntax highlighting between Enabled/Disabled instead of cycling through modes
- Improved syntax highlighting performance with incremental processing strategy
  - Small files (≤500 lines): Process from start for accuracy
  - Large files: Process visible area ± 100 line buffer for performance
- Optimized highlighting cache strategy for better memory usage

### Fixed
- **Critical**: Fixed syntax highlighting artifacts on Linux terminals caused by newline characters in highlighted output
- Fixed cursor position misalignment when editing with syntax highlighting enabled
- Fixed phantom characters appearing after deletion operations in highlight mode
- Fixed visual line duplication when inserting newlines with syntax highlighting

### Removed
- Removed "Fast" syntax highlighting mode (merged functionality into single accurate mode)
- Removed complexity of multi-mode switching for better user experience

## [0.2.3] - 2025-12-02

### Added
- Extended file type detection for shell configuration files (.bashrc, .zshrc, .profile, etc.)
- Support for Bash/Shell Script syntax highlighting on shell config files

### Changed
- Improved special filename handling with more robust syntax detection fallbacks
- Enhanced compatibility with various shell configuration file naming conventions

## [0.2.2] - 2025-12-02

### Changed
- Removed .claude directory from git tracking
- Updated .gitignore for better local development file handling

### Fixed
- Fixed selection mode behavior: now clears selection after copy/cut operations instead of just disabling selection mode

## [0.2.1] - 2025-12-01

### Added
- `--theme <THEME>` command-line option to set syntax highlighting theme
- `--list-themes` command-line option to list all available themes (7 themes)
- Support for custom theme selection on startup

### Changed
- Updated help documentation to include new theme options

### Fixed
- Improved theme selection and display clarity

## [0.2.0] - 2024-XX-XX

### Added
- Syntax highlighting support for 219+ programming languages
- Three syntax highlighting modes: Disabled, Fast, Accurate
- Ctrl+H shortcut to toggle syntax highlighting modes
- Support for both 24-bit true color and 256-color terminals
- Syntax definitions from bat project (MIT/Apache 2.0 licensed)

### Changed
- Enhanced rendering performance with syntax highlighting cache

## [0.1.0] - Initial Release

### Added
- Basic text editing functionality
- Multi-platform support (Windows, macOS, Linux)
- Clipboard operations (Ctrl+C, Ctrl+X, Ctrl+V)
- Undo/Redo support (Ctrl+Z, Ctrl+Y)
- Search functionality (Ctrl+F, F3, F4)
- Line comment toggling (Ctrl+/)
- Multiple character encoding support (UTF-8, GBK, Big5, Shift-JIS, etc.)
- Line numbers display toggle (Ctrl+L)
- Go to line (Ctrl+G)
- Selection modes with Shift or Ctrl+S
- Auto-save on quit with confirmation
