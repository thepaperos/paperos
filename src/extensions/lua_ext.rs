use crate::core::Extension;
use mlua::{Lua, Result as LuaResult};
use runact::Runtime;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// LuaExtension: embeds a Lua runtime and exposes the `paper` global API.
/// Users write config/extensions in Lua at ~/.config/paperos/init.lua.
#[allow(dead_code)]
pub struct LuaExtension {
    pub lua: Lua,
    config: Arc<Mutex<HashMap<String, String>>>,
    keybindings: Arc<Mutex<Vec<(String, String, String)>>>,
    quit_requested: Arc<Mutex<bool>>,
}

impl Extension for LuaExtension {
    fn name(&self) -> &str {
        "lua"
    }

    fn init(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        self.setup_paper_api()?;
        self.load_config()?;
        Ok(())
    }

    fn shutdown(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn on_key(&mut self, _ch: char) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn on_render(&mut self, _runtime: &Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl LuaExtension {
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
            config: Arc::new(Mutex::new(HashMap::new())),
            keybindings: Arc::new(Mutex::new(Vec::new())),
            quit_requested: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_quit_requested(&self) -> bool {
        *self.quit_requested.lock().unwrap()
    }

    #[allow(dead_code)]
    pub fn keybindings(&self) -> Vec<(String, String, String)> {
        self.keybindings.lock().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn config_value(&self, key: &str) -> Option<String> {
        self.config.lock().unwrap().get(key).cloned()
    }

    fn setup_paper_api(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let globals = self.lua.globals();
        let paper = self.lua.create_table()?;

        // paper.config — set/get typed settings
        let cfg = self.config.clone();
        let config_table = self.lua.create_table()?;
        {
            let cfg_ref = cfg.clone();
            config_table.set(
                "set",
                self.lua
                    .create_function(move |_, (key, value): (String, mlua::Value)| {
                        let mut map = cfg_ref.lock().unwrap();
                        let s = match value {
                            mlua::Value::String(s) => s.to_str()?.to_string(),
                            mlua::Value::Integer(n) => n.to_string(),
                            mlua::Value::Number(n) => n.to_string(),
                            mlua::Value::Boolean(b) => b.to_string(),
                            mlua::Value::Nil => "nil".to_string(),
                            _ => {
                                return Err(mlua::Error::FromLuaConversionError {
                                    from: "value",
                                    to: "string".to_string(),
                                    message: Some("unsupported value type".to_string()),
                                })
                            }
                        };
                        map.insert(key, s);
                        Ok(())
                    })?,
            )?;
        }
        {
            let cfg_ref = cfg.clone();
            config_table.set(
                "get",
                self.lua
                    .create_function(move |_, key: String| -> LuaResult<String> {
                        let map = cfg_ref.lock().unwrap();
                        Ok(map.get(&key).cloned().unwrap_or_default())
                    })?,
            )?;
        }
        paper.set("config", config_table)?;

        // paper.commands — register named commands (for Lua to call)
        let cmds = self.lua.create_table()?;
        cmds.set(
            "register",
            self.lua
                .create_function(move |lua, (name, func): (String, mlua::Function)| {
                    lua.globals()
                        .get::<mlua::Table>("paper")?
                        .get::<mlua::Table>("commands")?
                        .set(name, func)?;
                    Ok(())
                })?,
        )?;
        paper.set("commands", cmds)?;

        // paper.keys — key remapping
        let keys = self.keybindings.clone();
        let keys_table = self.lua.create_table()?;
        keys_table.set(
            "map",
            self.lua
                .create_function(move |_, (mode, key, cmd): (String, String, String)| {
                    let mut kb = keys.lock().unwrap();
                    kb.push((mode, key, cmd));
                    Ok(())
                })?,
        )?;
        paper.set("keys", keys_table)?;

        // paper.extensions — query available extensions
        let ext_table = self.lua.create_table()?;
        ext_table.set(
            "list",
            self.lua.create_function(|_, ()| {
                let exts = vec!["lua", "input", "render", "file", "buffer"];
                let lua = mlua::Lua::new();
                let t = lua.create_table()?;
                for (i, e) in exts.iter().enumerate() {
                    t.set(i + 1, *e)?;
                }
                Ok(t)
            })?,
        )?;
        paper.set("extensions", ext_table)?;

        // paper.quit
        let quit = self.quit_requested.clone();
        paper.set(
            "quit",
            self.lua.create_function(move |_, _: ()| {
                let mut q = quit.lock().unwrap();
                *q = true;
                Ok(())
            })?,
        )?;

        // paper.buffer — buffer operations (simple ones)
        let buffer_table = self.lua.create_table()?;
        buffer_table.set(
            "insert_text",
            self.lua.create_function(|_, _text: String| {
                // In a full implementation, this would send a message to the buffer actor.
                // For now, config scripts can register handlers via commands.
                println!("buffer.insert_text would send to buffer actor");
                Ok(())
            })?,
        )?;
        paper.set("buffer", buffer_table)?;

        globals.set("paper", paper)?;
        Ok(())
    }

    fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = self.config_path();
        if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)?;
            match self.lua.load(&config_str).exec() {
                Ok(_) => {
                    println!("Loaded config from {}", config_path.display());
                }
                Err(e) => {
                    eprintln!("Config error: {}", e);
                }
            }
        } else {
            println!(
                "No config found at {}, using defaults",
                config_path.display()
            );
        }
        Ok(())
    }

    fn config_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(format!("{}/.config/paperos/init.lua", home))
    }
}

impl Default for LuaExtension {
    fn default() -> Self {
        Self::new()
    }
}
