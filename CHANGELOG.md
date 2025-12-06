# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-12-06

### Changed
- **PageUp/PageDown behavior overhaul**: Now scrolls entire page while maintaining cursor's screen Y position (similar to VS Code/Vim)
- When no more pages to scroll: PageUp jumps to first line, PageDown jumps to last line
- Improved large file navigation with optimized end-of-file jump detection

### Fixed
- **Critical**: Fixed syntax highlighting token-level newline handling to prevent Linux terminal artifacts
- Optimized ANSI escape code generation: only output color codes on color changes (30-50% output size reduction)
- Single reset code at end of highlighted line instead of per-token reset
- Removed redundant post-processing trim in editor (now handled at engine level)

### Performance
- Large file end-page jump optimization: skip processing from file start when jumping to end
- Pre-allocated string buffer for ANSI output generation
- Reduced memory allocations in highlight engine

### Removed
- Removed unused `move_page_up` and `move_page_down` methods from Cursor (replaced by View-based paging)

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
