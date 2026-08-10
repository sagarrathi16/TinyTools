use image::Rgba;
use serde::{Deserialize, Serialize};

use crate::types::ToolResult;

pub fn strip_metadata(input_path: String, output_path: String) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    img.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult {
        success: true,
        output_path: Some(output_path),
        message: "Metadata stripped (re-encoded without EXIF)".into(),
    })
}

pub fn redact_regions(
    input_path: String,
    output_path: String,
    regions: Vec<(u32, u32, u32, u32)>,
    method: String,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    for (rx, ry, rw, rh) in &regions {
        match method.as_str() {
            "pixelate" => {
                let block = 10u32;
                for by in (0..*rh).step_by(block as usize) {
                    for bx in (0..*rw).step_by(block as usize) {
                        let sx = rx + bx;
                        let sy = ry + by;
                        let p = *rgba.get_pixel(sx.min(w - 1), sy.min(h - 1));
                        for dy in 0..block.min(rh - by) {
                            for dx in 0..block.min(rw - bx) {
                                let px = (sx + dx).min(w - 1);
                                let py = (sy + dy).min(h - 1);
                                rgba.put_pixel(px, py, p);
                            }
                        }
                    }
                }
            }
            "blur" => {
                let sigma = 7.0f64;
                for y in *ry..(*ry + *rh).min(h) {
                    for x in *rx..(*rx + *rw).min(w) {
                        let mut sum_r: f64 = 0.0;
                        let mut sum_g: f64 = 0.0;
                        let mut sum_b: f64 = 0.0;
                        let mut count: f64 = 0.0;
                        for dy in -3i32..=3 {
                            for dx in -3i32..=3 {
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }
                                let p = *rgba.get_pixel(nx as u32, ny as u32);
                                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                                let weight = (-dist * dist / (2.0 * sigma * sigma)).exp();
                                sum_r += p[0] as f64 * weight;
                                sum_g += p[1] as f64 * weight;
                                sum_b += p[2] as f64 * weight;
                                count += weight;
                            }
                        }
                        if count > 0.0 {
                            rgba.put_pixel(
                                x, y,
                                Rgba([
                                    (sum_r / count) as u8,
                                    (sum_g / count) as u8,
                                    (sum_b / count) as u8,
                                    255,
                                ]),
                            );
                        }
                    }
                }
            }
            _ => {
                for y in *ry..(*ry + *rh).min(h) {
                    for x in *rx..(*rx + *rw).min(w) {
                        rgba.put_pixel(x, y, Rgba([128, 128, 128, 255]));
                    }
                }
            }
        }
    }

    rgba.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Regions redacted".into() })
}

pub fn add_watermark(
    input_path: String,
    output_path: String,
    text: String,
    opacity: u8,
    position: String,
) -> Result<ToolResult, String> {
    use ab_glyph::{FontRef, PxScale, Font, ScaleFont};

    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let font_data = include_bytes!("../../../src-tauri/fonts/Inter.ttf");
    let font = FontRef::try_from_slice(font_data).map_err(|e| format!("Font load error: {}", e))?;

    let font_size = (w as f32 * 0.04).max(14.0).min(72.0);
    let scale = PxScale::from(font_size);

    let mut text_width = 0.0f32;
    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        text_width += font.as_scaled(scale).h_advance(glyph_id);
    }
    let text_height = font_size * 1.2;

    let (bx, by) = match position.as_str() {
        "top-left" => (16.0, 16.0),
        "top-right" => ((w as f32 - text_width - 16.0).max(0.0), 16.0),
        "bottom-left" => (16.0, (h as f32 - text_height - 16.0).max(0.0)),
        "center" => ((w as f32 - text_width) / 2.0, (h as f32 - text_height) / 2.0),
        _ => ((w as f32 - text_width - 16.0).max(0.0), (h as f32 - text_height - 16.0).max(0.0)),
    };

    let mut cursor_x = bx;
    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(cursor_x, by + font_size * 0.85));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|px, py, coverage| {
                let img_x = bounds.min.x as i32 + px as i32;
                let img_y = bounds.min.y as i32 + py as i32;
                if img_x >= 0 && img_y >= 0 && (img_x as u32) < w && (img_y as u32) < h {
                    let blend = (coverage * opacity as f32 / 255.0).min(1.0);
                    let p = *rgba.get_pixel(img_x as u32, img_y as u32);
                    let r = (p[0] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                    let g = (p[1] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                    let b = (p[2] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                    rgba.put_pixel(img_x as u32, img_y as u32, Rgba([r, g, b, 255]));
                }
            });
        }
        cursor_x += font.as_scaled(scale).h_advance(glyph_id);
    }

    rgba.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: format!("Watermark '{}' rendered at {}", text, position) })
}

pub fn add_image_watermark(
    input_path: String,
    watermark_path: String,
    output_path: String,
    opacity: u8,
    scale: f32,
    position: Option<String>,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let wm = image::open(&watermark_path).map_err(|e| e.to_string())?;
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let wm_rgba = wm.to_rgba8();
    let wm_new_w = (wm_rgba.width() as f32 * scale) as u32;
    let wm_new_h = (wm_rgba.height() as f32 * scale) as u32;
    let wm_resized = wm.resize(wm_new_w, wm_new_h, image::imageops::FilterType::Lanczos3);
    let wm_rgba = wm_resized.to_rgba8();

    let pos = position.as_deref().unwrap_or("bottom-right");
    let margin = 10u32;
    let (ox, oy) = match pos {
        "top-left" => (margin, margin),
        "top-right" => (w.saturating_sub(wm_rgba.width() + margin), margin),
        "bottom-left" => (margin, h.saturating_sub(wm_rgba.height() + margin)),
        "center" => ((w.saturating_sub(wm_rgba.width())) / 2, (h.saturating_sub(wm_rgba.height())) / 2),
        _ => (w.saturating_sub(wm_rgba.width() + margin), h.saturating_sub(wm_rgba.height() + margin)),
    };

    for wy in 0..wm_rgba.height() {
        for wx in 0..wm_rgba.width() {
            let px = ox + wx;
            let py = oy + wy;
            if px < w && py < h {
                let wp = *wm_rgba.get_pixel(wx, wy);
                let blend = (wp[3] as f64 / 255.0) * (opacity as f64 / 255.0);
                if blend > 0.0 {
                    let p = *rgba.get_pixel(px, py);
                    let r = (p[0] as f64 * (1.0 - blend) + wp[0] as f64 * blend) as u8;
                    let g = (p[1] as f64 * (1.0 - blend) + wp[1] as f64 * blend) as u8;
                    let b = (p[2] as f64 * (1.0 - blend) + wp[2] as f64 * blend) as u8;
                    rgba.put_pixel(px, py, Rgba([r, g, b, 255]));
                }
            }
        }
    }

    rgba.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Image watermark added".into() })
}