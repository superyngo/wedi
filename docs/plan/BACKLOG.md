# Backlog
Status: In progress

The one living record of open work. Rows move to Done with the commit that closed them and are
never deleted. Evidence is file + symbol, never a line number. `Verified` is the date the row was
last checked against the tree — not when it was opened.

## Open

| ID | Opened | Verified | Pri | Finding | Evidence | Effort | Acceptance |
|---|---|---|---|---|---|---|---|
| B1 | 2026-09-24 | 2026-09-24 | P2 | CI is disabled; fmt/clippy/tests are not enforced on push | `.github/workflows/ci.yml.disabled` | S | CI workflow runs fmt check, clippy `-D warnings`, and tests on push to `main` |
| B2 | 2026-09-24 | 2026-09-24 | P3 | No test checks that the help panel, `KEYMAP.md`, and `handle_key_event` agree | `src/help.rs` `get_help_sections`; `keymap::bindings::handle_key_event` | M | A test fails when a binding exists in one but not the other |
| B3 | 2026-09-24 | 2026-09-24 | P0 | A selection kept after Indent/Unindent/ToggleComment/Undo holds stale columns; Copy/Cut panics and release aborts, losing unsaved work | audit F1; `src/editor.rs` `Editor::get_selected_text`, `delete_selection` | S | Audit repro R1 copies without panicking; a regression test covers a selection outliving a line rewrite |
| B4 | 2026-09-24 | 2026-09-24 | P0 | Save rewrites encodings: UTF-16 comes out as UTF-8, the BOM is dropped, unmappable chars become `&#NNNN;` while the status bar says "File saved" | audit F2–F4; `RopeBuffer::save`, `save_to`, `save_as`, `from_file_with_encoding` | S | Unedited UTF-16LE/BE and UTF-8-BOM files save byte-identical; an unmappable char aborts the save with a status message |
| B5 | 2026-09-24 | 2026-09-24 | P0 | A non-UTF-8 file opened without `-f` is decoded lossily (U+FFFD) with no in-editor warning; saving corrupts it | audit F5; `RopeBuffer::from_file_with_encoding` | S | Opening such a file shows a status-bar warning; saving it asks for confirmation |
| B6 | 2026-09-24 | 2026-09-24 | P1 | Save is not atomic (`fs::write` truncates in place) | audit F6; `RopeBuffer::save` | M | Save writes a sibling temp file, then renames it; permissions and symlink targets are preserved |
| B7 | 2026-09-24 | 2026-09-24 | P1 | CRLF files: Backspace/Delete across a line break never join the lines on the first press; Enter, paste, and comment toggling insert LF | audit F7; `Editor::handle_command` `Backspace` / `Delete` / `ToggleComment`; `Terminal::read_input` | M | Audit repro R3 joins in one press; edits in a CRLF file keep CRLF |
| B8 | 2026-09-24 | 2026-09-24 | P1 | Search stores byte offsets that `Editor` uses as char columns; after Find on a CJK line, typing lands on the next line | audit F8; `Search::find_matches`; `Editor` `Find` / `FindNext` / `FindPrev` | S | Audit repro R2 saves `中文abcX` on line 1 |
| B9 | 2026-09-24 | 2026-09-24 | P1 | Cursor and layout desync: cache lookups for rows above the viewport return the top row; `visual_line_index` goes stale after Ctrl+O/Ctrl+L, resize, and direct field writes | audit F9, F11; `View::calculate_visual_lines_for_row`, `visual_to_logical_col`; `Cursor` | S–M | Audit probe P5 lands on the last visual line; audit repro R6 draws the cursor on its line |
| B10 | 2026-09-24 | 2026-09-24 | P1 | With syntax highlighting on (the default, including `.txt`), tabs print raw at 8-column stops while layout assumes 4; the cursor and wrapping misalign | audit F10; `LineHighlighter::ranges_to_ansi_optimized`; `utils::slice_ansi_text` | S | Audit repro R5: text ends at column 14 with highlighting on and off |
| B11 | 2026-09-24 | 2026-09-24 | P1 | Comment toggling converts indentation tabs to spaces and falls back to `#` for unknown types (JSON, HTML, CSS, Markdown) | audit F12, F13; `CommentHandler::detect_from_path`, `toggle_line_comment` | S | A Makefile tab line round-trips byte-identical; `.json` shows "No comment style" |
| B12 | 2026-09-24 | 2026-09-24 | P1 | Each render leaks a cloned `Theme` (~9 KB), about 1 GB per 100k keypresses | audit F14; `LineHighlighter::new` `Box::leak`; `HighlightEngine::create_highlighter` | XS | Audit repro R4: RSS stays within 1 MiB over 800 redraws |
| B13 | 2026-09-24 | 2026-09-24 | P1 | Prompts and the help panel drop bracketed paste and resize events | audit F15; `src/dialog.rs` `prompt_with_default`, `confirm`, `show_help` | S | Audit repro R7 shows `beta` in the prompt; resizing during a dialog redraws at the new size |
| B15 | 2026-09-24 | 2026-09-24 | P2 | Undo is one step per character; multi-line commands take N–2N steps; `[modified]` never clears on undoing to the save point | audit F17; `RopeBuffer::insert_char`; `History::push` | M | A typed word undoes in one step; a multi-line indent/comment undoes in one step; undoing to the save point clears `[modified]` |
| B16 | 2026-09-24 | 2026-09-24 | P2 | Highlighting re-parses every line above the viewport each frame (8.6 ms per 500 lines); large files start from a blank state; 33 scattered invalidation sites miss paste, undo, indent, and comment | audit F18, F25; `Editor::get_highlighted_lines`; `HighlightCache` | M | Frames reuse `ParseState`/`HighlightState` checkpoints; one `on_buffer_changed` hook invalidates; a block comment opened 150 lines above the viewport highlights correctly |
| B17 | 2026-09-24 | 2026-09-24 | P2 | `Editor` owns `Terminal`, so `handle_command` has no tests; B3, B7, B8, B9 live there | audit F26; `src/editor.rs` `Editor` | M | Command dispatch runs headless in unit tests; each fixed P0/P1 row has a regression test |
| B18 | 2026-09-24 | 2026-09-24 | P2 | Linux clipboard ignores exit status (over SSH, paste is empty and never falls back); Windows clipboard ignores API failures; AltGr chars are dropped on Windows | audit F19, F20; `ClipboardManager::set_text` / `get_text`; `handle_key_event` | S | A failed system clipboard falls back to the internal one; `CONTROL\|ALT` + char inserts (verify on Linux and Windows) |
| B19 | 2026-09-24 | 2026-09-24 | P2 | UX and render polish: any text ending in `\n` pastes as whole lines; stderr writes corrupt the TUI; a bad `--theme` silently disables highlighting; per-frame syscalls and clones | audit F21–F24; `Editor::paste_text`; `RopeBuffer` `eprintln!`; `Editor::new`; `View::render` | M | Each sub-item's audit fix is applied; one buffered flush per frame |
| B20 | 2026-09-24 | 2026-09-24 | P3 | `wedi-widget` is a re-export shell: `EditorConfig` is consumed by nothing, `ratatui` has no code, and AGENTS.md / ARCHITECTURE.md describe it as the config path | audit F27; `crates/wedi-widget` | M | The crate either drives `View`/`Editor` config or is folded into `wedi-core`; both docs updated |
| B21 | 2026-09-24 | 2026-09-24 | P3 | ~400 lines of dead code hidden by blanket `#[allow(dead_code)]` (including the inert `mouse-support` feature); width/tab logic copied 9×; fake F-key events; misleading names; edge-case underflows | audit F28–F32 | M | No `#[allow(dead_code)]` on used items; one width helper; `InputEvent::Resize`; the F32 edge cases fixed |

