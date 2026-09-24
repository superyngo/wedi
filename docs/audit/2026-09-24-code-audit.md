# Code audit
Status: Resolved (2026-09-24) — B18 awaits verification on Linux/Windows

Whole-codebase sweep for bugs, better implementations, performance, simplicity, and clarity, at
commit `4a76499` (v0.10.0 + docs). Evidence is file + symbol, never a line number. Findings are
scheduled in [`../plan/BACKLOG.md`](../plan/BACKLOG.md) (rows B3–B21); this file stays frozen once
they are addressed.

## Method

- Read all 30 `.rs` files (7,264 lines) across `wedi`, `wedi-core`, `wedi-widget`, and `examples/`.
- `cargo clippy --workspace --all-targets -- -D warnings`, with and without default features, plus
  a `clippy::pedantic` / `clippy::nursery` pass (leads only — most pedantic hits are style).
- **Library probes** (P1–P11): a throwaway crate linking `wedi-core` by path, run in release. Kept at
  `docs/tmp/claude-scratch/audit-probe/` (gitignored); each probe is restated under its finding.
- **Real-binary repros** (R1–R7): `target/debug/wedi` driven through a private tmux server
  (`tmux -L wediaudit`), reading back the saved file, the pane, and `#{cursor_x},#{cursor_y}`.
- Findings marked *by inspection* were not executed; *pending verification* ones need Linux or
  Windows.

## Summary

The architecture is sound: a rope buffer, a headless core crate, careful CJK width handling, a
panic hook that restores the terminal, and clean feature gating. The defects cluster in four places:

1. **The save path silently rewrites files.** UTF-16 files come back as UTF-8, BOMs are dropped,
   and unmappable characters become `&#NNNN;` entities — each reported as "File saved".
2. **`Editor` mutates buffer, cursor, and selection without keeping them consistent.** A stale
   selection crashes the process and loses unsaved work; a stale cursor layout draws the cursor on
   the wrong row. None of `Editor`'s 1,474 lines are under test.
3. **Column units are mixed.** Byte offsets (search), char columns (cursor), and visual columns
   (render) cross boundaries unconverted; tabs are 4 wide in layout and 8 wide on screen whenever
   syntax highlighting is on — the default.
4. **Every render leaks ~9 KB** and re-highlights every line above the viewport.

| Priority | Count | Meaning |
|---|---|---|
| P0 | 5 | Crash or silent data corruption |
| P1 | 11 | Wrong behavior in normal use, or a red quality gate |
| P2 | 11 | Performance, edge-case UX, or structure that invites the P0/P1 classes |
| P3 | 7 | Cleanup, dead code, clarity |

## P0 — crash and data integrity

### F1 Stale selection panics; release builds abort and lose unsaved work
- **Where:** `src/editor.rs` `Editor::get_selected_text` slices `chars[start_col..]` with stored
  selection columns; `Indent`, `Unindent`, and `ToggleComment` keep the selection ("保留選擇狀態")
  after rewriting its lines. `Undo` and `Redo` never clear it.
- **Evidence (R1):** file `"    ab\nline2\n"`, keys End, Shift+Down, Shift+Tab, Alt+C → `panicked at
  src/editor.rs … range start index 6 out of range for slice of length 2`. Release uses
  `panic = "abort"`, so the buffer is gone. `delete_selection` has the same stale columns and can
  delete across into the next line instead of panicking.
- **Fix:** clamp both ends to the line length in `get_selected_text` / `delete_selection`, and
  remap or clear `selection` after every buffer mutation. **Effort:** S.

### F2 UTF-16 files are saved as UTF-8
- **Where:** `crates/wedi-core/src/buffer/rope_buffer.rs` `RopeBuffer::save` (and `save_to`,
  `save_as`) call `encoding_rs::Encoding::encode`. For UTF-16LE/BE, encoding_rs's *output
  encoding* is UTF-8; the returned encoding (second tuple element) is discarded.
- **Evidence (P1):** `FF FE 48 00 65 00 …` ("Hello" UTF-16LE) → insert "X" → save →
  `58 48 65 6C 6C 6F` (UTF-8, no BOM), while `save_encoding()` still reports `UTF-16LE`. The
  `-e/-t utf-16le` flags cannot produce UTF-16 at all.
