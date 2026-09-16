# paperOS

A programmable editor built on the [Runact](https://github.com/) actor runtime.

## Features

- **Everything is a Command**: Every editor action is a named command (`left`, `insert`, `undo`, `open`, `write`, `quit`, etc.)
- **Minimal Core**: The core only contains the document model (`BufferActor`), runtime scaffold (`EditorRuntime`), and extension interface (`Extension` trait)
- **Lua Extension System**: Configuration and extensions are written in Lua — no need to recompile for customization
- **Vi-style keybindings**: Built-in normal/insert/command modes with full undo/redo

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

## License

[License here]
