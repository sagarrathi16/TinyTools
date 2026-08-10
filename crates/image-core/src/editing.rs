use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::types::ToolResult;

pub fn smart_crop(
    input_path: String,
    output_path: String,
    width: u32,
    height: u32,
    gravity: String,
) -> Result<ToolResult, String> {
    let mut img = image::open(&input_path).map_err(|e| e.to_string())?;
    let (w, h) = img.dimensions();

    let (sx, sy) = match gravity.as_str() {
        "center" => ((w.saturating_sub(width)) / 2, (h.saturating_sub(height)) / 2),
        "top" => (0, 0),
        "bottom" => (0, h.saturating_sub(height)),
        "left" => (0, 0),
        "right" => (w.saturating_sub(width), 0),
        _ => ((w.saturating_sub(width)) / 2, (h.saturating_sub(height)) / 2),
    };

    let cropped = img.crop(sx, sy, width.min(w - sx), height.min(h - sy));
    cropped.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Cropped".into() })
}

pub fn expand_canvas(
    input_path: String,
    output_path: String,
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
    color: String,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let new_w = w + left + right;
    let new_h = h + top + bottom;
    let mut out = RgbaImage::new(new_w, new_h);

    let c = parse_color(&color);
    for y in 0..new_h {
        for x in 0..new_w {
            out.put_pixel(x, y, c);
        }
    }

    for y in 0..h {
        for x in 0..w {
            let p = *rgba.get_pixel(x, y);
            out.put_pixel(x + left, y + top, p);
        }
    }

    out.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Canvas expanded".into() })
}

pub fn split_image(
    input_path: String,
    output_dir: String,
    rows: u32,
    cols: u32,
) -> Result<ToolResult, String> {
    let mut img = image::open(&input_path).map_err(|e| e.to_string())?;
    let (w, h) = img.dimensions();
    let cell_w = w / cols;
    let cell_h = h / rows;

    let mut count = 0;
    for r in 0..rows {
        for c in 0..cols {
            let cropped = img.crop(c * cell_w, r * cell_h, cell_w, cell_h);
            let path = format!("{}/{}_{}.png", output_dir, r, c);
            cropped.save(&path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(ToolResult {
        success: true,
        output_path: Some(output_dir),
        message: format!("Split into {}x{} = {} pieces", rows, cols, count),
    })
}

pub fn stitch_images(
    paths: Vec<String>,
    output_path: String,
    direction: String,
) -> Result<ToolResult, String> {
    if paths.is_empty() {
        return Err("No images provided".into());
    }

    let imgs: Vec<DynamicImage> = paths
        .iter()
        .map(|p| image::open(p))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let total_w: u32 = match direction.as_str() {
        "horizontal" => imgs.iter().map(|i| i.width()).sum(),
        _ => imgs.iter().map(|i| i.width()).max().unwrap_or(0),
    };
    let total_h: u32 = match direction.as_str() {
        "vertical" => imgs.iter().map(|i| i.height()).sum(),
        _ => imgs.iter().map(|i| i.height()).max().unwrap_or(0),
    };

    let mut out = RgbaImage::new(total_w, total_h);
    let mut offset = 0u32;

    for img in &imgs {
        let rgba = img.to_rgba8();
        match direction.as_str() {
            "horizontal" => {
                for y in 0..rgba.height().min(total_h) {
                    for x in 0..rgba.width() {
                        let p = *rgba.get_pixel(x, y);
                        out.put_pixel(x + offset, y, p);
                    }
                }
                offset += img.width();
            }
            _ => {
                for y in 0..rgba.height() {
                    for x in 0..rgba.width().min(total_w) {
                        let p = *rgba.get_pixel(x, y);
                        out.put_pixel(x, y + offset, p);
                    }
                }
                offset += img.height();
            }
        }
    }

    out.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Images stitched".into() })
}

fn parse_color(hex: &str) -> Rgba<u8> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            Rgba([r, g, b, 255])
        }
        _ => Rgba([255, 255, 255, 255]),
    }
}