- **Fix:** encode UTF-16 by hand (`encode_utf16` + `to_le_bytes`/`to_be_bytes`) and write the BOM.
  **Effort:** S.

### F3 BOM is dropped on save
- **Where:** `RopeBuffer::from_file_with_encoding` strips the BOM (`detect_unicode` → `bom_length`)
  and does not record that it existed; `save` never writes one.
- **Evidence (P2):** `EF BB BF 68 69` → open → save → `68 69`. This breaks tools that need the BOM
  (PowerShell 5.1 scripts, Excel CSV).
- **Fix:** store `has_bom` on `RopeBuffer` and emit it on save. **Effort:** S.

### F4 Unmappable characters become HTML entities, reported as success
- **Where:** `RopeBuffer::save`: encoding_rs replaces unmappable characters with decimal numeric
  character references; `had_errors` only triggers an `eprintln!`, which is invisible under the
  alternate screen. `Editor::handle_command` `Command::Save` then shows "File saved".
- **Evidence (P3):** GBK buffer containing `a😀b` → `save_to` returns `Ok`, file holds
  `a&#128512;b`.
- **Fix:** when `had_errors`, do not write; return an error naming the encoding so the status bar
  shows it. **Effort:** S.

### F5 Lossy decode is accepted silently
- **Where:** `RopeBuffer::from_file_with_encoding`: a file that is not valid UTF-8, opened without
  `-f`, is decoded with `get_system_ansi_encoding()` — UTF-8 on macOS/Linux — so every invalid byte
  becomes U+FFFD; `had_errors` only `eprintln!`s before the TUI clears the screen. Saving writes the
  U+FFFD back, irreversibly corrupting e.g. Latin-1 files.
- **Evidence:** by inspection.
- **Fix:** flag the buffer as lossy, say so in the status bar on open, and require confirmation (or
  `-f`) before saving over the original. **Effort:** S–M.

## P1 — wrong behavior in normal use

### F6 Save is not atomic
- **Where:** `RopeBuffer::save` uses `std::fs::write` (truncate, then write). Disk-full, a kill, or
  a crash mid-write leaves a truncated file.
- **Fix:** write a sibling temp file, `sync_all`, rename over the target; copy permissions and
  resolve symlinks first. **Effort:** M. *By inspection.*

### F7 CRLF files: Backspace/Delete across a line break are broken, and edits mix line endings
- **Where:** `Editor::handle_command` `Command::Backspace` (row > 0, col == 0) deletes the single
  char at `line_to_char(prev) + prev_line_len`, where the length excludes `"\r\n"` — that char is
  `\r`. `Command::Delete` at end of line does the same. Enter inserts `'\n'`,
  `Terminal::read_input` normalizes pastes to `\n`, and the `ToggleComment` rebuild rewrites
  `"\r\n"` to `"\n"`.
- **Evidence (R3):** `abc\r\ndef\r\n`, Down, Backspace, save → `abc\ndef\r\n` (not joined, CR
  stripped). A second Backspace → `ab\ndef\r\n`: it deletes `c`, and the lines never join.
- **Fix:** detect the dominant line ending on load (store on `RopeBuffer`); delete a line break as
  one unit; insert the detected ending on Enter, paste, and line rebuilds. **Effort:** M.

### F8 Search matches are byte offsets but used as char columns
- **Where:** `crates/wedi-core/src/search.rs` `Search::find_matches` stores the `str::find` byte
  index; `SearchHighlight` documents bytes and `view::byte_col_to_visual_col` converts correctly for
  rendering, but `Command::Find`, `FindNext`, and `FindPrev` assign the byte index to `cursor.col`.
- **Evidence (P4, R2):** line `中文abc`: match `(0, 6)`, line length 5 chars. Ctrl+F "abc", Enter,
  type X, save → `中文abc\nXxyz\n`; the X lands on the next line.
- **Fix:** store char columns and the query's char length; convert once in `find_matches`.
  **Effort:** S.

### F9 Layout cache returns the wrong row for rows above the viewport
- **Where:** `crates/wedi-core/src/view.rs` `View::calculate_visual_lines_for_row` and
  `View::visual_to_logical_col` index the cache with `row.saturating_sub(self.offset_row)`, which is
  `0` — the *top visible row* — for any `row < offset_row`. `Cursor::move_up`, `move_left`
  (wrapping to the previous line), and `move_page_up` all query rows above the viewport.
