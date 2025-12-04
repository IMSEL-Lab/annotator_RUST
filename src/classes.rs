use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDefinition {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalClassNode {
    pub key: u8,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<HierarchicalClassNode>,
    // Leaf properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassConfig {
    pub classes: Vec<ClassDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hierarchy: Vec<HierarchicalClassNode>,
}

impl Default for ClassConfig {
    fn default() -> Self {
        Self {
            classes: vec![
                ClassDefinition {
                    id: 1,
                    name: "Class 1".to_string(),
                    color: Some("#ff0000".to_string()),
                    shortcut: Some("1".to_string()),
                },
                ClassDefinition {
                    id: 2,
                    name: "Class 2".to_string(),
                    color: Some("#00ff00".to_string()),
                    shortcut: Some("2".to_string()),
                },
                ClassDefinition {
                    id: 3,
                    name: "Class 3".to_string(),
                    color: Some("#0000ff".to_string()),
                    shortcut: Some("3".to_string()),
                },
                ClassDefinition {
                    id: 4,
                    name: "Class 4".to_string(),
                    color: Some("#ffff00".to_string()),
                    shortcut: Some("4".to_string()),
                },
                ClassDefinition {
                    id: 5,
                    name: "Class 5".to_string(),
                    color: Some("#ff00ff".to_string()),
                    shortcut: Some("5".to_string()),
                },
            ],
            hierarchy: Vec::new(),
        }
    }
}

/// Flatten a hierarchy into a list of ClassDefinition
pub fn flatten_hierarchy(nodes: &[HierarchicalClassNode]) -> Vec<ClassDefinition> {
    let mut classes = Vec::new();
    for node in nodes {
        if let (Some(id), Some(name)) = (node.id, &node.name) {
            classes.push(ClassDefinition {
                id,
                name: name.clone(),
                color: node.color.clone(),
                shortcut: None, // Shortcuts are handled by the hierarchy navigation
            });
        }
        classes.extend(flatten_hierarchy(&node.children));
    }
    classes
}

/// Load class configuration from YAML file
pub fn load_classes(path: Option<&str>) -> ClassConfig {
    // Preferred search order:
    //   1) explicit path (if provided)
    //   2) ./classes.yaml in the repo (requested default)
    //   3) ./coco_hierarchy.yaml
    //   4) ~/.config/annotator/classes.yaml
    let mut search_paths: Vec<String> = Vec::new();
    if let Some(p) = path {
        search_paths.push(p.to_string());
    }
    search_paths.push("./classes.yaml".to_string());
    search_paths.push("./coco_hierarchy.yaml".to_string());
    search_paths.push("~/.config/annotator/classes.yaml".to_string());

    for candidate in search_paths {
        let expanded = shellexpand::tilde(&candidate);
        let path_obj = Path::new(expanded.as_ref());
        if !path_obj.exists() {
            continue;
        }

        match try_load_class_file(path_obj) {
            Ok(cfg) => return cfg,
            Err(e) => eprintln!("Failed to parse class config '{}': {}", path_obj.display(), e),
        }
    }

    // As a final fallback, try to use the bundled default at compile time
    if let Ok(cfg) = parse_class_content(include_str!("../classes.yaml")) {
        return cfg;
    }

    // Nothing found/parsable; fall back to built-in defaults
    eprintln!("No class config found; using defaults.");
    ClassConfig::default()
}

/// Attempt to load a class file; returns an error string on failure so caller
/// can continue searching other candidates.
fn try_load_class_file(path: &Path) -> Result<ClassConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("read error {}: {}", path.display(), e))?;

    parse_class_content(&content)
}

/// Parse class YAML content, accepting either a hierarchy array or full ClassConfig
fn parse_class_content(content: &str) -> Result<ClassConfig, String> {
    // Try parsing as pure hierarchy first
    if let Ok(hierarchy) = serde_yaml::from_str::<Vec<HierarchicalClassNode>>(content) {
        let classes = flatten_hierarchy(&hierarchy);
        return Ok(ClassConfig { classes, hierarchy });
    }

    // Try parsing as full ClassConfig
    match serde_yaml::from_str::<ClassConfig>(content) {
        Ok(mut config) => {
            if !config.hierarchy.is_empty() && config.classes.is_empty() {
                config.classes = flatten_hierarchy(&config.hierarchy);
            }
            Ok(config)
        }
        Err(e) => Err(format!("yaml parse error: {}", e)),
    }
}

/// Get class name by ID, or return a default if not found
pub fn get_class_name(config: &ClassConfig, class_id: i32) -> String {
    config
        .classes
        .iter()
        .find(|c| c.id == class_id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| format!("Class {}", class_id))
}

/// Get class color by ID, or return None if not found
pub fn get_class_color(config: &ClassConfig, class_id: i32) -> Option<String> {
    config
        .classes
        .iter()
        .find(|c| c.id == class_id)
        .and_then(|c| c.color.clone())
}

/// Save class configuration to YAML file
pub fn save_classes(config: &ClassConfig, path: &str) -> Result<(), String> {
    let expanded_path = shellexpand::tilde(path);

    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(expanded_path.as_ref()).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Serialize to YAML
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| format!("Failed to serialize class config: {}", e))?;

    // Write to file
    std::fs::write(expanded_path.as_ref(), yaml)
        .map_err(|e| format!("Failed to write class config file: {}", e))?;

    Ok(())
}

/// Create a default classes.yaml file in the config directory
pub fn create_default_classes_file() -> Result<String, String> {
    let config_dir = directories::ProjectDirs::from("", "", "annotator")
        .expect("Failed to determine config directory")
        .config_dir()
        .to_path_buf();

    let classes_path = config_dir.join("classes.yaml");

    // Create config directory if it doesn't exist
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    // Create default config
    let default_config = ClassConfig::default();
    save_classes(&default_config, classes_path.to_str().unwrap())?;

    Ok(classes_path.to_str().unwrap().to_string())
}
