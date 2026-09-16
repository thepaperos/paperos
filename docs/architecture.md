# PaperOS — Architectural Document

**Project:** PaperOS
**Language:** Rust
**Runtime:** Runact
**UI:** Web technology / WebView
**Extension Languages:** Lua initially, with WASM/Rust as future options
**Type:** Extensible programmable computing environment

---

# 1. Vision

PaperOS should not be designed as merely another text editor.

The long-term goal is:

> A programmable, extensible computing environment where the editor, terminal, filesystem, UI, automation, AI, Git, LSP, search, and other capabilities are composable extensions running on a small stable core.

The user should be able to modify the environment itself.

Instead of:

```text
Application
 ├── Editor
 ├── Terminal
 ├── File Manager
 ├── Git
 └── AI
```

being permanently hardcoded, PaperOS should provide:

```text
PaperOS Core
     │
     ├── Editor Extension
     ├── Terminal Extension
     ├── File Explorer Extension
     ├── Git Extension
     ├── LSP Extension
     ├── Search Extension
     ├── AI Extension
     ├── Database Extension
     └── User Extensions
```

---

# 2. Core Philosophy

PaperOS follows six major principles.

## Principle 1 — Small Core

The core should contain only functionality required to run the environment.

## Principle 2 — Everything Is a Command

User actions should be represented as commands.

## Principle 3 — Everything Important Produces Events

State changes should be observable.

## Principle 4 — Extensions Are First-Class

Features should be implemented as extensions rather than hardcoded application logic.

## Principle 5 — UI Is a Client

The UI should display and interact with system state rather than own the authoritative application state.

## Principle 6 — Components, Not Actors by Default

Everything is a logical component. Actors are used where isolation, concurrency, or lifecycle justification exist — not everywhere. Prefer the simplest structure that works; graduate to an actor when supervision, fault isolation, or concurrent message handling is genuinely needed.

---

# 3. High-Level Architecture

```text
                         PaperOS
                            │
                ┌───────────▼───────────┐
                │      Core Runtime     │
                │       Runact          │
                └───────────┬───────────┘
                            │
       ┌────────────────────┼─────────────────────┐
       │                    │                     │
┌──────▼──────┐      ┌──────▼──────┐      ┌──────▼──────┐
│ Extensions  │      │ Command Bus │      │ Event Bus   │
└──────┬──────┘      └──────┬──────┘      └──────┬──────┘
       │                    │                     │
       └────────────────────┼─────────────────────┘
                            │
                     ┌──────▼──────┐
                     │ Application │
                     │   State     │
                     └──────┬──────┘
                            │
                     ┌──────▼──────┐
                     │ UI Bridge   │
                     └──────┬──────┘
                            │
                     ┌──────▼──────┐
                     │ Web UI      │
                     │ HTML/CSS/TS │
                     └─────────────┘
```

---

# 4. Core

The PaperOS core should remain intentionally small.

Core responsibilities:

```text
Runtime
Actor management
Extension management
Command routing
Event routing
Configuration
Capability management
Resource management
UI bridge
Lifecycle
Persistence interfaces
```

The core should NOT contain:

```text
Editor implementation
Git implementation
Terminal implementation
AI implementation
File explorer implementation
LSP implementation
Database client
```

Those should be extensions.

---

# 5. Runact Integration

PaperOS should use Runact as its concurrency foundation.

Major PaperOS components can be actors.

Example:

```text
PaperOS Supervisor
│
├── ExtensionManager
├── CommandBus
├── EventBus
├── UI Bridge
├── Workspace
│
├── BufferManager
│   ├── BufferActor
│   ├── BufferActor
│   └── BufferActor
│
├── FileService
├── SearchService
├── GitService
├── TerminalService
└── AIService
```

This allows components to be isolated and supervised.

---

# 6. Buffer Architecture

The current `BufferActor` concept should remain important.

A buffer owns:

