# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### 2026-09-24

### Fixed
- `cargo clippy --workspace --all-targets -- -D warnings` passes again, with and without default features (B14)
- Memory leak: every render leaked a cloned syntax theme (~9 KB per redraw); the highlighter now borrows the theme from the global theme set (B12)
- Crash: copying or cutting after Tab/Shift+Tab/comment-toggle on a selection, or after Undo/Redo, panicked and lost unsaved work; block operations now keep a whole-line selection and Undo/Redo clear it (B3)
- Save no longer damages encodings: UTF-16LE/BE files were written as UTF-8, a byte-order mark was dropped, and characters the save encoding can't hold were written as `&#NNNN;` while reporting "File saved"; such saves now fail with a message and leave the file untouched (B4)
- Search on a line with multi-byte text (e.g. CJK) put the cursor at the match's byte offset, so typing landed on the wrong column or line; the cursor now lands on the match (B8)
- Cursor drawn on the wrong row after toggling wrap mode (Ctrl+O), line numbers (Ctrl+L), resizing, or Undo/Redo; moving up into a wrapped line above the viewport landed on its first visual line instead of its last (B9)
- With syntax highlighting on, tab-indented lines were drawn with 8-column tabs while the cursor assumed 4, so the cursor sat inside the text; highlighted lines now expand tabs to 4 spaces like plain rendering (B10)
- In CRLF files, Backspace/Delete at a line break took two presses to join lines, and Enter, paste and comment toggling inserted LF; lines now join in one press and edits keep CRLF (B7)
- Opening a file with bytes invalid in its encoding replaced them with `�` silently, and saving destroyed them; the status bar now warns on open, and saving needs a second Ctrl+S (B5)
- Saving truncated and rewrote the file in place, so a crash or full disk mid-save could lose it; saves now write a temp file and rename it over the original, keeping symlinks and permissions (B6)
- Toggling a comment turned tab indentation into spaces (breaking Makefiles), and unknown file types such as JSON, HTML, CSS and Markdown got `#` comments; indentation is now kept as-is and those types show "No comment style" (B11)
- Pasting into the search, go-to-line and encoding prompts did nothing, and resizing the window while a prompt, confirmation or the help panel was open left it drawn at the old size; prompts now accept paste (first line) and dialogs redraw at the new size (B13)
- Over SSH on Linux (no Wayland/X display) copy silently failed and paste inserted nothing; failing clipboard commands are now detected and copy/paste fall back to the internal clipboard with a status message. Windows clipboard API failures are detected too, and AltGr characters (reported as Ctrl+Alt on Windows, e.g. `@ { [ ] €`) are typed instead of dropped (B18)
- Undo removed one character at a time and needed several steps for one multi-line indent or comment toggle, and undoing back to the last save still showed the file as modified; undo now goes one word or one command at a time and clears the modified flag at the save point (B15)
- Syntax highlighting: a block comment or string opened more than 100 lines above the screen no longer shows as code in files over 500 lines, and Undo, paste, indent and comment toggling now re-colour the lines below. Highlighting resumes from saved parser checkpoints every 64 lines, so scrolling and typing no longer re-parse the file from the top (B16)
- An unknown `--theme` name exits with `Unknown theme '…'; see --list-themes` instead of silently opening without highlighting (B19)
- Pasting multi-line text from outside wedi that ends in a newline inserts it at the cursor; only a line wedi copied itself (Ctrl+C with no selection) still pastes above the current line (B19)
- Debug output no longer writes over the editor screen: `debug_log!` appends to `wedi-debug.log` in the system temp directory and only with `--debug` (debug builds no longer log by default); an unknown `-l` language now shows its warning in the status bar instead of flashing on stderr (B19)
- Rendering: each frame is written to one buffer and flushed once (it used to go through a 1 KiB line buffer with several flushes, which flickers over SSH); rows find their search matches by binary search; the status-bar path is resolved at load and save instead of every frame; layouts are borrowed, not cloned, per row; line length and UTF-8 detection no longer copy text (B19)

### Tests
- `Editor` command dispatch now runs headless (`Editor::with_terminal`, `Terminal::with_size`), with regression tests in `src/editor.rs` (B17, in progress)
- CI re-enabled (`.github/workflows/ci.yml`): fmt, clippy `-D warnings` with and without default features, and `cargo test --workspace` on Ubuntu, Windows and macOS for pushes and PRs to `main` (B1)

### Docs
- docs: reorganize into `docs/` layout (`CONTEXT.md` index, `docs/reference/` glossary, architecture, keymap, CLI; `docs/plan/BACKLOG.md`; documentation audit record)
- `CLAUDE.md` is now a pointer; its architecture overview moved to `docs/reference/ARCHITECTURE.md` with corrections (pico-args not clap, native clipboard not arboard, undo/redo stacks)
- `AGENTS.md` holds conduct only; CI notes corrected (CI is disabled)
- README: syntax-highlight toggle is Ctrl+T (was listed as Ctrl+J); added Ctrl+E, Ctrl+PageUp/Down, mouse wheel, `-e`, `-l/--language`, `--list-languages`; corrected ESC behavior and technical stack
- Fixed v0.2.0 release date
- docs: code audit record (`docs/audit/2026-09-24-code-audit.md`) — 34 findings, 18 confirmed by probes or real-binary repros; scheduled as backlog rows B3–B21

