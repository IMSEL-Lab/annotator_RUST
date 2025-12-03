// Pascal VOC XML format export

use std::fs;
use std::path::Path;

pub struct VocObject {
    pub name: String,
    pub bbox: (i32, i32, i32, i32), // xmin, ymin, xmax, ymax
    pub difficult: i32,
    pub truncated: i32,
    pub pose: String,
}

pub struct VocAnnotation {
    pub folder: String,
    pub filename: String,
    pub path: String,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub objects: Vec<VocObject>,
}

impl VocAnnotation {
    pub fn new(filename: String, width: i32, height: i32) -> Self {
        VocAnnotation {
            folder: "images".to_string(),
            filename,
            path: String::new(),
            width,
            height,
            depth: 3, // RGB
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, name: String, xmin: i32, ymin: i32, xmax: i32, ymax: i32) {
        self.objects.push(VocObject {
            name,
            bbox: (xmin, ymin, xmax, ymax),
            difficult: 0,
            truncated: 0,
            pose: "Unspecified".to_string(),
        });
    }

    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<annotation>\n");
        xml.push_str(&format!("  <folder>{}</folder>\n", self.folder));
        xml.push_str(&format!("  <filename>{}</filename>\n", self.filename));
        xml.push_str(&format!("  <path>{}</path>\n", self.path));
        xml.push_str("  <source>\n");
        xml.push_str("    <database>Unknown</database>\n");
        xml.push_str("  </source>\n");
        xml.push_str("  <size>\n");
        xml.push_str(&format!("    <width>{}</width>\n", self.width));
        xml.push_str(&format!("    <height>{}</height>\n", self.height));
        xml.push_str(&format!("    <depth>{}</depth>\n", self.depth));
        xml.push_str("  </size>\n");
        xml.push_str("  <segmented>0</segmented>\n");

        for obj in &self.objects {
            xml.push_str("  <object>\n");
            xml.push_str(&format!("    <name>{}</name>\n", obj.name));
            xml.push_str(&format!("    <pose>{}</pose>\n", obj.pose));
            xml.push_str(&format!("    <truncated>{}</truncated>\n", obj.truncated));
            xml.push_str(&format!("    <difficult>{}</difficult>\n", obj.difficult));
            xml.push_str("    <bndbox>\n");
            xml.push_str(&format!("      <xmin>{}</xmin>\n", obj.bbox.0));
            xml.push_str(&format!("      <ymin>{}</ymin>\n", obj.bbox.1));
            xml.push_str(&format!("      <xmax>{}</xmax>\n", obj.bbox.2));
            xml.push_str(&format!("      <ymax>{}</ymax>\n", obj.bbox.3));
            xml.push_str("    </bndbox>\n");
            xml.push_str("  </object>\n");
        }

        xml.push_str("</annotation>\n");
        xml
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let xml = self.to_xml();
        fs::write(path, xml)
            .map_err(|e| format!("Failed to write VOC XML: {e}"))?;
        Ok(())
    }
}
