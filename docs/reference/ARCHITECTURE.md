# Architecture

How wedi is put together. Vocabulary: [glossary.md](glossary.md). Bindings: [KEYMAP.md](KEYMAP.md).
Flags: [CLI.md](CLI.md).

## Workspace

| Crate | Path | Role |
|---|---|---|
| `wedi` | `src/` | The editor binary: argument parsing, editor loop, dialogs, help panel |
| `wedi-core` | `crates/wedi-core/` | Reusable editor primitives; no knowledge of the binary |

`wedi-widget` (a separate embedding crate up to v0.10.0) was folded into `wedi-core`: its
`EditorConfig` and `ScreenLayout` now live in `wedi_core::config` and `wedi_core::screen_layout`.
Nothing in the editor consumes them yet (backlog B21).

`examples/basic_usage.rs` shows `wedi-core` used standalone.

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
| `config` | `EditorConfig` | Display options for embedders (line numbers, wrap, tab width, theme); not yet read by `View` |
| `screen_layout` | `ScreenLayout` | Viewport size and scroll offset for embedders; independent of `View` |
| `clipboard` | `ClipboardManager` | **System clipboard**: Win32 API on Windows, pbcopy/pbpaste on macOS, wl-copy/wl-paste then xclip on Linux |
| `highlight` | `HighlightEngine`, `HighlightCache`, `LineState` | syntect engine, parser checkpoints and per-line ANSI cache (feature-gated) |
| `terminal` | `Terminal`, `InputEvent` | crossterm wrapper: raw mode, bracketed paste, size |
| `utils` | `ansi_slice`, `line_wrapper` | Width-aware slicing of ANSI-coloured text, wrapping |

## Cargo features

| Feature | Crates | Default | Effect |
|---|---|---|---|
| `syntax-highlighting` | `wedi`, `wedi-core` | on in `wedi` | syntect + bundled syntax set, Ctrl+T, `--theme`/`--language` flags |
| `mouse-support` | `wedi`, `wedi-core` | on | Mouse-wheel scrolling |

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

**Syntax highlighting.** `HighlightCache::highlight_rows` saves the syntect parser state
(`LineState`) every `CHECKPOINT_INTERVAL` (64) lines and resumes from the nearest checkpoint above
the viewport, so multi-line comments and strings are right at any scroll position. Highlighted lines
are cached as ANSI strings near the viewport. Every `RopeBuffer` edit (including Undo/Redo and
reload) records the first changed row; `Editor::get_highlighted_lines` takes it
(`take_changed_from`) before each render and calls `invalidate_from`, the only invalidation path.

**Save.** `RopeBuffer` encodes the rope with the save encoding (`encoding_rs`) and writes the file.

## Assets

`crates/wedi-core/assets/syntaxes.bin` — bat's serialized syntax set (219 syntaxes; MIT / Apache-2.0),
embedded via `include_bytes!` in `highlight::engine`.
