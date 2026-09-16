use runact::Runtime;
use std::any::Any;

#[allow(dead_code)]
pub trait Extension: Any + Send + Sync {
    fn name(&self) -> &str;

    fn init(&mut self, runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>>;

    fn shutdown(&mut self, runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        let _ = runtime;
        Ok(())
    }

    fn on_key(&mut self, ch: char) -> Result<(), Box<dyn std::error::Error>> {
        let _ = ch;
        Ok(())
    }

    fn on_render(&mut self, runtime: &Runtime) -> Result<(), Box<dyn std::error::Error>> {
        let _ = runtime;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
