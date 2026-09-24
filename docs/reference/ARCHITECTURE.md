# Architecture

How wedi is put together. Vocabulary: [glossary.md](glossary.md). Bindings: [KEYMAP.md](KEYMAP.md).
Flags: [CLI.md](CLI.md).

## Workspace

| Crate | Path | Role |
|---|---|---|
| `wedi` | `src/` | The editor binary: argument parsing, editor loop, dialogs, help panel |
| `wedi-core` | `crates/wedi-core/` | Reusable editor primitives; no knowledge of the binary |
| `wedi-widget` | `crates/wedi-widget/` | Embedding layer: `EditorConfig`, `ScreenLayout`, `LineLayout`, re-exports of `wedi-core` |

`examples/basic_usage.rs` shows the library crates used standalone.

### `wedi` (binary)

| File | Holds |
|---|---|
| `main.rs` | `Args::parse` (pico-args), encoding resolution, theme/language selection, startup |
| `editor.rs` | `Editor`: main loop, `Command` execution, **Search mode**, **Selection mode**, clipboard routing, highlight scheduling |
| `dialog.rs` | Single-line **Dialog** prompts and confirm |
| `help.rs` | `get_help_sections`, `get_about_entries` — one source for the **Help panel** and `--help` |

### `wedi-core` modules

| Module | Key types | Role |
|---|---|---|
| `buffer` | `RopeBuffer`, `History`, `Action`, `EncodingConfig`, `parse_encoding_label` | Rope text storage, file I/O with encoding, undo/redo |
| `cursor` | `Cursor` | Logical cursor position |
| `view` | `View`, `LineLayout`, `SearchHighlight` | Rendering, **Visual line** layout, scrolling, line numbers, **Display mode** |
| `keymap` | `Command`, `Keymap`, `handle_key_event` | Key event → **Command**; `Keymap` supports custom `bind` / `bind_selection` for embedders |
| `search` | `Search` | Query and match list |
| `comment` | `CommentHandler` | Comment prefix by file type |
| `clipboard` | `ClipboardManager` | **System clipboard**: Win32 API on Windows, pbcopy/pbpaste on macOS, wl-copy/wl-paste then xclip on Linux |
| `highlight` | `HighlightEngine`, `HighlightCache` | syntect engine and per-line ANSI cache (feature-gated) |
| `terminal` | `Terminal`, `InputEvent` | crossterm wrapper: raw mode, bracketed paste, size |
| `utils` | `ansi_slice`, `line_wrapper` | Width-aware slicing of ANSI-coloured text, wrapping |

## Cargo features

| Feature | Crates | Default | Effect |
|---|---|---|---|
| `syntax-highlighting` | `wedi`, `wedi-core`, `wedi-widget` | on in `wedi` | syntect + bundled syntax set, Ctrl+T, `--theme`/`--language` flags |
| `mouse-support` | all three | on | Mouse-wheel scrolling |
| `crossterm` | `wedi-widget` | on | crossterm backend |
| `ratatui` | `wedi-widget` | off | Optional ratatui dependency |

`cargo build --no-default-features` must compile.

## Flows

**Startup.** `Args::parse` → resolve `EncodingConfig` → `RopeBuffer` loads the file (encoding
detected unless given) → `HighlightEngine` picks syntax by extension, special filename, or
`--language` → `Editor::run`.

**Main loop.** Render (`View::render` with the highlight cache and `SearchHighlight`) → read
`InputEvent` (key or bracketed paste) → `handle_key_event` → execute the `Command` → scroll to
keep the cursor visible. Any buffer-modifying command exits **Search mode** and invalidates the
affected layout and highlight caches.

**Undo.** `History` keeps undo and redo stacks of groups of `Action`s (position + text), capped at
`max_size` groups. `Editor::handle_command` wraps each **Command** in `RopeBuffer::begin_group` /
`end_group`, so one command undoes in one step; consecutive single-character inserts merge into the
previous group until a new word starts. Each group has an id; the save point records the top id, and
Undo/Redo clear `[modified]` when they return to it. A new edit clears the redo stack.

**Syntax highlighting.** Scheduled from `editor.rs`: files up to `SMALL_FILE_THRESHOLD` (500) lines
are highlighted from the top; larger files start `BUFFER_LINES` (100) above the viewport. Results
are cached per line as ANSI strings in `HighlightCache`; character edits invalidate from the edited
line on (`invalidate_from_edit`), line-structure edits clear the whole cache.

**Save.** `RopeBuffer` encodes the rope with the save encoding (`encoding_rs`) and writes the file.

## Assets

`crates/wedi-core/assets/syntaxes.bin` — bat's serialized syntax set (219 syntaxes; MIT / Apache-2.0),
embedded via `include_bytes!` in `highlight::engine`.