### Removed
- Stale `Cargo.toml.backup` and unused duplicate root `assets/syntaxes.bin`
- The `wedi-widget` crate: its `EditorConfig` and `ScreenLayout` moved to `wedi_core::config` and `wedi_core::screen_layout`; the `LineLayout` and other re-exports are available from `wedi_core` directly (B20)
- Dead code (about 400 lines): the inert `mouse-support` feature (mouse capture was never enabled; the wheel works because terminals send arrow keys), `EditorConfig`, `ScreenLayout`, `LineWrapper`, `Terminal::read_key` / `set_cursor_position` / `hide_cursor` / `flush`, `RopeBuffer::save_as` / `can_undo` / `can_redo`, `HighlightEngine::detect_syntax_from_content`, `View::invalidate_lines`, `Command::ClearSelection`, and blanket `#[allow(dead_code)]` attributes that hid them (B21)

## [v0.10.0] - 2026-07-18

### Added
- Search matches are now highlighted while search mode is active — all matches get a dark-yellow background, the current match a bright-yellow background (new `wedi_core::SearchHighlight` passed to `View::render`)
- About page in the help panel (Tab/←/→ switches between Help and About) showing description, version, author, license, GitHub URL, and privacy statement
- Conventional keybindings `F1` (help), `F3`/`Shift+F3` (find next/previous)

### Changed
- **BREAKING**: `Ctrl+S` now saves the file (conventional binding); selection mode is toggled with `Alt+S` only. `Ctrl+W`/`Alt+W` remain as save aliases (help, README updated)
- ESC now dismisses one layer per press (message → selection/selection mode → search mode) instead of clearing everything at once; help text updated
- `PageUp`/`PageDown` cycle through matches while search mode is active and page otherwise (same smart-jump behavior as `Ctrl+N`/`Ctrl+P`; help, README, CLAUDE.md updated)
- Syntax highlighting is now preserved on rows outside the selection; only selected rows fall back to plain reverse-video rendering
- Confirm dialog now accepts `Enter` as yes (shown as `(Y/n)`)
- Help screen title now shows the app version (e.g. `WEDI v0.10.0 HELP`)
- refactor: help content is now structured data (`get_help_sections()`); the help panel renders section headers and key columns with colors and CJK-safe alignment
- refactor: single shared encoding label table `wedi_core::buffer::parse_encoding_label()`, replacing duplicated copies in `main.rs` and `editor.rs`
- refactor: `Cut`/`CutInternal` now share one `do_cut()` helper; `JumpTenthUp/Down` and their selection variants share one `jump_tenth()` helper

### Fixed
- `Ctrl+F` now jumps to the first match instead of the second (off-by-one in the initial jump); the first `Ctrl+N` after a search reports `Match 2/N` as expected
- Buffer-modifying commands (typing, delete, cut, paste, undo/redo, comment toggle, indent) now exit search mode, so stale match highlights no longer reappear after an edit or undo
- `cargo build --no-default-features` now compiles — the root and widget crates no longer hard-enable `wedi-core` default features; `mouse-support`/`syntax-highlighting` are forwarded properly (new `wedi-widget/mouse-support` feature, on by default)
- Update cached terminal size on resize so dialogs (search, go-to-line, encoding, help) render at the correct position after the terminal is resized
- Dialog text truncation now uses visual width instead of byte slicing, preventing a panic when CJK input exceeds the screen width; dialog cursor position now accounts for CJK double-width characters
- Help text and README listed `Ctrl+J` for toggling syntax highlight; the actual binding is `Ctrl+T`
- Clear syntax highlight cache when cutting a whole line (`Ctrl+X`/`Alt+X` without selection), matching `Ctrl+D` behavior and avoiding stale highlight residue

## [0.9.0] - 2026-06-07

### Added
- **wedi-core**: `Alt+W` as a supplementary keybinding for Save, for browser-based SSH terminals (e.g. Cloudflare) where `Ctrl+W` is intercepted to close the browser tab
- **wedi-core**: `Alt+S` as a supplementary keybinding for toggling selection mode, for browser-based SSH terminals where `Ctrl+S` is caught by XOFF flow control

### Docs
- Added Wenget install instructions to README

## [0.8.7] - 2026-02-26

### Added
- Status bar now displays full file path (absolute path) aligned to the right
- Path truncation with ellipsis prefix (…) when path exceeds screen width
- Added `RopeBuffer::file_display_path()` method for returning absolute file paths

## [0.8.6] - 2026-01-23

### Added
- **wedi-core**: Added `View::new_simple(rows, cols)` method for creating View without Terminal dependency
- **wedi-core**: Added `View::resize(rows, cols)` method for runtime size adjustment
- **wedi-widget**: Re-exported `Search` type from wedi-core for convenience
- **Documentation**: Added comprehensive rustdoc comments to `EditorConfig`, `ScreenLayout`, and `View` methods
- **Documentation**: Added comprehensive project overview (CLAUDE.md) covering architecture, code flow, and development guidelines

### Changed
- **wedi-widget**: Cleaned up module structure by removing incomplete/incompatible `state.rs` and `renderer` modules
- **wedi-widget**: Updated public API exports to include `Search`, `ScreenLayout`, and `LineLayout`

### Fixed
- **wedi-widget**: Fixed compilation by aligning widget API with current wedi-core implementation

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
- **Installation options**: Added support for Wenget and WinGet package managers

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

## [0.2.0] - 2025-11-27

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
