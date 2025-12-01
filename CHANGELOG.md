# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
