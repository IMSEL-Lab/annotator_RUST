// Export module for various annotation formats

pub mod coco;
pub mod voc;

/// Export format types
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    CocoJson,
    PascalVoc,
}

/// Export result with statistics
#[allow(dead_code)]
#[derive(Debug)]
pub struct ExportResult {
    pub images_exported: usize,
    pub annotations_exported: usize,
}

#[allow(dead_code)]
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
