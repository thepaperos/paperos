# paperOS Roadmap

## Overview

This document outlines the development phases for paperOS, the programmable editor built on the Runact actor runtime.

paperOS follows a **minimal core + extensions** architecture:

- **Core** (`src/core/`) — The minimal document model: `EditorRuntime` (spawn/shutdown, command registry, extension management), `Extension` trait (lifecycle + hooks), and `BufferActor` (document data model with undo/redo history). No file I/O, no terminal rendering, no input parsing, no scripting.
- **Built-in extensions** (`src/extensions/`) — `InputExtension` (key → command translation), `RenderExtension` (terminal UI, event loop), `FileExtension` (file I/O: open/save), `BufferCommands` (maps command names to BufferMessage sends), `LuaExtension` (Lua scripting engine), `BufferManager` (multi-buffer lifecycle: spawn, switch, track), `VisualModeExtension` (visual mode, yank, paste, search). Every editor action is a command.
- **Architecture doc**: see `docs/architecture.md`
- **Extension management**: see `docs/extensions.md`

---

## Current Status

| Component | Status | Tests |
|-----------|--------|-------|
| **Runact** (runtime) | v1.0.0 released | 48/48 passing |
| **paperOS** (core) | ✅ Minimal core: EditorRuntime, Extension trait, BufferActor | Compiles |
| **paperOS** (built-in extensions) | ✅ InputExtension, RenderExtension, FileExtension, BufferCommands, LuaExtension | Compiles |

---

## Phase 1 — Minimal Core ✅

### Deliverables

- [x] `paperos/` — Separate project (package name `paperos`)
- [x] `src/core/` — `Extension` trait, `EditorRuntime`, `BufferActor`
- [x] `src/extensions/` — `InputExtension`, `RenderExtension`, `FileExtension`, `BufferCommands`
- [x] `src/main.rs` — bootstraps runtime, spawns buffer, registers commands, runs event loop

### Core Module

```
src/core/
├── mod.rs        — re-exports
├── extension.rs  — Extension trait (Any + Send + Sync, init/shutdown/on_key/on_render)
├── runtime.rs    — EditorRuntime + CommandRegistry + CommandHandler trait
└── buffer.rs     — BufferActor + BufferMessage (document data model only)
```

### Built-in Extensions

```
src/extensions/
├── mod.rs
├── buffer_commands.rs — BufferCommands (maps commands → BufferMessages)
├── file.rs      — FileExtension (file I/O: open, write)
├── input.rs     — InputExtension (vi modes, key → command translation)
├── render.rs    — RenderExtension (crossterm + ratatui, event loop, status bar)
├── lua_ext.rs   — LuaExtension (Lua scripting engine via mlua)
├── buffer_manager.rs — BufferManager (multi-buffer lifecycle: spawn, switch, track)
└── visual_mode.rs — VisualModeExtension (visual mode, yank, paste, search)
```

---

## Phase 2 — Extension Architecture ✅

### Deliverables

- [x] Extension trait with `init()`, `shutdown()`, `on_key()`, `on_render()` hooks
- [x] Extension lookup by type: `runtime.extension::<T>()`
- [x] Command system — `CommandHandler` trait, `CommandRegistry`, `run_command()` dispatch
- [x] Event hooks (`on_key`, `on_render`, `shutdown`) implemented on all extensions
- [x] File I/O as commands (`open`/`e`, `write`/`w`)
- [x] Lua scripting engine (mlua) for config and extensions
- [x] Config dependency guards (requires/available pattern)
- [x] Extension lifecycle (init, activate, commands, keybindings)
- [x] Command dispatch fix (args[0] is command name, args[1+] are arguments)
- [x] Command parsing fix (space characters preserved in insert commands)
- [x] Quit command (`:q` / `:quit`)

---

## Roadmap to paperOS v1

### v0.5 — Working Editor

| Task | Priority | Status |
|------|----------|--------|
| Fix cursor rendering | high | ✅ |
| Buffer: line wrapping, scroll | high | ✅ |
| Input: complete vim keybindings (w, b, e, gg, G) | medium | ✅ |
| File: open/save via `:e` and `:w` commands | high | ✅ |
| Status bar: file name, line count, modified indicator | medium | ✅ |
| CLI: `paperos --help`, `paperos <file>` | high | ✅ |
| Buffer: open file from CLI argument | high | ✅ |

### v0.6 — Usability

| Task | Priority | Status |
|------|----------|--------|
| Visual mode (selection) | medium | ✅ |
|| Search (`/`) | medium | ✅ |
|| Clipboard (yank/paste/delete) | medium | ✅ |
|| Line numbers | medium | ✅ |
|| Multiple buffers/windows | medium | ✅ |

### v0.7 — Extensibility

| Task | Priority | Status |
|------|----------|--------|
|| Lua extension API docs | medium | ⬜ |
|| Built-in extensions: syntax highlighting | medium | ⬜ |
|| Example: custom status bar extension | low | ⬜ |

### v1.0 — Release

| Task | Priority | Status |
|------|----------|--------|
| `cargo publish` to crates.io | high | ✅ |
| README with screenshots | high | ✅ |
| `paperos --help`, `paperos <file>` CLI | high | ✅ |

---

## Success Metrics

### Editor
- Working editor
- Responsive UI
- Real-world functionality
- Extensions work

---

## Long-Term Evolution

```
Runtime Core
    ↓
Reliability (Supervision)
    ↓
Compute Pool
    ↓
Timers and I/O
    ↓
Resources and Capabilities
    ↓
Editor Runtime (paperOS)
    ↓
Programmable Environment
```

The runtime must earn every layer of complexity.