- **Evidence (P5):** row 0 wraps to 3 visual lines, viewport starts at row 1; the lookup for row 0
  returns `["short"]` (1 line), and `move_up` from row 1 lands on visual line 0 instead of 2.
  PageUp counts every row above the screen with the top row's height.
- **Fix:** `row.checked_sub(self.offset_row)`; skip the cache on `None`. **Effort:** XS.

### F10 Tabs render at 8 columns under syntax highlighting while layout assumes 4
- **Where:** `LineHighlighter::ranges_to_ansi_optimized` (`highlight/engine.rs`) passes raw `\t`
  through; `View::render` prints it via `slice_ansi_text`, which counts `\t` as width 1. The
  terminal then jumps to the next 8-column tab stop. Layout, cursor, and wrap use `TAB_WIDTH = 4`
  (`expand_tabs_and_build_map`). `.txt` also resolves to syntect's Plain Text syntax, so this hits
  every file with tabs by default.
- **Evidence (R5):** `\tfoo := 1`, End: the cursor is at x=14 both with highlighting on and off,
  but the text ends at column 16 when it's on (raw TAB) and at 14 when it's off (expanded). Wrapped
  lines containing tabs are also sliced at the wrong offsets.
- **Fix:** expand tabs (same rule as `expand_tabs_and_build_map`) before highlighting or while
  building the ANSI string. **Effort:** S.

### F11 Cursor layout state goes stale
- **Where:** `Cursor.visual_line_index` and `desired_visual_col` are recomputed only inside
  `Cursor::set_position` and the `move_*` methods. `Editor` writes `cursor.row`, `cursor.col`, and
  `desired_visual_col = col` (a char column, not a visual one) directly in `Undo`, `Redo`, `Find*`,
  `GoToLine`, `SelectAll`, `Indent`, `Unindent`, `paste_text`, and `do_cut`. Nothing resyncs after
  `View::toggle_display_mode`, `toggle_line_numbers`, or a resize.
- **Evidence (R6):** a 250-char line, End (cursor on visual line 3, screen row 2), Ctrl+O →
  the cursor is drawn at screen row 2 while the line is on row 0.
- **Fix:** make `Cursor` fields private, route placement through `set_position`, and resync after
  any layout change. **Effort:** S–M.

### F12 Comment toggle converts indentation tabs to spaces
- **Where:** `crates/wedi-core/src/comment.rs` `CommentHandler::toggle_line_comment`,
  `add_comment`, and `remove_comment` rebuild indentation as `" ".repeat(byte_len)`.
- **Evidence (P9):** Makefile `"\tcc -o x"` → `" # cc -o x"`; uncommenting yields a space-indented
  recipe, which `make` rejects. U+3000 (3 bytes) becomes 3 spaces.
- **Fix:** reuse the original leading-whitespace slice; implement `toggle` as `is_commented` ?
  `remove` : `add` (three copies of the same logic today). **Effort:** S.

### F13 Unknown file types get `#` comments
- **Where:** `CommentHandler::detect_from_path` defaults `_ => "#"`, so `has_comment_style()` is
  always true and the "No comment style for this file type" branch in `Editor` is unreachable.
- **Evidence (P10):** `.json` `{` → `# {`; `.html` `<p>` → `# <p>`. The same applies to `.css`,
  `.md`, `.xml`, and `.txt`.
- **Fix:** default to `None`; `CommentStyle::Block` exists but is never used. **Effort:** S.

### F14 Every render leaks a cloned theme (~9 KB)
- **Where:** `LineHighlighter::new` does `Box::leak(Box::new(theme))` on a `Theme` cloned by
  `HighlightEngine::create_highlighter`, which `Editor::get_highlighted_lines` calls every frame.
  The comment's premise ("theme 數量很少") is wrong: it leaks one theme per render, not per theme.
- **Evidence (P6, R4):** 100 × `create_highlighter` → +870,700 live bytes (8.7 KB each). On the
  binary: RSS 8,512 → 16,176 KiB after 800 redraws (≈9.8 KB each), i.e. ~1 GB per 100k keypresses.
