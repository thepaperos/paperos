# AGENTS.md — paperOS Development Guidelines

## Core Boundaries (Strict)

The `src/core/` module contains ONLY the document model and runtime infrastructure.
Violations require explicit approval from a senior maintainer.

### core/ may contain:
- `EditorRuntime` — actor spawning, command registry, extension lifecycle
- `Extension` trait — the interface all extensions implement
- `BufferActor` — document data model (lines, cursor, file_path, undo/redo)

### core/ must NOT contain:
1. `std::fs` calls — file I/O is always an extension
2. `crossterm` / `ratatui` — terminal UI is always an extension
3. `mlua` or any scripting engine — scripting is always an extension
4. Direct buffer field access from outside `BufferActor::handle`
5. Undo/redo logic — it is part of the document model (co-located with lines)
6. Visual mode / selection state — that is an extension concern
7. Search / clipboard state — belongs in extensions, not the document model

## Design Principles

1. Everything is a command — every action goes through `runtime.run_command()`
2. Minimal core — if it can be an extension, it must be an extension
3. Extensions communicate with BufferActor via `BufferMessage` sends only
4. Command dispatch uses `&Runtime` + `Option<ActorId>` (not `&EditorRuntime`)
5. Buffer lifecycle (spawn, switch, track) is an extension concern, not core

## Command System

- Commands registered via `runtime.register_command("name", handler)`
- `run_command("cmd args")` parses with `splitn(2)` — first token is command name,
  rest is preserved as a single argument (handles space chars correctly)
- Handlers implement `CommandHandler` trait: `handle(&mut self, args: &[String], runtime: &Runtime, buffer: Option<ActorId>)`

## Development

```bash
# Build
cargo check
cargo build --release

# Format
cargo fmt

# Run
cargo run -- file.txt
```

## Project References
- Architecture: docs/architecture.md
- Roadmap: docs/roadmap.md