## Pending verification

Landed, but the check needs a platform or pipeline not available locally.

| Item | Closed by | Verifies when | Fallback |
|---|---|---|---|

## Awaiting external

Blocked on a person or third party. **Not counted as open**.

| Item | Blocked on | Ready when |
|---|---|---|
| WinGet publishing | winget-pkgs package removal / repo rename | `.github/workflows/winget.yml.disabled` can be re-enabled |

## Watching

Known, deliberately not scheduled.

| Item | Why not now | Trigger | Re-read |
|---|---|---|---|
| Widget documentation and usage examples | Blocked on B20 (keep or fold the crate) | B20 decided "keep" | 2026-09-24 |
| More syntax themes | 7 syntect built-ins cover light and dark | User request | 2026-09-24 |
| Large-file performance beyond highlighting (highlighting is B16) | No measured regression outside highlighting | A reported slowdown with a reproducible file | 2026-09-24 |
| LSP support, multi-cursor, split view, plugins | Long-term ideas; conflict with the minimalist scope | A spec in `docs/spec/` | 2026-09-24 |
| CHANGELOG has no entries for tagged v0.1.5–v0.1.18, v0.5.1, v0.8.1 | Content can't be reconstructed without re-reading each diff | Someone needs those releases' notes | 2026-09-24 |
| Dependency drift: crossterm 0.27, `unicode-width` 0.1, `once_cell`, `winapi`, unused direct `serde` (audit F33) | Nothing broken; the crossterm bump touches every event path | A needed upstream fix, or starting B17 / B21 | 2026-09-24 |

## Done

| ID | Finding | Closed by |
|---|---|---|
| B14 | Clippy `-D warnings` failed (`items_after_test_module`; unused `mut` without default features) | `fix: make clippy -D warnings pass` (2026-09-24) |