- **Fix:** `THEME_SET` is already a `static`; hold `&'static Theme` from `THEME_SET.themes.get(..)`
  in `HighlightEngine` — no clone, no leak. **Effort:** XS.

### F15 Dialogs swallow bracketed paste and resize
- **Where:** `src/dialog.rs` `prompt_with_default`, `confirm`, and `show_help` call
  `crossterm::event::read` directly and drop `Event::Paste` and `Event::Resize`.
- **Evidence (R7):** Ctrl+F, `tmux paste-buffer -p "beta"` → prompt stays ` Search:`. A resize while
  a dialog is open leaves `View` at the old size until the next resize.
- **Fix:** insert paste text at the prompt cursor; on resize, update the size and redraw.
  **Effort:** S.

### F16 `cargo clippy -- -D warnings` fails
- **Where:** `clippy::items_after_test_module`: `impl Default for RopeBuffer` sits after
  `mod tests` in `rope_buffer.rs`. With `--no-default-features`, `src/help.rs`
  `get_help_sections` also warns on two unneeded `mut` (`navigation`, `code`).
- **Evidence:** the clippy run above. The gate AGENTS.md requires is red; CI is off (B1).
- **Fix:** move the impl above the tests; build the two `Vec`s with `#[cfg]`-gated elements
  instead of `mut` + `push`. **Effort:** XS.

## P2 — performance, edge cases, structure

### F17 Undo is per character and never returns to "unmodified"
- **Where:** `RopeBuffer::insert_char` pushes one `Action` per keystroke; multi-line commands
  (`ToggleComment`, `Indent`, `Unindent`, `DeleteLine` with a selection) push N–2N actions;
  replace-selection pushes 2. `History::push` evicts with `Vec::remove(0)` (O(n) at the 1,000 cap,
  which is only ~1,000 keystrokes). `modified` stays true after undoing back to the save point.
- **Evidence (P7):** typing `hello` takes 5 undos.
- **Fix:** coalesce adjacent inserts/deletes; `begin_group` / `end_group` for compound commands;
  `VecDeque`; record a save-point index. **Effort:** M.

### F18 Highlight cache neither saves work nor invalidates on every edit
- **Where:** `Editor::get_highlighted_lines` calls `LineHighlighter::highlight_line` (parse + ANSI
  build) for every row from `process_start` to the viewport bottom on every frame, cached or not.
  Files ≤ 500 lines restart from row 0. Large files restart from `start_row − 100` with a fresh
  state, so an unclosed block comment or string further up is mis-highlighted. Separately, paste,
  undo/redo, `delete_selection`, toggle comment, indent/unindent, and encoding reload never touch
  `highlight_cache`. Because `HighlightCache::is_valid` compares only the line's own text, lines
  below an edit that opens or closes a block comment keep stale colors.
- **Evidence (P11):** 8.6 ms per frame to highlight 500 lines (release, M4). The cache comment says
  syntect's `ParseState` is private; in syntect 5.3 `syntect::parsing::ParseState` and
  `syntect::highlighting::HighlightState` are public and `Clone`.
- **Fix:** checkpoint `(ParseState, HighlightState)` every N lines and resume from the nearest
  checkpoint above the viewport, formatting only visible lines. Invalidate checkpoints ≥ the edited
  row from one `Editor::on_buffer_changed(row)` hook. **Effort:** M.

### F19 Linux clipboard ignores exit status; Windows ignores API failures
- **Where:** `crates/wedi-core/src/clipboard.rs` `ClipboardManager::set_text` / `get_text` treat a
  spawned `wl-copy`, `wl-paste`, `xclip`, `pbcopy`, or `pbpaste` as success without checking
  `status.success()`. Over SSH with xclip installed and no display, `get_text` returns `Ok("")`, so
  Ctrl+V pastes nothing and never falls back to the internal clipboard; when `wl-copy` exists but
  fails, xclip is never tried. On Windows the `OpenClipboard` / `SetClipboardData` results are
  unchecked (`h_mem` leaks on failure). `is_available()` is hard-wired `true`, making the fallback
  messages in `Editor::set_clipboard_text` / `get_clipboard_text` unreachable.
- **Evidence:** by inspection; *pending verification* on Linux and Windows.
- **Fix:** check exit status and API results; map failure to `Err` so the internal clipboard is
  used; delete `is_available`. **Effort:** S.

