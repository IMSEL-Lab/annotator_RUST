// Export module for various annotation formats

pub mod coco;
pub mod voc;

use std::path::Path;

/// Export format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    CocoJson,
    PascalVoc,
}

/// Export result with statistics
#[derive(Debug)]
pub struct ExportResult {
    pub images_exported: usize,
    pub annotations_exported: usize,
}

impl ExportFormat {
    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::CocoJson => "COCO JSON",
            ExportFormat::PascalVoc => "Pascal VOC XML",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::CocoJson => "json",
            ExportFormat::PascalVoc => "xml",
        }
    }
}
