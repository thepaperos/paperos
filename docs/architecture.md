# paperOS Architecture

## Overview

paperOS follows a **minimal core + extensions** design. The core provides only
the runtime scaffold and the document data model. Everything else — input handling,
rendering, file I/O, syntax highlighting, command dispatch — is delivered as
an extension.

## Core Philosophy: Everything is a Command

Every user action and editor event is a **command**. Commands are registered in
the `EditorRuntime`'s command registry and dispatched through a single entry
point (`runtime.run_command("command_name args", buffer)`). This applies to:

- **Editor operations**: `left`, `right`, `up`, `down`, `insert <ch>`, `delete`, `deleteline`
- **Movement**: `w` (next word), `b` (prev word), `e` (end of word), `go <n>` (goto line), `lastline` (G)
- **Undo/redo**: `undo`, `redo`
- **File operations**: `open <path>`, `write`, `write <path>`
- **Mode transitions**: `i` (enter insert mode), `:` (enter command mode)
- **System operations**: `quit`, `help`, `version`

### Command Flow

```
User Input (keystroke) → InputExtension.handle_key → runtime.run_command → CommandHandler → BufferActor (via runtime.send)
```

1. **InputExtension** receives a keystroke and maps it to a command string
   (e.g., `h` → `"left"`, `i` → enter insert mode, `f` in insert mode → `"insert f"`)
2. **EditorRuntime.run_command()** parses the command string using `splitn(2)` logic:
   the first whitespace-separated token is the command name, and the remaining
   string (including spaces) is preserved as a single argument. This ensures
   `insert  ` (insert + space character) correctly preserves the space argument.
   The full command array (name + args) is passed to the handler.
3. The **CommandHandler** executes the command — typically by sending a
   `BufferMessage` to the BufferActor via `runtime.send()`
4. For file operations, the **FileExtension**'s command handler performs I/O
   (`std::fs::read_to_string`, `std::fs::write`) and sends `LoadContent` /
   `SetFilePath` messages to the buffer actor
5. **RenderExtension** renders the buffer state on each event loop tick

### Why Commands?

- **Discoverability**: every action has a named command
- **Scriptability**: commands can be invoked programmatically via `runtime.run_command()`
- **Consistency**: same interface for keybindings, CLI args, and config-file macros
- **Testability**: each command handler can be tested independently

## Module Structure

```
src/
├── core/
│   ├── mod.rs            — public re-exports
│   ├── extension.rs      — Extension trait (name, init, shutdown, on_key, on_render)
│   ├── runtime.rs        — EditorRuntime + CommandRegistry + CommandHandler trait
│   └── buffer.rs         — BufferActor + BufferMessage (document data model only)
├── extensions/
│   ├── mod.rs
│   ├── buffer_commands.rs — BufferCommands (maps commands → BufferMessages)
│   ├── buffer_manager.rs  — BufferManager (multi-buffer lifecycle: spawn, switch, track)
│   ├── basic_input.rs     — InputExtension (default: arrow keys, direct typing)
│   ├── vi_input.rs        — ViInputExtension (vi keybindings: h/j/k/l, modes)
│   ├── render.rs          — RenderExtension (terminal UI via crossterm + ratatui)
│   ├── file.rs            — FileExtension (file I/O commands: open, write)
│   ├── visual_mode.rs     — VisualModeExtension (visual mode, yank/cut/paste, search)
│   └── lua_ext.rs        — LuaExtension (Lua scripting engine via mlua)
└── main.rs               — bootstrap: register commands, spawn buffer, run loop
```

## Minimal Core — Strict Boundaries

The `core/` module contains ONLY the document data model and runtime infrastructure:

- **Extension trait**: the interface all extensions implement. Includes `init()`, `shutdown()`, `on_key()`, `on_render()` hooks, and `as_any()`/`as_any_mut()` for type lookup.
- **EditorRuntime**: owns the Runact `Runtime`, manages extension lifecycle,
  and maintains the `CommandRegistry` (just the dispatch mechanism)
- **BufferActor**: the document data model — owns `lines`, `cursor_row`,
  `cursor_col`, `file_path`, and undo/redo history

**Rules (enforced by code review):**
1. No `std::fs` calls in `core/` — file I/O is always an extension
2. No `crossterm`/`ratatui` imports in `core/` — terminal UI is always an extension
3. No direct field access to buffer state from extensions — only via `BufferMessage` sends
4. The BufferActor may contain undo/redo state since it's part of the document model
5. Command handlers live in extensions and communicate with the buffer via messages
6. No Lua or scripting engine imports in `core/` — scripting is always an extension

