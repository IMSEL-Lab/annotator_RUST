//! Utility functions for the annotator application.

use slint::SharedPixelBuffer;

/// Create a placeholder checkerboard image for when no dataset is loaded
pub fn placeholder_image() -> slint::Image {
    let width = 64u32;
    let height = 64u32;
    let mut buffer = SharedPixelBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let v = if (x / 8 + y / 8) % 2 == 0 { 60 } else { 110 };
            let i = ((y * width + x) * 3) as usize;
            let data = buffer.make_mut_bytes();
            data[i] = v;
            data[i + 1] = v;
            data[i + 2] = v;
        }
    }
    slint::Image::from_rgb8(buffer)
}

/// Parse a hex color string (e.g., "#ff0000") to a Slint Color
pub fn parse_color(hex: &str) -> Option<slint::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(slint::Color::from_rgb_u8(r, g, b))
    } else {
        None
    }
}
