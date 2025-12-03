use image::{DynamicImage, GrayImage, ImageBuffer, Luma};
use imageproc::filter::gaussian_blur_f32;
use std::path::Path;

/// Smart auto-resize using Sobel edge detection
/// Adjusts bbox edges to fit actual object blobs within ±30% search range
pub fn smart_auto_resize(
    image_path: &Path,
    bbox: (f32, f32, f32, f32),  // (x, y, width, height)
    image_size: (f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    let (x, y, width, height) = bbox;
    let (img_w, img_h) = image_size;

    // Load and process image
    let img = image::open(image_path).ok()?;
    let gray = img.to_luma8();

    // Apply Gaussian blur to reduce noise
    let blurred = gaussian_blur_f32(&gray, 1.5);

    // Compute Sobel gradients
    let gradient = compute_gradient_magnitude(&blurred);

    // Define search range (±30% of bbox dimensions)
    let search_w = (width * 0.3).max(5.0);
    let search_h = (height * 0.3).max(5.0);

    // Find best edges for each side
    let new_left = find_best_vertical_edge(
        &gradient,
        x,
        x - search_w,
        x + search_w,
        y,
        y + height,
        img_w,
        img_h,
    );

    let new_right = find_best_vertical_edge(
        &gradient,
        x + width,
        x + width - search_w,
        x + width + search_w,
        y,
        y + height,
        img_w,
        img_h,
    );

    let new_top = find_best_horizontal_edge(
        &gradient,
        y,
        y - search_h,
        y + search_h,
        x,
        x + width,
        img_w,
        img_h,
    );

    let new_bottom = find_best_horizontal_edge(
        &gradient,
        y + height,
        y + height - search_h,
        y + height + search_h,
        x,
        x + width,
        img_w,
        img_h,
    );

    // Compute new bbox
    let new_x = new_left;
    let new_y = new_top;
    let new_width = new_right - new_left;
    let new_height = new_bottom - new_top;

    // Validate new bbox
    if new_width < 10.0 || new_height < 10.0 {
        // If result is too small, return original
        return Some(bbox);
    }

    // Clamp to image bounds
    let final_x = new_x.max(0.0);
    let final_y = new_y.max(0.0);
    let final_width = (new_x + new_width).min(img_w) - final_x;
    let final_height = (new_y + new_height).min(img_h) - final_y;

    Some((final_x, final_y, final_width, final_height))
}

/// Compute gradient magnitude using Sobel operator
fn compute_gradient_magnitude(img: &GrayImage) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let (width, height) = img.dimensions();
    let mut gradient = ImageBuffer::new(width, height);

    // Sobel kernels
    let sobel_x = [
        [-1.0, 0.0, 1.0],
        [-2.0, 0.0, 2.0],
        [-1.0, 0.0, 1.0],
    ];

    let sobel_y = [
        [-1.0, -2.0, -1.0],
        [ 0.0,  0.0,  0.0],
        [ 1.0,  2.0,  1.0],
    ];

    // Apply Sobel operator
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let mut gx = 0.0;
            let mut gy = 0.0;

            // Convolve with Sobel kernels
            for ky in 0..3 {
                for kx in 0..3 {
                    let pixel = img.get_pixel(x + kx - 1, y + ky - 1)[0] as f32;
                    gx += pixel * sobel_x[ky as usize][kx as usize];
                    gy += pixel * sobel_y[ky as usize][kx as usize];
                }
            }

            // Gradient magnitude
            let magnitude = (gx * gx + gy * gy).sqrt();
            gradient.put_pixel(x, y, Luma([magnitude]));
        }
    }

    gradient
}

/// Find best vertical edge (for left/right sides)
/// Scans column range and finds column with highest average gradient
fn find_best_vertical_edge(
    gradient: &ImageBuffer<Luma<f32>, Vec<f32>>,
    center: f32,
    min_x: f32,
    max_x: f32,
    y_start: f32,
    y_end: f32,
    img_w: f32,
    img_h: f32,
) -> f32 {
    let min_x = min_x.max(0.0).min(img_w - 1.0);
    let max_x = max_x.max(0.0).min(img_w - 1.0);
    let y_start = y_start.max(0.0).min(img_h - 1.0);
    let y_end = y_end.max(0.0).min(img_h - 1.0);

    if min_x >= max_x || y_start >= y_end {
        return center;
    }

    let mut best_x = center;
    let mut best_score = 0.0;

    // Sample every column in range
    let step = ((max_x - min_x) / 30.0).max(1.0);
    let mut x = min_x;

    while x <= max_x {
        let ix = x as u32;
        let mut score = 0.0;
        let mut count = 0;

        // Average gradient along this column
        let y_step = ((y_end - y_start) / 20.0).max(1.0);
        let mut y = y_start;

        while y <= y_end {
            let iy = y as u32;
            if ix < gradient.width() && iy < gradient.height() {
                score += gradient.get_pixel(ix, iy)[0];
                count += 1;
            }
            y += y_step;
        }

        if count > 0 {
            score /= count as f32;
        }

        if score > best_score {
            best_score = score;
            best_x = x;
        }

        x += step;
    }

    // If no strong edge found, return center
    if best_score < 10.0 {
        return center;
    }

    best_x
}

/// Find best horizontal edge (for top/bottom sides)
/// Scans row range and finds row with highest average gradient
fn find_best_horizontal_edge(
    gradient: &ImageBuffer<Luma<f32>, Vec<f32>>,
    center: f32,
    min_y: f32,
    max_y: f32,
    x_start: f32,
    x_end: f32,
    img_w: f32,
    img_h: f32,
) -> f32 {
    let min_y = min_y.max(0.0).min(img_h - 1.0);
    let max_y = max_y.max(0.0).min(img_h - 1.0);
    let x_start = x_start.max(0.0).min(img_w - 1.0);
    let x_end = x_end.max(0.0).min(img_w - 1.0);

    if min_y >= max_y || x_start >= x_end {
        return center;
    }

    let mut best_y = center;
    let mut best_score = 0.0;

    // Sample every row in range
    let step = ((max_y - min_y) / 30.0).max(1.0);
    let mut y = min_y;

    while y <= max_y {
        let iy = y as u32;
        let mut score = 0.0;
        let mut count = 0;

        // Average gradient along this row
        let x_step = ((x_end - x_start) / 20.0).max(1.0);
        let mut x = x_start;

        while x <= x_end {
            let ix = x as u32;
            if ix < gradient.width() && iy < gradient.height() {
                score += gradient.get_pixel(ix, iy)[0];
                count += 1;
            }
            x += x_step;
        }

        if count > 0 {
            score /= count as f32;
        }

        if score > best_score {
            best_score = score;
            best_y = y;
        }

        y += step;
    }

    // If no strong edge found, return center
    if best_score < 10.0 {
        return center;
    }

    best_y
}