```text
Buffer content
Cursor
Selection
Undo history
Redo history
Dirty state
File association
Buffer metadata
```

Example:

```text
BufferActor
 ├── State
 ├── Mailbox
 └── Commands
```

Commands:

```text
Insert
Delete
Replace
Undo
Redo
MoveCursor
Select
Save
Reload
```

---

# 7. Everything Is a Command

User actions should become commands.

Example:

```text
Ctrl+S
   ↓
SaveBufferCommand
   ↓
Command Bus
   ↓
BufferActor
   ↓
Save
   ↓
BufferSavedEvent
   ↓
UI
```

The key idea:

> Keyboard input should not directly manipulate application state.

It should generate a command.

---

# 8. Command System

A command should have:

```text
Command ID
Name
Arguments
Source
Permissions
Execution target
```

Example:

```text
editor.buffer.save
```

Another:

```text
terminal.execute
```

Another:

```text
git.commit
```

Another:

```text
ai.explain_selection
```

Commands become the universal API of PaperOS.

---

# 9. Command Sources

Commands can originate from:

```text
Keyboard
Mouse
Command palette
Lua
AI
Extension
Web UI
Terminal
Automation
IPC
```

All of these eventually reach the same command system.

This prevents separate implementations of the same operation.

---

# 10. Events

Commands can produce events.

Example:

```text
Command
   ↓
Actor
   ↓
State change
   ↓
Event
```

Example:

```text
BufferChanged
BufferSaved
FileOpened
CursorMoved
SelectionChanged
ExtensionLoaded
ExtensionUnloaded
TerminalExited
GitStatusChanged
SearchCompleted
```

Events allow independent extensions to react without tightly coupling themselves.

---

# 11. Event-Driven Architecture

Example:

```text
BufferActor
    │
    │ BufferChanged
    ▼
Event Bus
    │
    ├── UI
    ├── LSP
    ├── AI
    ├── Status Bar
    └── Git
```

The buffer does not need to know that these consumers exist.

---

# 12. Extension System

Extensions are fundamental to PaperOS.

An extension may provide:

```text
Commands
Events
Actors
Views
Panels
Menus
Keybindings
Configuration
Services
Resources
Tasks
Capabilities
```

Example:

```text
Git Extension
 ├── git.status
 ├── git.commit
 ├── git.push
 ├── GitActor
 ├── SourceControlView
 └── Git configuration
```

---

# 13. Extension Lifecycle

Every extension should have a lifecycle.

```text
Discovered
   ↓
Loaded
   ↓
Initialized
   ↓
Running
   ↓
Stopping
   ↓
Stopped
```

An extension may also be:

```text
Reloaded
Disabled
Upgraded
Failed
```

The extension manager controls this lifecycle.

---

# 14. Extension Isolation

An extension should not automatically have unrestricted access.

Instead:

```text
Extension
    │
    ▼
Capabilities
```

Examples:

```text
filesystem.read
filesystem.write
network
process.spawn
ui.create
workspace.read
workspace.write
clipboard
```

This enables controlled extension permissions.

---

# 15. Lua

Lua should initially be the high-level scripting language.

Lua is appropriate for:

* user customization
* keybindings
* commands
* automation
* configuration
* small extensions
* editor workflows

Example conceptual API:

```lua
commands.register("hello", function()
    ui.notify("Hello")
end)
```

Lua should interact with PaperOS through stable APIs rather than directly manipulating internal Rust structures.

---

# 16. Future WASM Extensions

WASM can later provide stronger isolation.

Potential extension model:

```text
Rust Native Extension
       │
       ├── maximum capability
       └── high performance

WASM Extension
       │
       ├── sandboxed
       └── portable

Lua Extension
       │
       ├── lightweight
       └── highly customizable
```

Do not implement all three simultaneously.

Start with Lua.

---

# 17. UI Architecture

The UI should use web technology.

