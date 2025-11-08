# wedi

A cross-platform minimalist lightweight CLI text editor written in Rust.

## Features

- ✅ Cross-platform support (Windows, macOS, Linux)
- ✅ Lightweight and fast startup
- ✅ Basic text editing operations
- ✅ Line numbers display (toggleable)
- ✅ Clipboard support (copy, cut, paste)
- ✅ Selection mode with Shift key
- ✅ Smart line operations
- ✅ **Undo/Redo (Ctrl+Z/Y)** 🎉
- ✅ **Search functionality (Ctrl+F, F3)** 🎉
- ✅ **Comment toggling (Ctrl+U/\\/)** 🎉
- ✅ **Go to line (Ctrl+G)** 🎉
- ✅ **Tab/Shift+Tab indentation** 🎉
- ✅ **Fast navigation (Ctrl+Arrows/Home/End)** 🎉
- ✅ **Chinese character support** 🎉
- ✅ **Comment highlighting** 🎉
- 🚧 Syntax highlighting (coming soon)

## Installation

### From Source

```bash
cargo build --release
```

The binary will be available at `target/release/wedi`.

## Usage

```bash
# Open or create a file
wedi <filename>

# Show help
wedi -h

# Show version
wedi -v

# Enable debug mode
wedi --debug <filename>
```

## Keyboard Shortcuts

### Basic Operations

- **Arrow Keys**: Move cursor
- **Home / End**: Move to line start/end
- **Page Up / Page Down**: Scroll page
- **Ctrl+S**: Save file
- **Ctrl+Q**: Quit editor (press twice if modified)
- **Ctrl+L**: Toggle line numbers
- **Ctrl+G**: Go to line
- **Tab**: Insert 4 spaces
- **Shift+Tab**: Remove up to 4 leading spaces

### Fast Navigation

- **Ctrl+Up** / **Ctrl+Home**: Jump to first line
- **Ctrl+Down** / **Ctrl+End**: Jump to last line
- **Ctrl+Left**: Jump to line start
- **Ctrl+Right**: Jump to line end

### Selection Mode

- **Shift + Arrow Keys**: Select text
- **Shift + Home / End**: Select to line start/end
- **Shift + Page Up / Down**: Select page
- **Ctrl+Shift+Left**: Select to line start
- **Ctrl+Shift+Right**: Select to line end
- **Ctrl+A**: Select all
- **ESC**: Clear selection

### Editing Operations

| Shortcut             | With Selection            | Without Selection     |
| -------------------- | ------------------------- | --------------------- |
| **Ctrl+C**           | Copy selected text        | Copy current line     |
| **Ctrl+X**           | Cut selected text         | Cut current line\*    |
| **Ctrl+V**           | Paste (replace selection) | Paste at cursor\*\*   |
| **Ctrl+D**           | Delete selected text      | Delete current line\* |
| **Backspace/Delete** | Delete selection          | Delete character      |
| **Any character**    | Replace selection         | Insert character      |

\* After cutting/deleting a line, cursor moves up one line  
\*\* Whole-line paste inserts at line start, pushing original line down

### Advanced Operations 🎉

- **Ctrl+Z**: Undo
- **Ctrl+Y**: Redo
- **Ctrl+F**: Find text (search with navigation)
- **F3**: Find next match
- **Shift+F3**: Find previous match
- **Ctrl+G**: Go to line
- **Ctrl+U** / **Ctrl+\\** / **Ctrl+/**: Toggle comment (supports multiple languages)

## Supported Comment Styles

wedi automatically detects file type and applies appropriate comment style:

- **Rust, C/C++, Java, JavaScript, TypeScript, Go, C#**: `//`
- **Python, Shell, PowerShell, Ruby, YAML, TOML**: `#`
- **SQL, Lua, Haskell**: `--`
- **Batch, CMD**: `REM`
- **Vim**: `"`

Comments are highlighted in green color for better visibility.

## Technical Stack

- **Language**: Rust 2021 Edition
- **Terminal Library**: crossterm (terminal control and event handling)
- **Text Buffer**: ropey (efficient text buffer with undo/redo)
- **Clipboard**: arboard (cross-platform clipboard)
- **CLI Parsing**: clap (command-line argument parsing)
- **Unicode Support**: unicode-width (proper CJK character handling)

## Development

### Build

```bash
cargo build
```

### Run

```bash
cargo run -- <filename>
```

### Test

```bash
cargo test
```

### Release Build

```bash
cargo build --release
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
