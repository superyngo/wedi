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

### Quick Install (One-Line Command)

#### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/superyngo/wedi/main/install.ps1 | iex
```

**Uninstall:**
```powershell
irm https://raw.githubusercontent.com/superyngo/wedi/main/install.ps1 | iex -Uninstall
```

#### Linux / macOS (Bash)

```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/wedi/main/install.sh | bash
```

**Uninstall:**
```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/wedi/main/install.sh | bash -s uninstall
```

The installation script will:
- Automatically detect your OS and architecture
- Download the latest precompiled binary from GitHub Releases
- Install to:
  - Windows: `%LOCALAPPDATA%\Programs\wedi`
  - Linux/macOS: `~/.local/bin`
- Add the installation directory to your PATH (if needed)

**Supported Platforms:**
- Windows (x86_64, ARM64)
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)

> **Note:** Replace `superyngo` with the actual GitHub superyngo in the URLs above.

### Manual Installation

#### From Precompiled Binaries

Download the latest release for your platform from the [Releases](https://github.com/superyngo/wedi/releases) page.

**Windows:**
```powershell
# Extract the downloaded zip file and move wedi.exe to a directory in your PATH
# For example:
move wedi.exe %LOCALAPPDATA%\Programs\wedi\
```

**Linux/macOS:**
```bash
# Extract the downloaded tar.gz file and move wedi to a directory in your PATH
tar -xzf wedi-*.tar.gz
chmod +x wedi
mv wedi ~/.local/bin/
```

#### From Source

If you prefer to build from source, ensure you have [Rust](https://rustup.rs/) installed:

```bash
# Clone the repository
git clone https://github.com/superyngo/wedi.git
cd wedi

# Build release binary
cargo build --release

# The binary will be available at:
# - Windows: target\release\wedi.exe
# - Linux/macOS: target/release/wedi

# Install manually
# Windows:
copy target\release\wedi.exe %LOCALAPPDATA%\Programs\wedi\

# Linux/macOS:
cp target/release/wedi ~/.local/bin/
chmod +x ~/.local/bin/wedi
```

## Usage

```bash
# Open or create a file
wedi <filename>

# Show help
wedi -h
# or
wedi --help

# Show version
wedi -v
# or
wedi --version

# Enable debug mode
wedi --debug <filename>
```

### Encoding Options

wedi supports specifying different encodings for reading and saving files:

```bash
# Specify source encoding (reading)
wedi <filename> --from-encoding <encoding>
# or use shorthand
wedi <filename> -f <encoding>

# Specify target encoding (saving)
wedi <filename> --to-encoding <encoding>
# or use shorthand
wedi <filename> -t <encoding>

# Specify both source and target encoding
wedi <filename> -f <encoding> -t <encoding>
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
When saving files, the encoding priority is: `--to-encoding` > `--from-encoding` > detected encoding from file.

**Examples:**
```bash
# Read UTF-8, save as GBK
wedi file.txt -f utf-8 -t gbk

# Read with auto-detection, save as UTF-16LE
wedi file.txt -t utf-16le

# Read GBK, save as GBK
wedi file.txt -f gbk
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