## Lua Extension System

paperOS configuration and extensions are programmable via **Lua** (using the `mlua` crate).

### Extension Lifecycle

Lua extensions are loaded from `~/.config/paperos/extensions/`:

```
~/.config/paperos/
├── init.lua               # Bootstrap config
├── extensions/            # Lua extensions
│   └── my-extension/
│       └── init.lua
└── config/                # Config modules
    ├── appearance.lua
    └── keybindings.lua
```

Each extension is a Lua file returning a table:

```lua
-- extensions/my-extension/init.lua
return {
  name = "my-extension",
  version = "0.1.0",
  
  init = function(config)
    -- Initialize, return public API
    return { do_thing = function() ... end }
  end,
  
  activate = function(api)
    paper.commands.register("my-command", function(args) ... end)
    paper.keys.map("n", "<leader>m", "my-command")
  end
}
```

### Config Dependency Guards

Config files can declare extension dependencies. If the extension is missing,
the config block is skipped with a warning (not an error):

```lua
-- config/syntax.lua
return {
  requires = "tree-sitter",  -- guard: only applies if tree-sitter is loaded
  apply = function(ts_api)
    ts_api.enable_syntax_highlighting()
  end
}
```

Core enforces this: missing dependencies cause graceful degradation, not crashes.

### Lua API Surface

```
paper
├── config.get/set("key", value)
├── config.watch("key", fn)
├── commands.register("name", fn)
├── commands.run("name", args)
├── keys.map("mode", "key", "command")
├── events.on("event_name", fn)
├── extensions.load("name")
├── extensions.get("name")
├── extensions.list()
└── buffer (current buffer operations)
```

### Development Workflow

```bash
# Create new extension
mkdir -p ~/.config/paperos/extensions/my-extension

# Development mode (auto-reload)
paperos --dev --extension my-extension

# Test extension
paperos --test extension=my-extension
```

## Command Registration

Commands are registered via `runtime.register_command("name", handler)`.
Each handler implements `CommandHandler`:

```rust
trait CommandHandler: Any + Send + Sync {
    fn handle(&mut self, args: &[String], runtime: &Runtime, buffer: Option<ActorId>) -> Result<(), Box<dyn Error>>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

Multiple command names can share a handler via `SharedBufferCommands` or
`SharedFileCommands` wrappers that use `Arc<Mutex<T>>` internally.

## Built-in Commands

| Command | Handler | Description |
|--------|---------|-------------|
| `h` / `left` | BufferCommands | Move cursor left |
| `j` / `down` | BufferCommands | Move cursor down |
| `k` / `up` | BufferCommands | Move cursor up |
| `l` / `right` | BufferCommands | Move cursor right |
| `w` | BufferCommands | Move to next word |
| `b` | BufferCommands | Move to previous word |
| `e` | BufferCommands | Move to end of word |
| `go <n>` / `line <n>` | BufferCommands | Go to line N |
| `lastline` / `G` | BufferCommands | Go to last line |
| `0` | BufferCommands | Move to start of line |
| `$` | BufferCommands | Move to end of line |
| `i` | InputExtension | Enter insert mode |
| `insert <ch>` | BufferCommands | Insert character |
| `newline` | BufferCommands | Insert new line |
| `delete` / `x` | BufferCommands | Delete character |
| `deleteline` / `dd` | BufferCommands | Delete line |
| `undo` / `u` | BufferCommands | Undo |
| `redo` / `U` | BufferCommands | Redo |
| `open <path>` / `e <path>` | FileExtension | Open file |
| `write` / `w` | FileExtension | Save file |
|| `quit` / `q` | QuitCommand | Exit editor |
|| `bnext` | BufferManager | Switch to next buffer |
|| `bprev` / `bprevious` | BufferManager | Switch to previous buffer |
|| `badd <file>` / `b <file>` | BufferManager | Open file as new buffer |
|| `blist` / `buffers` | BufferManager | List all open buffers |

## Extension Lifecycle

1. `init(runtime)` — extension initializes
2. `on_key(ch)` — extension receives a key event (default: no-op)
3. `on_render(runtime)` — extension renders (default: no-op)
4. `shutdown(runtime)` — extension cleans up

Extensions are looked up by type via `runtime.extension::<T>()`.
