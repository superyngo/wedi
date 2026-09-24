# Keymap

Every key binding, as implemented in `wedi_core::keymap::bindings::handle_key_event`. The in-editor
**Help panel** (`src/help.rs` `get_help_sections`) and the README shortcut list are written from this
table. Terms are defined in [glossary.md](glossary.md). CLI flags are in [CLI.md](CLI.md).

## Editing

| Key | Command |
|---|---|
| Ctrl+S / Ctrl+W / Alt+W | Save (Alt+W for browser SSH terminals where Ctrl+W closes the tab) |
| Ctrl+Q | Quit (press twice if modified) |
| Ctrl+Z / Ctrl+Y | Undo / Redo (one typed word, or one whole command such as a multi-line indent, per step) |
| Backspace / Delete | Delete before / under cursor, or the **Selection** |
| Ctrl+D | Delete current line, or every line touched by the **Selection** |
| Tab / Shift+Tab | Indent (4 spaces) / unindent (up to 4 leading spaces); applies to selected lines |
| Enter, Ctrl+J, Ctrl+M | Insert newline (Ctrl+J/Ctrl+M cover terminals that paste newlines as control characters) |

## Navigation

| Key | Command |
|---|---|
| Arrows | Move cursor |
| Home / Ctrl+Left | Line start |
| End / Ctrl+Right | Line end |
| Ctrl+Up / Ctrl+Home | First line |
| Ctrl+Down / Ctrl+End | Last line |
| PageUp / PageDown | **Smart jump** |
| Ctrl+PageUp / Ctrl+PageDown | Jump 1/10 of the file |
| Ctrl+G | Go to line |
| Mouse wheel | Move cursor up/down (feature `mouse-support`) |

## Selection

| Key | Command |
|---|---|
| Shift+movement | Extend **Selection** (all movement keys above, including Ctrl combinations) |
| Alt+S | Toggle **Selection mode**; plain movement keys then extend the selection |
| Ctrl+A | Select all |
| ESC | Dismiss one layer per press: message, then selection / Selection mode, then **Search mode** |

## Clipboard

| Key | Command |
|---|---|
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / cut / paste via **System clipboard** (selection, or current line when none) |
| Alt+C / Alt+X / Alt+V | Same, **Internal clipboard** only |

## Search

| Key | Command |
|---|---|
| Ctrl+F | Find (last query pre-filled); jumps to the first match and enters **Search mode** |
| Ctrl+N / F3 / PageDown | Next match in Search mode, else page down |
| Ctrl+P / Shift+F3 / PageUp | Previous match in Search mode, else page up |

## View and code

| Key | Command |
|---|---|
| Ctrl+/ / Ctrl+\\ / Ctrl+K | Toggle line comment (style by file type: `//`, `#`, `--`, `REM`, `"`; `#` for files without an extension; unknown extensions such as `.json`, `.html`, `.css`, `.md` show "No comment style") |
| Ctrl+L | Toggle line numbers; sets **Display mode** to wrap when on, scroll when off |
| Ctrl+O | Toggle **Display mode** only |
| Ctrl+T | Toggle **Syntax highlighting** (feature `syntax-highlighting`) |
| Ctrl+E | Change file encoding (labels as in [CLI.md](CLI.md#encodings)) |
| Ctrl+H / F1 | Open the **Help panel** |

## Comment styles

| Prefix | Languages |
|---|---|
| `//` | Rust, C/C++, Java, JavaScript, TypeScript, Go, C# |
| `#` | Python, Shell, PowerShell, Ruby, YAML, TOML |
| `--` | SQL, Lua, Haskell |
| `REM` | Batch, CMD |
| `"` | Vim |
