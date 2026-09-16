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

## TDD Method

Every change follows red → green → refactor. No exceptions.

### 1. Write acceptance tests first

Place behavioural tests in `tests/acceptance/<feature>_tests.rs`.
Add a `[[test]]` target in `Cargo.toml` so cargo discovers subdirectory files.

One test per acceptance criterion. Each test hits a real public entry point
(`ensure_cursor_visible`, `clamp_scroll`, `BufferMessage::MoveDown`, etc.)
— no mocked internals, no assertion-free smoke checks.

Run them. They must fail (compile error or assertion failure). That is the
signal that the feature does not yet exist.

### 2. Write failing unit tests

Place internal logic tests in the same file as the unit under test, inside
`#[cfg(test)] mod tests { ... }`. These cover edge cases, clamping, and
private helper behaviour that acceptance tests cannot reach directly.

Run them. They must also fail.

### 3. Implement

Make both test sets pass with the smallest change that satisfies them.
Follow existing codebase patterns (read sibling files first).
Do not add unrelated features, refactors, or dependencies.

### 4. Verify

```bash
cargo test          # acceptance + unit — all green
cargo check         # no new warnings
cargo fmt           # formatted
```

### Rules

- **Fix the code, never the test.** If an acceptance test fails, the
  implementation is wrong — rewrite the test only if the criterion itself
  was misunderstood (ask the user first).
- **No test, no feature.** An acceptance criterion without a matching
  failing test does not count as done.
- **Agent self-reporting is not evidence.** Only a green `cargo test`
  output counts. Print it.
- **Minimal change.** Each commit should be the smallest diff that
  makes the failing tests pass. Do not sneak in refactors.
