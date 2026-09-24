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
| Widget documentation and usage examples | Only `examples/basic_usage.rs` exists; no external embedder yet | First external user of `wedi-widget` | 2026-09-24 |
| More syntax themes | 7 syntect built-ins cover light and dark | User request | 2026-09-24 |
| Large-file performance | No measured regression | A reported slowdown with a reproducible file | 2026-09-24 |
| LSP support, multi-cursor, split view, plugins | Long-term ideas; conflict with the minimalist scope | A spec in `docs/spec/` | 2026-09-24 |
| CHANGELOG has no entries for tagged v0.1.5–v0.1.18, v0.5.1, v0.8.1 | Content can't be reconstructed without re-reading each diff | Someone needs those releases' notes | 2026-09-24 |

## Done

| ID | Finding | Closed by |
|---|---|---|
