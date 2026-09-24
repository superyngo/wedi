# Reference

Current behavior only. Anything historical — a superseded design, a shipped plan, a resolved
investigation — lives in `../spec/`, `../plan/`, `../debug/`, or `../audit/`, not here.

- **[glossary.md](glossary.md)** — canonical vocabulary; read first.
- **[ARCHITECTURE.md](ARCHITECTURE.md)** — crates, modules, features, and the main data flows.
- **[KEYMAP.md](KEYMAP.md)** — every key binding and the mode it depends on.
- **[CLI.md](CLI.md)** — command-line flags, encodings, themes.

Machine-checked: none yet. `KEYMAP.md` is written against `keymap::bindings::handle_key_event`
and `src/help.rs` `get_help_sections`; re-check both when a binding changes.

See also [`../adr/`](../adr/README.md) for decision records.
