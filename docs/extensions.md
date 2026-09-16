# Extension Management System

## Overview

paperOS supports user extensions installed in `~/.config/paperos/extensions/`.
This document describes the discovery, metadata format, dependency resolution,
and load ordering rules used by the extension manager.

## Extension Layout

Extensions reside in `~/.config/paperos/extensions/<name>/`:

```
extensions/
├── my-greeter/        — extension directory
│   ├── manifest.json  — metadata (name, version, dependencies)
│   └── init.lua       — extension code (returns a Lua table)
├── tree-sitter/
│   ├── manifest.json
│   └── init.lua
```

## Manifest Format

Each extension declares a `manifest.json`:

```json
{
  "name": "syntax-highlight",
  "version": "1.0.0",
  "description": "Tree-sitter based syntax highlighting",
  "author": "User Name",
  "depends": ["buffer-api>=0.2", "config>=1.0.0"],
  "provides": ["syntax-highlighting"]
}
```

Fields:

| Field       | Type     | Description                                      |
|-------------|----------|--------------------------------------------------|
| `name`      | string   | Unique extension identifier                      |
| `version`   | string   | SemVer-style version string                      |
| `description`| string  | Human-readable summary                           |
| `author`    | string   | Extension author                                 |
| `depends`   | string[] | List of required extensions with version constraints |
| `provides`  | string[] | List of capabilities other extensions can depend on |

## Version Constraints

Dependencies support these constraints:

| Syntax             | Meaning                                    |
|--------------------|--------------------------------------------|
| `"name"`           | Any version                                |
| `"name>=X.Y.Z"`    | Minimum version (inclusive)                |
| `"name==X.Y.Z"`    | Exact version                              |
| `"name>=X.Y,<X+1.0"`| Range — at least X.Y, less than X+1.0      |
| `"name>1.0,<=2.0"` | Greater than 1.0 and at most 2.0           |

Example: `"buffer-api>=0.2,<1.0"` — any version from 0.2 up to (but not
including) 1.0.

## Dependency Resolution

### Algorithm

1. **Parse all available extensions** from `extensions/` directory.
2. **Build dependency graph** — each node has its version constraints.
3. **Select compatible versions** — find the highest version satisfying all
   constraints for each dependency.
4. **Topological sort** — load dependencies before dependents.
5. **Handle conflicts** — if no version satisfies all constraints, use the
   highest requested version and warn.

### Conflict Example

Extensions requesting `buffer-api`:

- `syntax-highlight` requires `buffer-api>=0.2`
- `debugger` requires `buffer-api>=0.3`
- `legacy-plugin` requires `buffer-api<0.3`

**Resolution:**
- Load `buffer-api` version `0.3` (highest satisfying first two)
- Load `syntax-highlight` and `debugger`
- Skip `legacy-plugin` with warning:
  `"legacy-plugin skipped: requires buffer-api<0.3 but 0.3 installed"`

### Circular Dependencies

Circular dependencies (A → B → A) produce an error:

```
Error: Circular dependency detected: A -> B -> A
```

## Load Order

Extensions load in **topological order**: dependencies first, then dependents.
This ensures that when an extension initializes, all its dependencies are
already running.

## Isolation

- Each extension runs in its **own Lua state** (isolated global variables).
- Extensions communicate only through **shared Actor messages** (not direct
  function calls).
- Each extension can register **commands** and **event handlers**.

## Side-by-Side Versions (Future)

If the ecosystem grows to need conflicting versions simultaneously, extensions
can be namespaced:

```
extensions/
├── buffer-api-v0.2/
├── buffer-api-v0.3/
└── syntax-highlight/   # depends on exact "buffer-api-v0.2"
```

Each version runs in its own Lua state with isolated `BufferActor` instances.
This is a future capability — currently only single-version resolution is
supported.

## CLI Interface

Commands are available via `paperos ext` or `:ext` in command mode:

```bash
paperos ext list              # Show all extensions (enabled/disabled)
paperos ext list --json       # JSON output for tooling
paperos ext enable <name>     # Enable an extension
paperos ext disable <name>    # Disable an extension
paperos ext install <url>     # Download and install from a git URL
paperos ext info <name>       # Show manifest and load status
paperos ext reload            # Reload extensions without restart
```

## Runtime API

Extensions interact via the `paper` global:

```lua
paper.extensions.get("name")   — get a handle to another extension
paper.extensions.has("name")   — check if extension is loaded
paper.commands.register("cmd", fn)  — register a command
paper.buffer.get_content()     — get current buffer content
paper.buffer.send_message(msg) — send a message to the buffer actor
paper.status.set(text)         — update status line
```