Recommended conceptual architecture:

```text
Rust / Runact
      │
      │ UI protocol
      ▼
WebView
      │
      ├── HTML
      ├── CSS
      └── TypeScript
```

The web UI should not become the application backend.

Rust remains authoritative.

---

# 18. UI as a Projection

Think of the UI as a projection of PaperOS state.

```text
PaperOS State
      │
      ▼
Events
      │
      ▼
UI State
      │
      ▼
DOM
```

User interaction flows in the opposite direction:

```text
DOM interaction
      │
      ▼
Command
      │
      ▼
PaperOS
```

---

# 19. UI Protocol

The UI bridge should eventually define a stable protocol.

Conceptually:

```text
UI → Core

command
input
focus
resize
view action

Core → UI

event
state update
notification
view update
layout update
```

The protocol should be independent of the actual WebView implementation.

This makes future interfaces possible.

---

# 20. Multiple UIs

The same PaperOS core should eventually support:

```text
Web UI
Desktop WebView
Terminal UI
Remote Web UI
Potential native UI
```

All should communicate with the same core through the UI protocol.

---

# 21. Workspace

The workspace represents the user's working environment.

It may contain:

```text
Projects
Files
Buffers
Extensions
Tasks
Terminals
Git repositories
Search indexes
Configuration
```

The workspace should be independent from the UI.

---

# 22. File System

Filesystem functionality should be implemented through a service/extension.

Possible responsibilities:

```text
Open file
Read file
Write file
Watch file
Search directory
Rename
Delete
Create
```

The editor should not directly own all filesystem logic.

---

# 23. Terminal

Terminal should be an extension.

Architecture:

```text
Terminal UI
     │
     ▼
Terminal Command
     │
     ▼
TerminalActor
     │
     ▼
OS Process
```

The actor owns the subprocess lifecycle.

Runact supervision can detect process failures.

---

# 24. Git

Git should be an extension.

Potential components:

```text
GitActor
GitService
SourceControlView
Git commands
Git events
```

Example:

```text
git.status
git.diff
git.stage
git.commit
git.push
git.pull
```

The UI should not execute Git directly.

---

# 25. LSP

Language Server Protocol integration should be an extension.

Architecture:

```text
Editor
  │
  ▼
LSP Extension
  │
  ▼
Language Server
```

The LSP extension can observe:

```text
BufferChanged
FileOpened
FileSaved
CursorMoved
```

and issue:

```text
diagnostics
completion
hover
definition
references
rename
format
```

---

# 26. Search

Search should be a service/extension.

Architecture:

```text
Search Command
      │
      ▼
SearchActor
      │
      ▼
Compute Pool
      │
      ▼
SearchResultEvent
      │
      ▼
UI
```

Large searches should not block the normal actor scheduler.

---

# 27. AI Integration

AI should be an extension rather than a core feature.

Potential components:

```text
AI Manager
AI Provider
AI Agent
AI Context
AI Tools
AI Commands
```

AI could interact with PaperOS using the same command/event architecture.

Example:

```text
User
 │
 └── "Fix this error"

AI Agent
 │
 ├── read buffer
 ├── inspect diagnostics
 ├── inspect files
 ├── run tests
 ├── modify buffer
 └── explain result
```

AI should operate through capabilities and commands.

It should not receive unrestricted internal access.

---

# 28. AI as an Actor

A long-running AI agent can be represented as an actor.

```text
AIAgentActor
 ├── conversation state
 ├── context
 ├── task state
 ├── tool state
 └── cancellation
```

It can receive:

```text
StartTask
UserMessage
ToolResult
Cancel
Pause
Resume
```

This fits naturally into Runact.

---

# 29. Configuration

Configuration should itself be extensible.

Possible sources:

```text
Default configuration
Workspace configuration
User configuration
Extension configuration
Session configuration
```

Configuration should be observable.

Example:

```text
ConfigChanged
```

