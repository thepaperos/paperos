# PaperOS Roadmap

## Overview

PaperOS is a programmable computing environment built on the Runact actor runtime. The editor is only the first major extension — terminal, Git, LSP, search, AI, and user extensions follow.

PaperOS follows a **minimal core + extensions** architecture:

- **Core** (`src/core/`) — The runtime scaffold: `EditorRuntime` (actor spawning, command registry, extension lifecycle), `Extension` trait (lifecycle + hooks), and `BufferActor` (document data model with undo/redo history). No file I/O, no terminal rendering, no input parsing, no scripting.
- **Built-in extensions** (`src/extensions/`) — `InputExtension` (key → command translation), `ViInputExtension` (vi-style keybindings), `RenderExtension` (terminal UI, event loop), `FileExtension` (file I/O: open/save), `BufferCommands` (maps command names to BufferMessage sends), `LuaExtension` (Lua scripting engine), `BufferManager` (multi-buffer lifecycle), `VisualModeExtension` (visual mode, yank, paste, search).
- **Architecture doc**: see `docs/architecture.md`
- **Extension management**: see `docs/extensions.md`

---

## Current Status

| Component | Status | Tests |
|-----------|--------|-------|
| **Runact** (runtime) | v1.0.0 released | 48/48 passing |
| **paperOS** (core) | Minimal core: EditorRuntime, Extension trait, BufferActor | Compiles |
| **paperOS** (extensions) | InputExtension, ViInputExtension, RenderExtension, FileExtension, LuaExtension, BufferManager, VisualModeExtension | Compiles |

---

## Phase 1 — Architecture (partially complete)

Implement/document:

- [x] `paperos/` — Separate project (package name `paperos`)
- [x] `src/core/` — `Extension` trait, `EditorRuntime`, `BufferActor`
- [x] `src/extensions/` — All built-in extensions
- [x] `src/main.rs` — bootstraps runtime, spawns buffer, registers commands, runs event loop
- [x] Command system — `CommandHandler` trait, `CommandRegistry`, `run_command()` dispatch
- [x] Extension lifecycle (`init`, `shutdown`, `on_key`, `on_render`)
- [x] Extension lookup by type: `runtime.extension::<T>()`
- [x] Event hooks implemented on all extensions
- [x] File I/O as commands (`open`/`e`, `write`/`w`)
- [x] Lua scripting engine (mlua) for config and extensions
- [ ] Event bus (cross-extension state change notification)
- [ ] UI protocol definition
- [ ] Capability management (extension permissions)

---

## Phase 2 — Editor Vertical Slice

Implement:

- [ ] BufferActor with full CRUD through commands
- [ ] Open / Edit / Undo / Redo / Save entirely through commands → actors → events
- [ ] Web UI connected through UI protocol
- [ ] Basic keybindings mapped to commands

This proves the architecture works end-to-end. No shortcut implementation should bypass the command/event system.

---

## Phase 3 — Extension System

Implement:

- [ ] Extension manifest format
- [ ] Extension discovery and loading from `~/.config/paperos/extensions/`
- [ ] Lua API (`paper.*` namespace)
- [ ] Extension lifecycle management (load / unload / reload)
- [ ] Extension configuration
- [ ] Command/event API for extensions
- [ ] Extension isolation (own Lua state)

---

## Phase 4 — Development Environment

Add:

- [ ] Filesystem service/extension (open, read, write, watch, search directory)
- [ ] Terminal extension (TerminalActor + subprocess lifecycle via Runact)
- [ ] Search extension (SearchActor + compute pool for large searches)
- [ ] Git extension (GitActor + source control view + git commands)
- [ ] LSP extension (diagnostics, completion, hover, definition, references, rename)

---

## Phase 5 — Automation

Add:

- [ ] Event-driven workflows (`on("file.saved", handler)`)
- [ ] Task scheduling
- [ ] Timers
- [ ] Lua automation API
- [ ] Auto-format on save, auto-test on commit

---

## Phase 6 — AI

Add:

- [ ] AI provider abstraction
- [ ] AI actors (AIAgentActor with conversation state, context, task state, tool state)
- [ ] Tool system (read buffer, inspect diagnostics, run tests, modify buffer)
- [ ] Context system
- [ ] Agent workflows
- [ ] Permission system for AI operations

---

## Phase 7 — Extension Ecosystem

Add:

- [ ] Package manager (discover, install, update, remove)
- [ ] Extension registry
- [ ] Versioning and compatibility checking
- [ ] Dependency resolution
- [ ] Updates and upgrade management
- [ ] Sandboxing (WASM extensions)

---

## Development Guidelines

See `AGENTS.md` for development rules. All changes follow TDD: red → green → refactor.

```bash
cargo check           # type check
cargo build --release # optimized build
cargo test            # run all tests
cargo fmt             # format code
```

---

## Success Metrics

- Working editor with responsive UI
- Extensions compose through commands and events
- Extension failure does not crash the environment
- Multiple UIs supported through stable protocol
- Users can add, remove, and replace extensions
- Real-world editing workflows function correctly

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