### F20 AltGr characters are dropped on Windows
- **Where:** `keymap::bindings::handle_key_event` inserts `KeyCode::Char` only with `NONE` or
  `SHIFT`. crossterm reports AltGr as `CONTROL | ALT` on Windows, so `@ { [ \ ] } €` on German,
  French, or Polish layouts produce nothing.
- **Evidence:** by inspection; *pending verification* on Windows.
- **Fix:** treat `CONTROL | ALT` + `Char` as `Insert`. **Effort:** XS.

### F21 Whole-line paste applies to any text ending in `\n`
- **Where:** `Editor::paste_text` treats any text ending in `'\n'` as a line paste and inserts it
  at the line start. That is right for wedi's own no-selection copy, but wrong for multi-line text
  pasted from elsewhere (system clipboard or bracketed paste), which lands above the current line
  instead of at the cursor.
- **Fix:** remember whether the internal clipboard holds a line copy, and apply line semantics only
  then. **Effort:** S. *By inspection.*

### F22 Warnings and debug output are written to stderr while the TUI is up
- **Where:** `RopeBuffer::save` / `save_to` / `save_as` / `from_file_with_encoding` (reached from
  Ctrl+E reload) `eprintln!` and `debug_log!`, and so does `LineHighlighter::highlight_line`. That
  text lands on the alternate screen. It happens always in debug builds, and in release with
  `--debug`. Messages printed before the TUI starts (`-l` fallback warning, decode warnings) are
  erased by `Terminal::clear_screen`.
- **Evidence:** R1's pane capture shows `[DEBUG]` lines printed.
- **Fix:** return warnings to `Editor.message`; write debug logs to a file. **Effort:** S.

### F23 An invalid `--theme` silently disables highlighting
- **Where:** `Editor::new` uses `HighlightEngine::new(..).ok()`.
- **Fix:** fail at argument parsing with the `--list-themes` hint, the way `-l` warns.
  **Effort:** XS.

### F24 Rendering does avoidable work per frame
- **Where:**
  - `View::render` and `render_status_bar` `queue!` into `io::stdout()` (a 1 KiB line buffer) with
    several `execute!` flushes: many write syscalls per frame, which flickers over SSH.
  - For each visible row, `row_matches` filters *all* search matches: O(rows × M).
  - `RopeBuffer::file_display_path` calls `canonicalize()` (a syscall) every frame.
  - `LineLayout` is cloned per row in `render` and `get_cursor_visual_position`.
  - `Cursor::line_len` / `update_*` convert the line to a `String` 2–3 times per keystroke.
  - `RopeBuffer::detect_unicode` decodes the whole file into a `String` only to test UTF-8
    validity, then decodes again.
- **Fix:**
  - One `BufWriter` over `stdout().lock()` per frame, flushed once.
  - `partition_point` on the row-sorted matches.
  - Cache the display path at load and save.
  - Borrow layouts instead of cloning.
  - Count chars on the `RopeSlice`.
  - `std::str::from_utf8` for the validity check.
- **Effort:** S each.

### F25 `Editor::handle_command` is a 669-line match with copy-pasted blocks
- **Where:** `src/editor.rs`:
  - The comment-toggle line rebuild appears twice (single-line and selection paths).
  - The `ChangeEncoding` reload-and-reset block appears twice, verbatim.
  - The cursor-reset sequences are repeated.
  - Cache invalidation is spread over 33 call sites, the syntax-cache ones each with its own
    `#[cfg]`.
  - F1, F11, and F18 all come from these sites not keeping selection, cursor, and caches in step.
- **Fix:**
  - Per-command methods.
  - `replace_line(row, text)` and `reload(encoding)` helpers.
  - One `on_buffer_changed(from_row)` that invalidates both caches and fixes up the selection.
- **Effort:** M.

### F26 `Editor` is untestable, and has no tests
- **Where:** `Editor` owns `Terminal`, so `handle_command` cannot run headless; `src/editor.rs`
  has zero tests and there is no `tests/` directory. F1, F7, F8, and F11 all live there. The root
  dev-dependencies `assert_cmd` and `predicates` are unused.
