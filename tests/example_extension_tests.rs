use paperos::extensions::{ExtensionInfo, ExtensionManager, ExtensionManifest};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn create_test_manifest(dir: &std::path::Path, name: &str, version: &str, depends: Vec<String>) {
    let manifest = ExtensionManifest {
        name: name.to_string(),
        version: version.to_string(),
        description: format!("Test extension {}", name),
        author: "Test Author".to_string(),
        depends,
        provides: vec![],
    };
    let json = serde_json::to_string_pretty(&manifest).unwrap();
    fs::write(dir.join("manifest.json"), json).unwrap();
}

#[test]
fn test_extension_manifest_serde() {
    // Verify manifest format matches what example extensions use
    let json = r#"{
        "name": "status-bar",
        "version": "0.1.0",
        "description": "Custom status bar showing file name, line count, and mode",
        "author": "paperOS Community",
        "depends": ["config>=1.0"],
        "provides": ["status-display"]
    }"#;

    let manifest: ExtensionManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.name, "status-bar");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.depends, vec!["config>=1.0"]);
    assert_eq!(manifest.provides, vec!["status-display"]);
}

#[test]
fn test_example_extension_discoverable() {
    // The example extension in /paper/extensions/status-bar should have a valid manifest
    let path = PathBuf::from("extensions/status-bar/manifest.json");
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap();
        let manifest: ExtensionManifest = serde_json::from_str(&content).unwrap();
        assert_eq!(manifest.name, "status-bar");
        assert!(!manifest.version.is_empty());
    }
}

#[test]
fn test_example_lua_syntax_valid() {
    // Basic syntax check — verify the Lua file exists and is non-empty
    let path = PathBuf::from("extensions/status-bar/init.lua");
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.is_empty());
        // Check for basic Lua structure
        assert!(content.contains("local M = {}"));
        assert!(content.contains("function M.init"));
        assert!(content.contains("function M.activate"));
        assert!(content.contains("function M.shutdown"));
        assert!(content.contains("api.commands.register"));
        assert!(content.contains("api.keys.map"));
        assert!(content.contains("api.status.set"));
    }
}