Extensions can react without restarting the application.

---

# 30. Keybindings

Keybindings should not be hardcoded.

Example:

```text
Ctrl+S → editor.buffer.save
Ctrl+P → command.palette
Ctrl+Shift+P → command.palette
```

Users should be able to replace them.

A keybinding is simply:

```text
Input
  ↓
Command
```

---

# 31. Modes

The existing Vi-style mode architecture can remain an extension-level feature.

For example:

```text
Normal
Insert
Visual
Command
```

Modes should map input to commands.

This means another extension could provide:

```text
Emacs-style bindings
Modal bindings
Custom bindings
```

without changing the core.

---

# 32. Views

Views should be extension-provided.

Examples:

```text
EditorView
FileTreeView
TerminalView
GitView
SearchView
ProblemsView
AIView
DatabaseView
```

A view should describe:

```text
View ID
State
Commands
Events
Layout
Rendering
```

The core should provide the infrastructure, not every view implementation.

---

# 33. Layout

The window/layout system should also become extensible.

Conceptually:

```text
Workspace
│
├── Sidebar
├── Main Area
│   ├── Editor
│   └── Terminal
└── Bottom Panel
    └── Problems
```

Users should eventually be able to customize layouts programmatically.

---

# 34. Everything Composes

A major design goal is composition.

For example:

```text
Git
  ↓
Buffer
  ↓
LSP
  ↓
AI
  ↓
Terminal
  ↓
Tests
```

The components communicate through commands and events.

No component should need to know the implementation details of unrelated components.

---

# 35. Extension Example

A hypothetical Markdown extension:

```text
Markdown Extension
│
├── markdown.preview
├── markdown.format
├── MarkdownActor
├── MarkdownView
├── keybindings
└── configuration
```

Another extension could replace the Markdown renderer without modifying the core.

---

# 36. Package/Extension Manager

Eventually PaperOS should have an extension manager.

Responsibilities:

```text
Discover
Install
Update
Remove
Enable
Disable
Load
Unload
Reload
Resolve dependencies
Check compatibility
Manage permissions
```

However, this should come after the extension API is stable.

---

# 37. Extension Manifest

An extension should describe itself.

Example conceptual manifest:

```toml
id = "paperos.git"
name = "Git"
version = "0.1.0"

[capabilities]
filesystem = true
process = true

[commands]
status = "git.status"
commit = "git.commit"
```

The actual format can be decided later.

---

# 38. Versioning

The most important API to stabilize is the extension API.

Potential versioning:

```text
Extension API 1
Extension API 2
Extension API 3
```

Core internals can change as long as the extension contract remains compatible.

---

# 39. Persistence

PaperOS should distinguish:

```text
Application state
Workspace state
User configuration
Extension state
Session state
```

For example:

```text
User configuration:
~/.config/paperos/

Workspace:
project/.paperos/

Session:
runtime-managed
```

Do not tightly couple persistence to the UI.

---

# 40. Error Handling

Errors should be represented explicitly.

Examples:

```text
CommandError
ExtensionError
PermissionError
FileError
ProcessError
UiError
TimeoutError
```

An extension failure should normally affect that extension rather than the entire application.

---

# 41. Crash Isolation

Suppose:

```text
Git Extension crashes
```

PaperOS should ideally remain operational:

```text
Editor       Running
Terminal     Running
File Manager Running
AI           Running

Git          Restarting
```

This is one of the major reasons to use Runact.

---

# 42. Development Architecture

Initial source structure can remain relatively simple.

Conceptually:

```text
paperos/
├── src/
│   ├── core/
│   ├── runtime/
│   ├── commands/
│   ├── events/
│   ├── extensions/
│   ├── workspace/
│   ├── ui/
│   └── main.rs
│
├── extensions/
│   ├── editor/
│   ├── terminal/
│   ├── filesystem/
│   ├── git/
│   ├── search/
│   └── lsp/
│
├── ui/
│   ├── src/
│   ├── styles/
│   └── index.html
│
└── docs/
```

