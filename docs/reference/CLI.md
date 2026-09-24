# Command line

Parsed by `Args::parse` in `src/main.rs` (pico-args). Key bindings are in [KEYMAP.md](KEYMAP.md).

```
wedi [OPTIONS] [FILE]
```

`FILE` defaults to `Untitled` when omitted. Unknown arguments print a warning and are ignored.

| Flag | Effect |
|---|---|
| `-h`, `--help` | Print help and key bindings, exit |
| `-v`, `--version` | Print version, exit |
| `--debug` | Debug logging |
| `-e`, `--encoding <ENC>` | Read and save encoding |
| `-f`, `--from-encoding <ENC>` | Read encoding (overrides `-e`) |
| `-t`, `--to-encoding <ENC>` | Save encoding (overrides `-e`) |
| `--theme <NAME>` | Syntax **Theme**; an unknown name exits with an error ¹ |
| `-l`, `--language <NAME>` | Force the syntax, e.g. `-l rust script` ¹ |
| `--list-themes` | List themes, exit ¹ |
| `--list-languages` | List syntaxes, exit ¹ |

¹ Only with the `syntax-highlighting` feature (default).

## Encodings

Read encoding: `--from-encoding` > `--encoding` > auto-detected. Save encoding: `--to-encoding` >
`--encoding` > `--from-encoding` > the file's detected encoding.
A UTF-8 or UTF-16 byte-order mark is kept on save while the save encoding is unchanged. A character the
save encoding can't represent makes the save fail with `Save failed: '<char>' (U+XXXX) can't be saved as <ENC>`;
the file on disk is left untouched.
If the file has bytes that are invalid in the read encoding, they show as `�` and the status bar warns on open;
the first Ctrl+S only repeats the warning, and a second Ctrl+S in a row saves (the invalid bytes are lost).
Use Ctrl+E or `-f` to reopen the file in the right encoding instead.
Saving writes a temporary `.<name>.wedi-<pid>.tmp` next to the file and renames it over the original, so a
crash or full disk never leaves a half-written file. Symlinks are followed and kept, file permissions are
kept, and read-only files are refused. Other hard links to the file keep the old contents.
Labels (`wedi_core::buffer::parse_encoding_label`, shared with Ctrl+E), case-insensitive:

| Label | Encoding |
|---|---|
| `utf-8`, `utf8` | UTF-8 |
| `utf-16le`, `utf16le` / `utf-16be`, `utf16be` | UTF-16 |
| `gbk`, `cp936` | GBK |
| `shift-jis`, `shift_jis`, `sjis` | Shift-JIS |
| `big5`, `cp950` | Big5 |
| `cp1252`, `windows-1252` | Windows-1252 |
| anything else | Any WHATWG label accepted by `encoding_rs::Encoding::for_label` |

## Themes

`--list-themes` prints the set; the default is `base16-eighties.dark`. Built-ins: InspiredGitHub,
Solarized (dark), Solarized (light), base16-eighties.dark, base16-mocha.dark, base16-ocean.dark,
base16-ocean.light.
