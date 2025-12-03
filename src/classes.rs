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
pub struct ClassConfig {
    pub classes: Vec<ClassDefinition>,
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
        }
    }
}

/// Load class configuration from YAML file
pub fn load_classes(path: Option<&str>) -> ClassConfig {
    let config_path = match path {
        Some(p) => p.to_string(),
        None => {
            // Try default locations
            let default_locations = vec![
                "./classes.yaml",
                "~/.config/annotator/classes.yaml",
            ];

            let mut found_path = None;
            for loc in default_locations {
                let expanded = shellexpand::tilde(loc);
                if Path::new(expanded.as_ref()).exists() {
                    found_path = Some(expanded.to_string());
                    break;
                }
            }

            match found_path {
                Some(p) => p,
                None => return ClassConfig::default(),
            }
        }
    };

    // Expand tilde in path
    let expanded_path = shellexpand::tilde(&config_path);

    match std::fs::read_to_string(expanded_path.as_ref()) {
        Ok(content) => match serde_yaml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse class config YAML: {}. Using defaults.", e);
                ClassConfig::default()
            }
        },
        Err(e) => {
            eprintln!(
                "Failed to read class config file '{}': {}. Using defaults.",
                expanded_path, e
            );
            ClassConfig::default()
        }
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
