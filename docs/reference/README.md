# Reference

Current behavior only. Anything historical — a superseded design, a shipped plan, a resolved
investigation — lives in `../spec/`, `../plan/`, `../debug/`, or `../audit/`, not here.

- **[glossary.md](glossary.md)** — canonical vocabulary; read first.
- **[ARCHITECTURE.md](ARCHITECTURE.md)** — crates, modules, features, and the main data flows.
- **[KEYMAP.md](KEYMAP.md)** — every key binding and the mode it depends on.
- **[CLI.md](CLI.md)** — command-line flags, encodings, themes.

Machine-checked: `KEYMAP.md` by `help::tests` in `src/help.rs` (help panel ↔ `KEYMAP.md` ↔
`handle_key_event`).

See also [`../adr/`](../adr/README.md) for decision records.
