# Glossary

Canonical vocabulary for wedi. Code identifiers, UI strings, commit messages, and every other
document use these terms. Adding a term means adding its entry in the same commit.

**Buffer**:
The text of the open file, held in a rope (`RopeBuffer`) together with its encoding and modified flag.
_Avoid_: Document, content.

**Selection**:
A contiguous range of text marked for copy, cut, delete, indent, or comment toggling. Made with
Shift+movement or while **Selection mode** is on.
_Avoid_: Highlight (reserved for **Syntax highlighting** and **Match highlight**).

**Selection mode**:
A toggled state (Alt+S) in which plain movement keys extend the **Selection**, for terminals that
do not deliver Shift modifiers. Exits on Alt+S, ESC, or any editing command.
_Avoid_: Mark mode, visual mode.

**Search mode**:
The state entered when Ctrl+F finds at least one match. While active, **Smart jump** keys cycle
matches and every match carries a **Match highlight**. Exits on ESC or any buffer-modifying command;
the query is kept for the next Ctrl+F.
_Avoid_: Find mode.

**Smart jump**:
The shared behavior of Ctrl+N/Ctrl+P, F3/Shift+F3, and PageDown/PageUp: next/previous match in
**Search mode**, otherwise page down/up.
_Avoid_: —

**Match highlight**:
The background colour on search matches in **Search mode** — dark yellow for every match, bright
yellow for the current one (`SearchHighlight`).
_Avoid_: Search highlight colour.

**Syntax highlighting**:
Token colouring from syntect with bat's syntax set, toggled by Ctrl+T. Optional Cargo feature
`syntax-highlighting`.
_Avoid_: Colouring, syntax mode.

**Theme**:
One of the syntect built-in colour schemes used by **Syntax highlighting**, chosen with `--theme`.
_Avoid_: Colour scheme, palette.

**Display mode**:
How long lines are shown: **wrap** (continue on the next visual line) or **scroll** (single visual
line, horizontal scrolling). Ctrl+O toggles only the display mode; Ctrl+L toggles line numbers and
sets wrap to match.
_Avoid_: Single-line/multi-line mode (older changelog wording).

**Visual line**:
One screen row. A logical line in wrap **Display mode** may span several visual lines (`LineLayout`).
_Avoid_: Row (outside rendering code), screen line.

**System clipboard**:
The OS clipboard, reached via pbcopy/pbpaste on macOS, wl-copy/xclip on Linux, and the Win32 API
on Windows. Used by Ctrl+C/X/V.
_Avoid_: External clipboard.

**Internal clipboard**:
wedi's in-process clipboard string. Alt+C/X/V use only it; Ctrl+C/X also write to it, and Ctrl+V
falls back to it when the **System clipboard** is unavailable.
_Avoid_: Local clipboard, register.

**Command**:
An editor action (`keymap::Command`) produced from a key event and executed by the editor loop.
_Avoid_: Action (reserved for undo-history `Action`).

**Dialog**:
A single-line prompt over the status bar (search, go-to-line, change encoding, confirm).
_Avoid_: Popup, modal.

**Help panel**:
The full-screen overlay opened by F1, with a Help page and an About page switched by Tab/←/→.
_Avoid_: Help dialog.
