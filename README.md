# paperOS

A programmable computing environment built on the [Runact](https://github.com/) actor runtime.

## Features

- **Everything is a Command**: Every user action is a named command routed through the command bus
- **Minimal Core**: The core only contains the runtime, actor management, command routing, event routing, extension lifecycle, and capability management
- **Extensions Are First-Class**: Editor, terminal, Git, LSP, search, AI, and other capabilities are composable extensions
- **Lua Extension System**: Configuration and extensions are written in Lua — no recompilation needed for customization
- **Crash Isolation**: Extension failures don't crash the environment — Runact supervision restarts failed components
- **Multiple UIs**: The same core supports web, desktop, terminal, and remote UIs through a stable UI protocol

## Installation

```bash
git clone <repo>
cd paperos
cargo build --release
```

## Usage

```bash
# Open a file
paperos myfile.txt

# Show help
paperos --help

# Show version
paperos --version
```

## Configuration

Configuration is loaded from `~/.config/paperos/init.lua`:

```lua
-- Basic settings
paper.config.set("theme", "gruvbox-dark")
paper.config.set("tab_width", 4)

-- Register a custom command
paper.commands.register("greet", function()
  print("Hello from paperOS!")
end)

-- Keybinding
paper.keys.map("normal", "<leader>g", "greet")
```

## Architecture

See [docs/architecture.md](docs/architecture.md) for the full architecture document.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md) for the development roadmap.

## Secret Management

See [docs/secrets.md](docs/secrets.md) for the secure secret-management architecture (capability-based access, credential stores, redaction).

## License

[License here]
