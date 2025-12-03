use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub annotation_modes: AnnotationModesConfig,
    #[serde(default)]
    pub dataset: DatasetConfig,
    #[serde(default)]
    pub classes: ClassesConfig,
    #[serde(default)]
    pub export: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: i32,
    #[serde(default = "default_true")]
    pub show_left_sidebar: bool,
    #[serde(default = "default_false")]
    pub show_right_sidebar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationModesConfig {
    #[serde(default = "default_true")]
    pub enable_points: bool,
    #[serde(default = "default_true")]
    pub enable_bboxes: bool,
    #[serde(default = "default_true")]
    pub enable_polygons: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    #[serde(default = "default_false")]
    pub randomize_order: bool,
    #[serde(default = "default_autosave_interval")]
    pub auto_save_interval_seconds: u64,
    #[serde(default)]
    pub recent_datasets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassesConfig {
    pub config_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    #[serde(default = "default_export_format")]
    pub default_format: String,
    #[serde(default = "default_coco_category_start_id")]
    pub coco_category_start_id: i32,
}

// Default value functions
fn default_theme() -> String {
    "dark".to_string()
}

fn default_sidebar_width() -> i32 {
    250
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_autosave_interval() -> u64 {
    5
}

fn default_export_format() -> String {
    "yolo".to_string()
}

fn default_coco_category_start_id() -> i32 {
    1
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            sidebar_width: default_sidebar_width(),
            show_left_sidebar: true,
            show_right_sidebar: false,
        }
    }
}

impl Default for AnnotationModesConfig {
    fn default() -> Self {
        Self {
            enable_points: true,
            enable_bboxes: true,
            enable_polygons: true,
        }
    }
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            randomize_order: false,
            auto_save_interval_seconds: 5,
            recent_datasets: Vec::new(),
        }
    }
}

impl Default for ClassesConfig {
    fn default() -> Self {
        Self { config_file: None }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_format: default_export_format(),
            coco_category_start_id: 1,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig::default(),
            annotation_modes: AnnotationModesConfig::default(),
            dataset: DatasetConfig::default(),
            classes: ClassesConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

/// Get the path to the config file
pub fn config_path() -> PathBuf {
    let config_dir = directories::ProjectDirs::from("", "", "annotator")
        .expect("Failed to determine config directory")
        .config_dir()
        .to_path_buf();
    config_dir.join("config.toml")
}

/// Load configuration from file, or return default if file doesn't exist
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to parse config file: {}. Using defaults.", e);
                    AppConfig::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read config file: {}. Using defaults.", e);
                AppConfig::default()
            }
        }
    } else {
        AppConfig::default()
    }
}

/// Save configuration to file
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();

    // Create config directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Serialize config to TOML
    let toml = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Write to file
    std::fs::write(&path, toml)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Add a dataset path to recent datasets list
pub fn add_recent_dataset(config: &mut AppConfig, path: String) {
    // Remove if already in list
    config.dataset.recent_datasets.retain(|p| p != &path);

    // Add to front
    config.dataset.recent_datasets.insert(0, path);

    // Keep only last 10
    config.dataset.recent_datasets.truncate(10);
}
