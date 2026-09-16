-- paperOS sample configuration
-- This file is loaded from ~/.config/paperos/init.lua

-- Basic settings
paper.config.set("theme", "gruvbox-dark")
paper.config.set("tab_width", 4)
paper.config.set("line_numbers", true)

-- Register a custom command
paper.commands.register("greet", function()
  print("Hello from paperOS!")
end)

-- Keybinding: <leader>g triggers the greet command
paper.keys.map("normal", "<leader>g", "greet")

-- Quit command
paper.commands.register("q", function()
  paper.quit()
end)
