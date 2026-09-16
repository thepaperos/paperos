mod core;
mod extensions;

use crate::core::{CommandHandler, EditorRuntime};
use crate::extensions::{
    BufferCommands, BufferManager, FileExtension, InputExtension, LuaExtension, RenderExtension,
    SharedBufferCommands, SharedFileCommands, ViInputExtension, VisualModeExtension,
};
use std::sync::{Arc, Mutex, RwLock};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Handle CLI flags
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-V" => {
                println!("paperos v{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    // Find the first file argument (not a flag)
    let file_to_open: Option<String> = args.iter().skip(1).find(|a| !a.starts_with('-')).cloned();

    let mut runtime = EditorRuntime::new()?;

    // BufferManager: spawns BufferActor instances and tracks the active one.
    // Keeps core minimal — buffer lifecycle is an extension concern.
    // Wrapped in Arc<Mutex> so command handlers can switch buffers.
    let mut buf_mgr = BufferManager::new();

    // Load file content if provided
    let file_content = file_to_open
        .as_ref()
        .and_then(|f| std::fs::read_to_string(f).ok());

    let buffer = buf_mgr.add_buffer(runtime.runtime_mut(), file_content, file_to_open.clone());

    // Register buffer operation commands (movement, insert, delete, undo, redo)
    let mut buffer_cmds = BufferCommands::new();
    buffer_cmds.set_buffer(buffer);
    let shared_buffer_cmds = SharedBufferCommands::new(buffer_cmds);
    let buffer_command_names = [
        "h",
        "j",
        "k",
        "l",
        "w",
        "b",
        "e",
        "G",
        "u",
        "U",
        "0",
        "$",
        "left",
        "right",
        "up",
        "down",
        "home",
        "end",
        "nextword",
        "prevword",
        "endword",
        "lastline",
        "insert",
        "delete",
        "deleteline",
        "newline",
        "undo",
        "redo",
        "ctrl-r",
        "go",
        "line",
    ];
    for cmd in &buffer_command_names {
        runtime.register_command(
            cmd.to_string(),
            Box::new(shared_buffer_cmds.clone_handler()),
        );
    }

    // Register file I/O commands (:e, :w)
    let file_ext = Arc::new(Mutex::new({
        let mut f = FileExtension::new();
        f.set_buffer(buffer);
        f
    }));
    let file_shared = SharedFileCommands::new(file_ext.clone());
    for cmd in &["e", "open", "edit", "w", "write", "save"] {
        runtime.register_command(cmd.to_string(), Box::new(file_shared.clone_handler()));
    }

    // Register quit command
    runtime.register_command("q".to_string(), Box::new(QuitCommand));
    runtime.register_command("quit".to_string(), Box::new(QuitCommand));

    // Add the LuaExtension — provides config loading + paper.* API
    let lua_ext = LuaExtension::new();
    runtime.add_extension(Box::new(lua_ext))?;

    // Check if user requested quit after config load
    if let Some(ext) = runtime.extension::<LuaExtension>() {
        if ext.is_quit_requested() {
            runtime.shutdown()?;
            return Ok(());
        }
    }

    // Create the InputExtension (basic/text-editor keybindings: arrows, direct typing)
    let input = Arc::new(RwLock::new(InputExtension::new()));

    // Create the ViInputExtension (vi-style keybindings: h/j/k/l, modes)
    let vi_input = Arc::new(RwLock::new(ViInputExtension::new()));

    // Create the VisualModeExtension (visual mode, yank, paste, search)
    let visual = Arc::new(RwLock::new({
        let mut v = VisualModeExtension::new();
        v.set_buffer(buffer);
        v
    }));

    // Set up the render extension (terminal UI + event loop)
    let mut render = RenderExtension::new();
    render.set_buffer(buffer);
    render.set_input(input);
    render.set_vi_input(vi_input);
    render.set_visual_mode(visual);

    render.run_event_loop(&mut runtime, Some(buffer))?;

    runtime.shutdown()?;
    Ok(())
}

/// Quit command handler — sets the quit flag on the LuaExtension.
struct QuitCommand;

fn print_help() {
    println!(
        "paperOS v{} — a programmable editor built on the Runact actor runtime",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("    paperos [OPTIONS] [FILE]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Show this help message");
    println!("    -V, --version    Print version and exit");
    println!();
    println!("ARGS:");
    println!("    <FILE>           File to open");
    println!();
    println!("CONFIG:");
    println!("    Configuration is loaded from ~/.config/paperos/init.lua");
    println!("    Extensions are loaded from ~/.config/paperos/extensions/");
}

impl CommandHandler for QuitCommand {
    fn handle(
        &mut self,
        _args: &[String],
        _runtime: &runact::Runtime,
        _buffer: Option<runact::ActorId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Quitting...");
        // Set quit flag so the event loop exits
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