- **Fix:** split the editing state (buffer, cursor, selection, search, clipboard, command dispatch)
  from terminal I/O. Add a regression test per fixed finding. **Effort:** M.

### F27 `wedi-widget` is a re-export shell
- **Where:**
  - `crates/wedi-widget`: nothing consumes `EditorConfig` (`tab_width`, `theme`, `wrap_mode`).
    `View` uses the constant `TAB_WIDTH`, and the widget's default theme (`base16-ocean.dark`)
    differs from the app's (`base16-eighties.dark`).
  - `ScreenLayout` is unrelated to `View`, and the `ratatui` feature has no code.
  - The binary depends on the crate but uses nothing from it (`src/lib.rs` only re-exports).
  - `AGENTS.md` "Adding a Feature" step 2 and `docs/reference/ARCHITECTURE.md` describe
    `EditorConfig` as the config path, which is inaccurate today.
- **Fix:** decide whether to make it a real widget (config drives `View` and `Editor`) or fold it
  into `wedi-core`, then correct those two docs. **Effort:** M (a decision first).

## P3 — cleanup and clarity

### F28 Dead code, about 400 lines, partly hidden by blanket `#[allow(dead_code)]`
- **`mouse-support` feature:** `EnableMouseCapture` is never sent, so `Event::Mouse` never arrives.
  `handle_mouse_event` and the F22/F23 bindings are dead. The wheel works only because terminals
  send arrow keys in the alternate screen. `docs/reference/ARCHITECTURE.md` and `KEYMAP.md` credit
  the feature for it.
- **Terminal:** `read_key` (duplicates `read_input`), `set_cursor_position`, `hide_cursor`, and
  `flush`.
- **Utils and view:** `utils::LineWrapper`, `char_width`, `ansi_visual_width` (test-only),
  `view::calculate_hash`, and `View::invalidate_lines`.
- **Buffer:** `RopeBuffer::save_to` / `save_as` (near-copies of `save`), `can_undo` / `can_redo`,
  and the commented-out `from_file` and `EncodingSpec`.
- **Highlighting:** `HighlightCache::invalidate`, `invalidate_range`, `stats`, and the unused
  `EditType` variants. `HighlightEngine::detect_syntax_from_content` is unused, though shebang
  detection would help extensionless scripts.
- **Editor and CLI:** `Editor.highlight_config`, `Args.list_themes`, and `Command::ClearSelection`
  (no key binding).
- **Stale allows:** `#[allow(dead_code)]` on items that *are* used (`dialog::*`,
  `handle_key_event`, `Search`, `CommentHandler`, `Command`) stops rustc from reporting the real
  dead code above.

### F29 Character-width and tab logic is duplicated in 9 places
- **Where:** `view::expand_tabs_and_build_map`, `byte_col_to_visual_col`,
  `View::logical_col_to_visual_col`, the `visual_to_logical_col` fallback, `slice_visible_text`,
  `wrap_line`, `utils::visual_width`, `dialog::truncate_to_width`, and `slice_ansi_text`.
- The copies already disagree about tabs (F10).
- **Fix:** one `display_width(ch)` in `utils`.

### F30 Non-key events are encoded as fake function keys
- **Where:** `Terminal::read_input` sends a resize as `KeyCode::F(21)` and the wheel as
  `F(22)` / `F(23)`.
- A real F21 key (Shift+F9 on some terminals) triggers a resize.
- **Fix:** `InputEvent` already exists; add a `Resize` variant.

### F31 Misleading names and double work
- `View::render_status_bar`'s `selection_mode` parameter receives `selection.is_some()`.
- The debug ruler is detected with `message.starts_with("DEBUG")`, although `Editor` knows
  `debug_mode`.
- `View::render` calls `scroll_if_needed` again after `Editor::run` already did.
- `quit_times: u8` is used as a bool.

### F32 Small edge cases
- `View::scroll_horizontal_if_needed` computes `available_width - HORIZONTAL_SCROLL_MARGIN`, which
  underflows below 5 columns (a debug-build panic). The dialogs' `cols as usize - 1` underflows at
  0 columns.
- `DeleteLine` on a last line without a trailing newline leaves an empty line behind.
- Ctrl+H (help) collides with terminals that send `^H` for Backspace or Ctrl+Backspace.
- Search always starts at the top of the file, not at the cursor.
- A nonexistent file opens as already `modified`, so quitting an untouched new file asks for
  confirmation.

