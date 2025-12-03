// COCO JSON format export
// http://cocodataset.org/#format-data

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoInfo {
    pub year: i32,
    pub version: String,
    pub description: String,
    pub contributor: String,
    pub date_created: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoImage {
    pub id: i32,
    pub width: i32,
    pub height: i32,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoAnnotation {
    pub id: i32,
    pub image_id: i32,
    pub category_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[f64; 4]>, // [x, y, width, height]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmentation: Option<Vec<Vec<f64>>>, // [[x1, y1, x2, y2, ...]]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<f64>,
    pub iscrowd: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoCategory {
    pub id: i32,
    pub name: String,
    pub supercategory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoDataset {
    pub info: CocoInfo,
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,
    pub categories: Vec<CocoCategory>,
}

impl CocoDataset {
    pub fn new() -> Self {
        let now = chrono::Local::now();
        CocoDataset {
            info: CocoInfo {
                year: now.year(),
                version: "1.0".to_string(),
                description: "Dataset exported from Annotator".to_string(),
                contributor: "Annotator".to_string(),
                date_created: now.format("%Y-%m-%d").to_string(),
            },
            images: Vec::new(),
            annotations: Vec::new(),
            categories: Vec::new(),
        }
    }

    pub fn add_category(&mut self, id: i32, name: String) {
        self.categories.push(CocoCategory {
            id,
            name,
            supercategory: "object".to_string(),
        });
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize COCO JSON: {e}"))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write COCO JSON: {e}"))?;
        Ok(())
    }
}
