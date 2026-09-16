use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Manifest for a paperOS extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub provides: Vec<String>,
}

/// ExtensionManager: discovers, loads, and manages extensions.
/// Reads from ~/.config/paperos/extensions/ and ~/.config/paperos/extensions-active/ (symlinks).
pub struct ExtensionManager {
    config_dir: PathBuf,
    extensions_dir: PathBuf,
    active_dir: PathBuf,
}

impl ExtensionManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("paperos");
        let extensions_dir = config_dir.join("extensions");
        let active_dir = config_dir.join("extensions-active");
        Self {
            config_dir,
            extensions_dir,
            active_dir,
        }
    }

    /// List all available extensions (installed + enabled/disabled status).
    pub fn list(&self) -> Vec<ExtensionInfo> {
        let mut result = Vec::new();

        if !self.extensions_dir.exists() {
            return result;
        }

        if let Ok(entries) = std::fs::read_dir(&self.extensions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(info) = self.read_extension_info(&path) {
                        let enabled = self.is_enabled(&info.name);
                        result.push(ExtensionInfo {
                            name: info.name,
                            version: info.version,
                            description: info.description,
                            author: info.author,
                            depends: info.depends,
                            provides: info.provides,
                            enabled,
                        });
                    }
                }
            }
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// Read manifest.json from an extension directory.
    fn read_extension_info(&self, dir: &Path) -> Option<ExtensionManifest> {
        let manifest_path = dir.join("manifest.json");
        if !manifest_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&manifest_path).ok()?;
        serde_json::from_str::<ExtensionManifest>(&content).ok()
    }

    /// Check if an extension is enabled (has a symlink in active dir).
    pub fn is_enabled(&self, name: &str) -> bool {
        let link_path = self.active_dir.join(name);
        link_path.exists() || link_path.is_symlink()
    }

    /// Enable an extension by creating a symlink.
    pub fn enable(&self, name: &str) -> Result<(), String> {
        let ext_path = self.extensions_dir.join(name);
        if !ext_path.exists() {
            return Err(format!(
                "Extension '{}' not found in extensions directory",
                name
            ));
        }

        std::fs::create_dir_all(&self.active_dir).map_err(|e| e.to_string())?;

        let link_path = self.active_dir.join(name);
        if link_path.exists() || link_path.is_symlink() {
            // Already enabled
            return Ok(());
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&ext_path, &link_path)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&ext_path, &link_path)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;

        Ok(())
    }

    /// Disable an extension by removing the symlink.
    pub fn disable(&self, name: &str) -> Result<(), String> {
        let link_path = self.active_dir.join(name);
        if !link_path.exists() && !link_path.is_symlink() {
            return Err(format!("Extension '{}' is not enabled", name));
        }

        std::fs::remove_file(&link_path).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Install an extension from a git URL.
    pub fn install_from_git(&self, url: &str, name: Option<&str>) -> Result<(), String> {
        std::fs::create_dir_all(&self.extensions_dir).map_err(|e| e.to_string())?;

        // Derive name from URL if not provided
        let ext_name = name.map(String::from).unwrap_or_else(|| {
            Path::new(url)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "extension".to_string())
        });

        let dest = self.extensions_dir.join(&ext_name);
        if dest.exists() {
            return Err(format!(
                "Extension '{}' already installed at {}",
                ext_name,
                dest.display()
            ));
        }

        // Clone via git
        let result = Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg(url)
            .arg(&dest)
            .status()
            .map_err(|e| format!("Failed to run git: {}", e))?;

        if !result.success() {
            return Err(format!(
                "Git clone failed with exit code: {:?}",
                result.code()
            ));
        }

        // Verify manifest exists
        if !dest.join("manifest.json").exists() {
            return Err(format!(
                "Extension '{}' does not contain manifest.json — cloning into {}",
                ext_name,
                dest.display()
            ));
        }

        Ok(())
    }

    /// Install from a local zip file.
    pub fn install_from_zip(&self, zip_path: &Path) -> Result<(), String> {
        let content = std::fs::read(zip_path).map_err(|e| e.to_string())?;
        let cursor = std::io::Cursor::new(content);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

        std::fs::create_dir_all(&self.extensions_dir).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => self.extensions_dir.join(path),
                None => continue,
            };

            if file.is_dir() {
                std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    /// Resolve dependency order using topological sort.
    /// Returns extensions in load order, or error on cycles/missing deps.
    pub fn resolve_load_order(&self) -> Result<Vec<String>, String> {
        let list = self.list();
        let enabled: Vec<&ExtensionInfo> = list.iter().filter(|e| e.enabled).collect();

        // Build name -> manifest map
        let mut manifests: HashMap<String, &ExtensionInfo> = HashMap::new();
        for ext in &enabled {
            manifests.insert(ext.name.clone(), *ext);
        }

        // Build adjacency list (dependency -> dependents)
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        for ext in &enabled {
            in_degree.entry(ext.name.clone()).or_insert(0);
            for dep in &ext.depends {
                // Parse dep name (strip version constraint)
                let dep_name = parse_dep_name(dep);
                if !manifests.contains_key(&dep_name) {
                    return Err(format!(
                        "Missing dependency '{}' required by '{}'",
                        dep_name, ext.name
                    ));
                }
                edges.entry(dep_name).or_default().push(ext.name.clone());
                *in_degree.get_mut(&ext.name).unwrap() += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut order: Vec<String> = Vec::new();

        while let Some(node) = queue.pop() {
            order.push(node.clone());
            if let Some(dependents) = edges.get(&node) {
                for dep in dependents {
                    if let Some(d) = in_degree.get_mut(dep) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        let enabled_names: Vec<String> = enabled.iter().map(|e| e.name.clone()).collect();
        if order.len() != enabled_names.len() {
            // Cycle detected
            let in_cycle: Vec<String> = enabled_names
                .iter()
                .filter(|n| !order.contains(n))
                .cloned()
                .collect();
            return Err(format!(
                "Circular dependency detected: {}",
                in_cycle.join(" -> ")
            ));
        }

        Ok(order)
    }
}

/// Parse a dependency spec like "name>=1.0" to just "name"
fn parse_dep_name(spec: &str) -> String {
    spec.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub depends: Vec<String>,
    pub provides: Vec<String>,
    pub enabled: bool,
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dep_name() {
        assert_eq!(parse_dep_name("buffer-api"), "buffer-api");
        assert_eq!(parse_dep_name("buffer-api>=0.2"), "buffer-api");
        assert_eq!(parse_dep_name("config==1.0.0"), "config");
        assert_eq!(parse_dep_name("some-ext>=0.1,<1.0"), "some-ext");
    }

    #[test]
    fn test_extension_info_structure() {
        let info = ExtensionInfo {
            name: "test-ext".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Tester".to_string(),
            depends: vec!["buffer-api>=0.2".to_string()],
            provides: vec!["testing".to_string()],
            enabled: true,
        };
        assert_eq!(info.name, "test-ext");
        assert_eq!(info.version, "1.0.0");
        assert!(info.enabled);
    }

    #[test]
    fn test_manager_creation() {
        let mgr = ExtensionManager::new();
        assert!(mgr.extensions_dir.ends_with("paperos/extensions"));
        assert!(mgr.active_dir.ends_with("paperos/extensions-active"));
    }

    #[test]
    fn test_list_empty_when_no_dir() {
        let mgr = ExtensionManager::new();
        let list = mgr.list();
        // May be empty or have system extensions — just verify it doesn't crash
        assert!(list.len() <= 100); // sanity check
    }
}