Do not prematurely split every module into separate crates.

---

# 43. Current Editor → Future PaperOS

The existing editor should evolve incrementally.

Current:

```text
PaperOS
 ├── EditorRuntime
 ├── BufferActor
 ├── Commands
 ├── Lua
 └── Vi modes
```

Next:

```text
PaperOS
 ├── Core
 ├── Runact runtime
 ├── Extension manager
 ├── Command bus
 ├── Event bus
 ├── Buffer extension
 ├── UI bridge
 └── Web UI
```

Later:

```text
PaperOS
 ├── Editor
 ├── Terminal
 ├── Files
 ├── Git
 ├── LSP
 ├── Search
 ├── AI
 ├── Automation
 └── User extensions
```

---

# 44. Vertical Slice

Before implementing dozens of features, build one complete vertical slice.

The first complete flow should be:

```text
PaperOS starts
     ↓
Runact starts
     ↓
Extension manager starts
     ↓
Editor extension loads
     ↓
BufferActor starts
     ↓
Web UI connects
     ↓
User opens file
     ↓
Command generated
     ↓
Command routed
     ↓
BufferActor changes state
     ↓
BufferChanged event
     ↓
UI receives event
     ↓
UI updates
```

This proves the architecture.

---

# 45. Second Vertical Slice

Then implement:

```text
Open
Edit
Undo
Redo
Save
```

Entirely through:

```text
Commands
Actors
Events
UI
```

No shortcut implementation should bypass the architecture.

---

# 46. Third Vertical Slice

Add:

```text
Search
```

Flow:

```text
Search Command
     ↓
SearchActor
     ↓
Compute Pool
     ↓
Search Result
     ↓
Event
     ↓
Search View
```

This proves Runact's CPU-work integration.

---

# 47. Fourth Vertical Slice

Add:

```text
Terminal
```

Flow:

```text
Terminal Command
     ↓
TerminalActor
     ↓
OS Process
     ↓
Output Event
     ↓
Terminal View
```

This proves resource ownership and process lifecycle.

---

# 48. Fifth Vertical Slice

Add:

```text
Lua Extension
```

Example:

```text
Lua
 ↓
register command
 ↓
command palette
 ↓
command execution
 ↓
event
 ↓
UI
```

This proves that the system is genuinely extensible.

---

# 49. Security Model

Extensions should be treated as potentially untrusted.

Potential permission classes:

```text
filesystem.read
filesystem.write
network
process.spawn
shell
clipboard
workspace
ui
secrets
```

An extension should declare required capabilities.

The user/runtime decides whether they are granted.

---

# 50. Remote Future

The architecture should eventually allow:

```text
Local UI
     │
     ▼
PaperOS Core
```

and:

```text
Browser
     │
     ▼
Remote PaperOS
```

Potentially:

```text
VS Code-like local client
Browser client
Terminal client
Mobile client
```

The core does not need to know which UI is connected.

---

# 51. Automation

Because everything is commands and events, automation becomes natural.

Example:

```text
FileSaved
   ↓
Run formatter
   ↓
Run tests
   ↓
Run Git status
   ↓
Notify user
```

This can be implemented as an automation extension.

---

# 52. User-Defined Workflows

Eventually users should be able to define workflows.

Example conceptual Lua:

```lua
on("file.saved", function(event)
    commands.execute("editor.format")
    commands.execute("test.run")
end)
```

This turns PaperOS into a programmable environment.

---

# 53. Metaprogramming

Metaprogramming should be considered at the extension layer, not the core.

The core should remain predictable.

Lua can provide:

```text
dynamic commands
dynamic keybindings
dynamic workflows
dynamic UI configuration
dynamic automation
```

Future WASM/Rust extensions can provide stronger compiled functionality.

