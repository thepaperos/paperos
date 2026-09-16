use super::extension::Extension;
use runact::Runtime;
use std::any::Any;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct EditorRuntime {
    runtime: Runtime,
    extensions: Vec<Box<dyn Extension>>,
    commands: HashMap<String, Box<dyn CommandHandler>>,
}

#[allow(dead_code)]
pub trait CommandHandler: Any + Send + Sync {
    fn handle(
        &mut self,
        args: &[String],
        runtime: &Runtime,
        buffer: Option<runact::ActorId>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[allow(dead_code)]
impl EditorRuntime {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;
        Ok(Self {
            runtime,
            extensions: Vec::new(),
            commands: HashMap::new(),
        })
    }

    pub fn add_extension(
        &mut self,
        mut ext: Box<dyn Extension>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        ext.init(&mut self.runtime)?;
        self.extensions.push(ext);
        Ok(())
    }

    pub fn register_command(&mut self, name: String, handler: Box<dyn CommandHandler>) {
        self.commands.insert(name, handler);
    }

    pub fn run_command(
        &mut self,
        input: &str,
        buffer: Option<runact::ActorId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let input = input.trim_start_matches(':');
        // Parse the command: first whitespace-separated token is the command name,
        // the rest of the string is the argument (preserved verbatim, including spaces).
        let parts: Vec<String> = if let Some(space_pos) = input.find(char::is_whitespace) {
            let cmd = input[..space_pos].to_string();
            // Take exactly the character after the space (for insert commands)
            let rest = input[space_pos + 1..].to_string();
            if rest.is_empty() {
                vec![cmd]
            } else {
                vec![cmd, rest]
            }
        } else {
            vec![input.to_string()]
        };
        if parts.is_empty() {
            return Ok(());
        }
        let cmd = parts[0].as_str();
        let args = &parts; // Pass full parts array (cmd is args[0])
        if let Some(handler) = self.commands.get_mut(cmd) {
            handler.handle(args, &self.runtime, buffer)
        } else {
            println!("Unknown command: {}", cmd);
            Ok(())
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for ext in &mut self.extensions {
            ext.shutdown(&mut self.runtime)?;
        }
        self.runtime.shutdown()?;
        Ok(())
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    pub fn extension<T: Extension + 'static>(&self) -> Option<&T> {
        self.extensions
            .iter()
            .find_map(|ext| ext.as_any().downcast_ref::<T>())
    }

    pub fn extension_mut<T: Extension + 'static>(&mut self) -> Option<&mut T> {
        self.extensions
            .iter_mut()
            .find_map(|ext| ext.as_any_mut().downcast_mut::<T>())
    }
}

impl Default for EditorRuntime {
    fn default() -> Self {
        Self {
            runtime: Runtime::new().expect("failed to create runtime"),
            extensions: Vec::new(),
            commands: HashMap::new(),
        }
    }
}
