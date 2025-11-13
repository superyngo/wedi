# wedi

A lightweight, easy-to-use console text editor written in Rust.

## Features

- ✅ Cross-platform support (Windows, macOS, Linux)
- ✅ Lightweight and fast startup
- ✅ Basic text editing operations
- ✅ Line numbers display (toggleable)
- ✅ Clipboard support (copy, cut, paste)
- ✅ Selection mode with Shift key
- ✅ **Ctrl+S Selection Mode** (for terminals without Shift key support) 🎉
- ✅ Smart line operations
- ✅ **Undo/Redo (Ctrl+Z/Y)** 🎉
- ✅ **Search functionality (Ctrl+F, F3)** 🎉
- ✅ **Comment toggling (Ctrl+K/\\//)** 🎉
- ✅ **Go to line (Ctrl+G)** 🎉
- ✅ **Tab/Shift+Tab indentation** 🎉
- ✅ **Fast navigation (Ctrl+H/E, Ctrl+Arrows/Home/End)** 🎉
- ✅ **Chinese character support** 🎉
- ✅ **Comment highlighting** 🎉

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

### Encoding Options

wedi supports specifying different encodings for reading and saving files:

```bash
# Specify decoding (reading) encoding
wedi <filename> --dec <encoding>

# Specify encoding (saving) encoding
wedi <filename> --en <encoding>

# Specify both decoding and encoding
wedi <filename> --dec <encoding> --en <encoding>
```

**Supported Encodings:**
- `utf-8` / `utf8` (default)
- `utf-16le` / `utf-16be`
- `gbk` (Chinese GBK)
- `shift-jis` (Japanese Shift-JIS)
- `big5` (Traditional Chinese Big5)
- `cp1252` (Western European Windows-1252)
- And many more...

**Encoding Priority for Saving:**
When saving files, the encoding priority is: `--en` > `--dec` > detected encoding from file.

**Examples:**
```bash
# Read UTF-8, save as GBK
wedi file.txt --dec utf-8 --en gbk

# Read with auto-detection, save as UTF-16LE
wedi file.txt --en utf-16le

# Read GBK, save as GBK
wedi file.txt --dec gbk
```

## Keyboard Shortcuts

### Basic Editing

- **Ctrl+W**: Save file
- **Ctrl+Q**: Quit (press twice if modified)
- **Ctrl+Z**: Undo
- **Ctrl+Y**: Redo
- **Backspace**: Delete character before cursor or selected text
- **Delete**: Delete character under cursor or selected text
- **Ctrl+D**: Delete current line or selected lines
- **Tab**: Indent (insert 4 spaces or indent selected lines)
- **Shift+Tab**: Unindent (remove up to 4 leading spaces)

### Navigation

- **Arrow Keys**: Move cursor
- **Ctrl+Up** / **Ctrl+Home**: Move to first line
- **Ctrl+Down** / **Ctrl+End**: Move to last line
- **Home** / **Ctrl+H**: Move to line start
- **End** / **Ctrl+E**: Move to line end
- **Page Up / Page Down**: Scroll page up/down
- **Ctrl+G**: Go to line number

### Selection

- **Ctrl+S**: Toggle selection mode (for terminals without Shift key support)
- **Shift + Arrow Keys**: Select text
- **Shift + Home / End**: Select to line start/end
- **Shift + Page Up / Down**: Select page up/down
- **Shift + Ctrl + Arrows**: Quick select to line/file start/end
- **Shift + Ctrl + H / E**: Quick select to line start/end
- **Ctrl+A**: Select all
- **ESC**: Clear selection and messages

> **Note**: In Ctrl+S selection mode, all movement keys (arrows, Home/End, Page Up/Down, Ctrl+arrows, Ctrl+H/E) will extend selection. Press Ctrl+S again, ESC, or perform any editing operation to exit selection mode.

### Clipboard

- **Ctrl+C**: Copy (selection or current line)
- **Ctrl+X**: Cut (selection or current line)
- **Ctrl+V**: Paste
- **Alt+C**: Internal Copy (selection or current line)
- **Alt+X**: Internal Cut (selection or current line)
- **Alt+V**: Internal Paste

### Search

- **Ctrl+F**: Find text
- **F3**: Find next match
- **Shift+F3**: Find previous match

### Code

- **Ctrl+/** / **Ctrl+\\** / **Ctrl+K**: Toggle line comment
- **Ctrl+L**: Toggle line numbers

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
