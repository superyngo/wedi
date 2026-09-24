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
| B15 | 2026-09-24 | 2026-09-24 | P2 | Undo is one step per character; multi-line commands take N–2N steps; `[modified]` never clears on undoing to the save point | audit F17; `RopeBuffer::insert_char`; `History::push` | M | A typed word undoes in one step; a multi-line indent/comment undoes in one step; undoing to the save point clears `[modified]` |
| B16 | 2026-09-24 | 2026-09-24 | P2 | Highlighting re-parses every line above the viewport each frame (8.6 ms per 500 lines); large files start from a blank state; 33 scattered invalidation sites miss paste, undo, indent, and comment | audit F18, F25; `Editor::get_highlighted_lines`; `HighlightCache` | M | Frames reuse `ParseState`/`HighlightState` checkpoints; one `on_buffer_changed` hook invalidates; a block comment opened 150 lines above the viewport highlights correctly |
| B17 | 2026-09-24 | 2026-09-24 | P2 | `Editor` owns `Terminal`, so `handle_command` has no tests; B3, B7, B8, B9 live there. Progress: headless harness landed (`Editor::with_terminal` + `Terminal::with_size`, `src/editor.rs` `mod tests`); B3, B4, B5, B6, B7, B8, B9, B10, B11 have regression tests. Dialogs (`src/dialog.rs`, B13) still read crossterm events directly and are verified only on the real binary | audit F26; `src/editor.rs` `Editor` | M | Command dispatch runs headless in unit tests; each fixed P0/P1 row has a regression test |
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
| B12 | Each render leaked a cloned `Theme` (~9 KB); R4 now +368 KiB over 800 redraws (was +7,664 KiB) | `fix: stop leaking a syntax theme per render` (2026-09-24) |
| B3 | Stale selection after Indent/Unindent/ToggleComment/Undo panicked on Copy/Cut; block ops now reselect whole lines, Undo/Redo clear the selection, and selection bounds are clamped | `fix: stale selection no longer crashes copy/cut` (2026-09-24) |
| B4 | Save wrote UTF-16 as UTF-8, dropped the BOM, and wrote unmappable chars as `&#NNNN;`; now byte-identical round trip and a failing save for unmappable chars | `fix: save keeps UTF-16, BOM, and refuses unmappable chars` (2026-09-24) |
| B8 | Search jumps used byte offsets as char columns; `Editor::jump_to_match` converts them; R2 saves `中文Xabc` | `fix: search jumps to the match's char column` (2026-09-24) |
| B9 | Layout cache lookups for rows above the viewport returned the top row; `visual_line_index` went stale after Ctrl+O/Ctrl+L/resize/Undo/Redo. `View::cached_layout` uses `checked_sub`; `Editor::resync_cursor` and `Cursor::set_position` resync. P5 test and R6 pass | `fix: keep cursor visual line in sync with layout` (2026-09-24) |
| B10 | Highlighted lines emitted raw `\t` (terminal 8-column stops) while layout used 4; `LineHighlighter` now expands tabs to `view::TAB_WIDTH`. R5 ends at column 14 with highlighting on and off | `fix: expand tabs in highlighted lines to match layout` (2026-09-24) |
| B7 | CRLF line breaks were deleted one char at a time and new breaks were LF. Backspace/Delete now delete the whole break; Enter, paste and comment toggling use `RopeBuffer::line_ending` / the line's own ending. R3 joins in one press | `fix: CRLF files join in one press and keep CRLF on edit` (2026-09-24) |
| B5 | Lossy decoding (U+FFFD) was silent. `RopeBuffer::is_lossy` drives a status-bar warning on open/reload; Save needs a second consecutive Ctrl+S (same pattern as Ctrl+Q) | `fix: warn on lossy decode and confirm before saving it` (2026-09-24) |
| B6 | Save truncated the file in place. `write_atomic` writes a sibling temp file, fsyncs, and renames over the target; symlinks write through to their target, permissions are copied, read-only files are refused. Hard links to the old inode are not updated | `fix: save atomically via temp file and rename` (2026-09-24) |
| B11 | Comment toggling rebuilt indentation as spaces and guessed `#` for every unknown extension. `CommentHandler` keeps the original leading whitespace; unknown extensions have no style (no extension still maps to `#`). Makefile tab line round-trips; `.json` shows "No comment style" | `fix: keep tabs when toggling comments, no guess for unknown types` (2026-09-24) |
| B13 | Dialog loops read only key events, so bracketed paste and resize were dropped. `prompt_with_default` inserts the first line of a paste; the prompt, confirm and help loops redraw at the new size on resize; `dialog::take_resized` makes the editor run `Resize` after the dialog. R7 shows `beta`; verified on the real binary (no unit test, see B17) | `fix: dialogs accept bracketed paste and follow resizes` (2026-09-24) |