### F33 Dependency drift (not urgent)
- crossterm 0.27 (current 0.29), `unicode-width` 0.1, `once_cell` (std `LazyLock` covers it), and
  `winapi` (unmaintained; `windows-sys`).
- `wedi-core` depends on `serde` unconditionally but never uses it directly.

### F34 Clippy pedantic leads worth taking
- `too_many_lines`:
  - `handle_command`: 669 lines
  - `View::render`: 259
  - `handle_key_event`: 173
  - `get_system_ansi_encoding`: 153 lines, mostly `cfg!(debug_assertions)` prints around a
    code-page table
- `unused_self` (`Cursor::line_len`, `View::slice_visible_text`,
  `HighlightEngine::detect_syntax_from_path`).
- `redundant_clone` (`src/main.rs` `print_themes`, `HighlightCache` tests).
- `match_same_arms` in `handle_key_event`.
- The `usize as u16` casts in `View` and `dialog` are bounded by the terminal size (benign).
- `suspicious_operation_groupings` in `View::scroll_if_needed` is a false positive.

### F35 Positive notes (keep)
- A rope buffer with char-indexed edits, and CJK width handled throughout.
- The panic hook plus `Drop for Terminal` restore the terminal on panic and on early return.
- Features are cleanly gated: `--no-default-features` builds.
- The syntax set is embedded, so highlighting is deterministic with no runtime assets.
- The keymap is centralized in one function, and the help panel is data-driven
  (`get_help_sections`).

## Prioritized action plan

| Horizon | Items |
|---|---|
| Quick wins (< 1 day) | F14 leak, F16 clippy, F9 cache lookup, F1 crash, F2–F4 save integrity, F8 search columns, F10 tabs, F12–F13 comments, F15 dialog paste, F20 AltGr, F23 theme |
| Medium (1–5 days) | F7 CRLF, F11 cursor sync, F25 + F18 invalidation hook, F17 undo grouping, F6 atomic save, F5 lossy decode, F19 clipboard, F26 testable editor core, F21 paste semantics, F22 in-TUI warnings |
| Long term | F18 checkpointed highlighting, F27 widget decision, F28–F31 cleanup, F33 dependencies |

Suggested order: **F26's seam first** (small), so every later fix lands with a regression test,
then P0 and P1 top-down.

## Metrics

| Metric | Value |
|---|---|
| Rust files / lines | 30 / 7,264 (`src/editor.rs` 1,474; `view.rs` 1,127; `rope_buffer.rs` 890) |
| Unit tests | 33 in `wedi-core`, 3 in `src/dialog.rs`, 0 for `Editor`; doc tests in `wedi-widget` / `View` |
| Clippy `-D warnings` | fails (1 error; F16) |
| Clippy pedantic + nursery | ~270 hits, mostly `must_use` / `const fn` / format-args style |
| Confirmed by execution | 18 (P1–P11, R1–R7) |
| Complexity hotspots | 4 functions > 150 lines |

## Repro recipes

Assumptions: the debug binary, a UTF-8 locale, an 100×30 pane, and files created fresh in a temp
directory.

| ID | Input | Keys | Observed |
|---|---|---|---|
| R1 | `    ab\nline2\n` | End, Shift+Down, Shift+Tab, Alt+C | panic in `get_selected_text` |
| R2 | `中文abc\nxyz\n` | Ctrl+F `abc` Enter, `X`, Ctrl+S | file `中文abc\nXxyz\n` |
| R3 | `abc\r\ndef\r\n` | Down, Backspace, Ctrl+S (twice) | `abc\ndef\r\n`, then `ab\ndef\r\n` |
| R4 | copy of `src/editor.rs` as `big.rs` | 400 × (Down, Up) | RSS +7,664 KiB |
| R5 | `t.go` = `\tfoo := 1\n` | End; then Ctrl+T | cursor x=14 both times; text ends at 16 with highlighting on, 14 off |
| R6 | 250 × `y`, newline, `next` | End, Ctrl+O | cursor drawn on screen row 2; line is on row 0 |
| R7 | `alpha\nbeta\n` | Ctrl+F, bracketed paste `beta` | prompt stays empty |
