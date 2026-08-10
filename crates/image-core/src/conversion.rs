use image::codecs::jpeg::JpegEncoder;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use crate::types::ToolResult;

pub fn compress_image(
    input_path: String,
    output_path: String,
    quality: u8,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let ext = get_extension(&output_path);

    match ext.as_str() {
        "jpg" | "jpeg" => {
            let rgb = img.to_rgb8();
            let mut buf = Cursor::new(Vec::new());
            let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
            rgb.write_with_encoder(encoder).map_err(|e| e.to_string())?;
            fs::write(&output_path, buf.into_inner()).map_err(|e| e.to_string())?;
        }
        _ => {
            img.save(&output_path).map_err(|e| e.to_string())?;
        }
    }

    Ok(ToolResult {
        success: true,
        output_path: Some(output_path),
        message: "Image compressed successfully".to_string(),
    })
}

pub fn convert_format(
    input_path: String,
    output_path: String,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let ext = get_extension(&output_path).to_uppercase();
    img.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult {
        success: true,
        output_path: Some(output_path),
        message: format!("Converted to {}", ext),
    })
}

pub fn convert_heic(input_path: String, output_path: String) -> Result<ToolResult, String> {
    match image::open(&input_path) {
        Ok(img) => {
            img.save(&output_path).map_err(|e| format!("HEIC decoding not supported on this system: {}", e))?;
            Ok(ToolResult { success: true, output_path: Some(output_path), message: "HEIC converted".into() })
        }
        Err(e) => Err(format!("HEIC decoding failed: {}. Install libheif on your system.", e)),
    }
}

pub fn raster_to_svg(input_path: String, output_path: String) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let gray = img.grayscale().to_luma8();
    let (w, h) = gray.dimensions();

    let threshold = 128u8;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        w, h, w, h
    );
    svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);

    for y in 0..h {
        let mut in_path = false;
        let mut path_d = String::new();
        for x in 0..=w {
            let dark = if x < w {
                gray.get_pixel(x, y).0[0] < threshold
            } else {
                false
            };
            if dark && !in_path {
                path_d = format!("M{},{}", x, y);
                in_path = true;
            } else if !dark && in_path {
                path_d.push_str(&format!("L{},{}Z", x, y));
                svg.push_str(&format!(
                    r#"<path d="{}" fill="black"/>"#,
                    path_d
                ));
                in_path = false;
            }
        }
    }

    svg.push_str("</svg>");
    fs::write(&output_path, svg).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Vectorized to SVG".into() })
}

pub fn rotate_image(
    input_path: String,
    output_path: String,
    degrees: u32,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let rotated = match degrees % 360 {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => img,
    };
    rotated.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: format!("Rotated {}°", degrees) })
}

pub fn grayscale(input_path: String, output_path: String) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let gray = img.grayscale();
    gray.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Grayscale applied".into() })
}

pub fn blur_image(input_path: String, output_path: String, sigma: f32) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let blurred = img.blur(sigma as f32);
    blurred.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Blur applied".into() })
}

pub fn sharpen_image(input_path: String, output_path: String, amount: f32, radius: u32) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let sharpened = img.unsharpen(amount as f32, radius as i32);
    sharpened.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Sharpen applied".into() })
}

pub fn flip_image(input_path: String, output_path: String, direction: String) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let flipped = if direction == "vertical" { img.flipv() } else { img.fliph() };
    flipped.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Flip applied".into() })
}

pub fn resize_image(
    input_path: String,
    output_path: String,
    max_w: u32,
    max_h: u32,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let (w, h) = img.dimensions();
    let ratio = (max_w as f64 / w as f64).min(max_h as f64 / h as f64).min(1.0);
    let new_w = (w as f64 * ratio) as u32;
    let new_h = (h as f64 * ratio) as u32;
    let resized = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
    resized.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: format!("Resized to {}x{}", new_w, new_h) })
}

fn get_extension(path: &str) -> String {
    PathBuf::from(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase()
}