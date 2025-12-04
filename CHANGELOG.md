# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