---

# 54. Core vs Extension Boundary

The most important architectural decision is what belongs in the core.

### Core

```text
Runtime
Actors
Commands
Events
Extension lifecycle
Capabilities
Resources
UI protocol
Configuration interfaces
Workspace interfaces
```

### Extensions

```text
Editor
Terminal
Filesystem
Git
LSP
Search
AI
Database
Markdown
Debugging
Testing
Docker
Cloud
Automation
```

When uncertain:

> Prefer extension unless the feature is required to make the extension system itself work.

---

# 55. Architectural Invariants

### Invariant 1

The core must remain small.

### Invariant 2

The UI must not own authoritative application state.

### Invariant 3

User actions become commands.

### Invariant 4

Important state changes produce events.

### Invariant 5

Extensions communicate through stable APIs.

### Invariant 6

Extensions receive explicit capabilities.

### Invariant 7

Long-running work must not block normal actor workers.

### Invariant 8

Editor functionality should not be inseparable from PaperOS core.

### Invariant 9

The same core should support multiple UI implementations.

### Invariant 10

A failed extension should not normally crash the whole environment.

---

# 56. Development Roadmap

## Phase 1 — Architecture

Implement/document:

```text
Core
Runact integration
Commands
Events
Extension lifecycle
Capabilities
UI protocol
```

Do not add many features.

---

## Phase 2 — Editor Vertical Slice

Implement:

```text
BufferActor
Open
Edit
Undo
Redo
Save
Web UI
```

---

## Phase 3 — Extension System

Implement:

```text
Extension manifest
Extension loading
Lua API
Commands
Events
Configuration
Capabilities
```

---

## Phase 4 — Development Environment

Add:

```text
Filesystem
Terminal
Search
Git
LSP
```

---

## Phase 5 — Automation

Add:

```text
Workflows
Tasks
Timers
Event handlers
Lua automation
```

---

## Phase 6 — AI

Add:

```text
AI provider abstraction
AI actors
Tool system
Context system
Agent workflows
Permission system
```

---

## Phase 7 — Extension Ecosystem

Add:

```text
Package manager
Extension registry
Versioning
Dependency resolution
Updates
Sandboxing
```

---

# 57. Long-Term Architecture

The eventual system should look like:

```text
                         PaperOS
                            │
                     ┌──────▼──────┐
                     │ Small Core  │
                     └──────┬──────┘
                            │
                     ┌──────▼──────┐
                     │   Runact    │
                     └──────┬──────┘
                            │
       ┌────────────────────┼─────────────────────┐
       │                    │                     │
    Commands             Events              Capabilities
       │                    │                     │
       └────────────────────┼─────────────────────┘
                            │
                  ┌─────────▼──────────┐
                  │    Extensions      │
                  └─────────┬──────────┘
                            │
     ┌────────┬────────┬────┼────┬────────┬────────┐
     │        │        │    │    │        │        │
   Editor  Terminal  Files Git  LSP     Search     AI
     │        │        │    │    │        │        │
     └────────┴────────┴────┼────┴────────┴────────┘
                            │
                       UI Protocol
                            │
                  ┌─────────▼─────────┐
                  │     Web UI        │
                  └───────────────────┘
```

---

# 58. Final Architectural Definition

PaperOS should be understood as:

> A programmable computing environment built on Runact, where application capabilities are implemented as isolated, composable extensions communicating through commands and events, with a Rust core maintaining authoritative state and a web-based UI acting as a client.

The editor is only the first major extension.

The terminal is an extension.

Git is an extension.

LSP is an extension.

AI is an extension.

Even parts of the UI are extensions.

The long-term product is therefore not simply:

```text
"Rust text editor"
```

but:

```text
Programmable Computing Environment
```

with:

```text
Runact
   ↓
PaperOS Core
   ↓
Extensions
   ↓
Commands + Events
   ↓
User-defined Environment
```
