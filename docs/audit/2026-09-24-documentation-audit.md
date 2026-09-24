# Documentation audit
Status: Resolved (2026-09-24)

Two-pass sweep (structure, then accuracy against the code) of every document in the repo.

**Resolution.** All findings fixed in the same commit: `docs/` layout and `CONTEXT.md` created;
`CLAUDE.md` content moved to `docs/reference/ARCHITECTURE.md` with C1–C5 corrected; `AGENTS.md`
reduced to conduct; README R1–R9 fixed; CHANGELOG L1 fixed. S4 files removed. Deferred to
[`../plan/BACKLOG.md`](../plan/BACKLOG.md): S5 (B1 and the WinGet row), L4 (Watching). L2–L3 left as-is: released history.

## Structure (wens-dev-principles docs)
- S1 No `docs/` tree, no `CONTEXT.md`, no glossary, no BACKLOG (docs 1, 2, 5, 17).
- S2 Two agent instruction files: `CLAUDE.md` (zh-TW, 312 lines) is an architecture/flow reference;
  `AGENTS.md` (en, 232 lines) is conduct + a partial structure tree. They overlap and neither points
  at a docs index (docs 3).
- S3 `CLAUDE.md` §未來優化方向 is a roadmap/backlog living in an instruction file (docs 17).
- S4 Stale tracked files: `Cargo.toml.backup` (pre-workspace v0.7.0 manifest); root
  `assets/syntaxes.bin` is byte-identical to `crates/wedi-core/assets/syntaxes.bin` and unused —
  `highlight::engine` `SERIALIZED_SYNTAX_SET` includes the crate copy only.
- S5 `.github/workflows/ci.yml.disabled`, `winget.yml.disabled` — CI is off.
- S6 No `docs/tmp/*-scratch/` entry in `.gitignore` (docs 12).

## Accuracy — README.md
- R1 §Syntax Highlighting says **Ctrl+J** toggles highlighting; binding is **Ctrl+T** (`keymap::bindings` `handle_key_event`). Ctrl+J inserts a newline.
- R2 Duplicate `## Syntax Highlighting` / `### Syntax Highlighting` headings.
- R3 Keyboard section missing: Ctrl+E (change encoding), Ctrl+PageUp/Down (jump 1/10), mouse wheel.
- R4 ESC described as "Clear selection and messages"; actual: dismiss one layer per press (help.rs, CHANGELOG v0.10.0).
- R5 Usage missing flags: `-e/--encoding`, `-l/--language`, `--list-languages` (`Args::parse`).
- R6 Technical Stack lists `arboard` and `clap` — neither is a dependency (clipboard shells out to
  pbcopy / wl-copy / xclip; CLI uses `pico-args`). "ropey … with undo/redo" wrong — undo is `buffer::history::History`.
- R7 Bogus note "Replace `superyngo` with the actual GitHub superyngo in the URLs above."
- R8 Features line "Comment toggling (Ctrl+K/\\//)" garbled; 🎉 emoji on every line.
- R9 Third-party section cites `assets/syntaxes.bin` (the unused root copy).

## Accuracy — CLAUDE.md
- C1 Says CLI parsing uses `clap`; it is `pico-args`.
- C2 Lists `arboard` as a core dependency; not a dependency.
- C3 Undo/redo flow describes `actions: Vec<Action>` + `current` index; actual `History` has
  `undo_stack` / `redo_stack` / `max_size`.
- C4 Highlight large/small-file strategy (500 / ±100) is in `editor.rs` (`SMALL_FILE_THRESHOLD`,
  `BUFFER_LINES`), not `View::render` as the flow implies.
- C5 `wedi-widget` optional `ratatui` feature and `mouse-support` feature undocumented.

## Accuracy — AGENTS.md
- A1 §CI/CD Notes claims CI runs on 3 OSes and enforces clippy/fmt — `ci.yml` is disabled.
- A2 Project structure omits `view.rs`, `search.rs`, `comment.rs`, `clipboard.rs`, `terminal.rs`, `examples/`.

## Accuracy — CHANGELOG.md
- L1 `## [0.2.0] - 2024-XX-XX` — tag `v0.2.0` is dated 2025-11-27.
- L2 `## [0.1.0] - Initial Release` — no 0.1.0 tag; first tag `v0.1.5` 2025-11-08.
- L3 Heading style mixed: `[v0.10.0]` vs `[0.9.0]` and older.
- L4 No entries for tagged v0.5.1, v0.8.1 (and v0.1.5–v0.1.18). Record only; do not invent content.
- L5 Size (271 lines) — no archiving needed (docs 16).

## Verified OK
- 7 themes, default `base16-eighties.dark`, 219 languages (`--list-themes`, `--list-languages`).
- Comment-style table matches help.rs. `cargo build --examples` passes.
