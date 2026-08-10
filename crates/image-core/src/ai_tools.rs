use image::{GenericImageView, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::types::ToolResult;

fn perceptual_dist(p: Rgba<u8>, bg: [u8; 3]) -> f64 {
    let dr = p[0] as f64 - bg[0] as f64;
    let dg = p[1] as f64 - bg[1] as f64;
    let db = p[2] as f64 - bg[2] as f64;
    (0.299 * dr * dr + 0.587 * dg * dg + 0.114 * db * db).sqrt()
}

pub fn remove_background(input_path: String, output_path: String) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = RgbaImage::new(w, h);

    let border = 8u32;
    let mut bg_r: Vec<u8> = Vec::new();
    let mut bg_g: Vec<u8> = Vec::new();
    let mut bg_b: Vec<u8> = Vec::new();

    for x in 0..w {
        for d in 0..border {
            if d >= h { break; }
            let pt = *rgba.get_pixel(x, d);
            bg_r.push(pt[0]); bg_g.push(pt[1]); bg_b.push(pt[2]);
            let pb = *rgba.get_pixel(x, h - 1 - d);
            bg_r.push(pb[0]); bg_g.push(pb[1]); bg_b.push(pb[2]);
        }
    }
    for y in 0..h {
        for d in 0..border {
            if d >= w { break; }
            let pl = *rgba.get_pixel(d, y);
            bg_r.push(pl[0]); bg_g.push(pl[1]); bg_b.push(pl[2]);
            let pr = *rgba.get_pixel(w - 1 - d, y);
            bg_r.push(pr[0]); bg_g.push(pr[1]); bg_b.push(pr[2]);
        }
    }

    bg_r.sort_unstable();
    bg_g.sort_unstable();
    bg_b.sort_unstable();
    let mid = bg_r.len() / 2;
    let bg_color = [bg_r[mid], bg_g[mid], bg_b[mid]];

    let total_px = (w * h) as usize;
    let mut dist_map: Vec<f64> = Vec::with_capacity(total_px);
    let mut max_dist: f64 = 0.0;

    for y in 0..h {
        for x in 0..w {
            let p = *rgba.get_pixel(x, y);
            let d = perceptual_dist(p, bg_color);
            dist_map.push(d);
            if d > max_dist { max_dist = d; }
        }
    }
    if max_dist < 1.0 { max_dist = 1.0; }

    let mut sorted_dists = dist_map.clone();
    sorted_dists.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let bg_pct = 0.60;
    let thresh_idx = (total_px as f64 * bg_pct).min(total_px as f64 - 1.0) as usize;
    let percentile_thresh = sorted_dists[thresh_idx];

    let bins = 256usize;
    let mut hist = vec![0u32; bins];
    for &d in &dist_map {
        let bin = ((d / max_dist) * (bins as f64 - 1.0)) as usize;
        hist[bin] += 1;
    }
    let total_f = total_px as f64;
    let mut sum_all: f64 = 0.0;
    for i in 0..bins { sum_all += i as f64 * hist[i] as f64; }
    let mut sum_bg: f64 = 0.0;
    let mut w_bg: f64 = 0.0;
    let mut best_otsu = 0usize;
    let mut best_var: f64 = 0.0;
    for i in 0..bins {
        w_bg += hist[i] as f64;
        if w_bg < 1.0 { continue; }
        let w_fg = total_f - w_bg;
        if w_fg < 1.0 { break; }
        sum_bg += i as f64 * hist[i] as f64;
        let mean_bg = sum_bg / w_bg;
        let mean_fg = (sum_all - sum_bg) / w_fg;
        let variance = w_bg * w_fg * (mean_bg - mean_fg).powi(2);
        if variance > best_var {
            best_var = variance;
            best_otsu = i;
        }
    }
    let otsu_thresh = best_otsu as f64 / (bins as f64 - 1.0) * max_dist;

    let threshold = percentile_thresh.max(otsu_thresh).max(max_dist * 0.02);
    let transition_half = max_dist * 0.12;

    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let pixel = *rgba.get_pixel(x, y);
            let d = dist_map[i];

            let alpha = if d >= threshold + transition_half {
                255u8
            } else if d <= threshold - transition_half {
                0u8
            } else {
                let t = ((d - (threshold - transition_half)) / (2.0 * transition_half)).clamp(0.0, 1.0);
                let t = t * t * (3.0 - 2.0 * t);
                (t * 255.0) as u8
            };

            out.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], alpha]));
        }
    }

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let alpha = out.get_pixel(x, y)[3];
            if alpha > 128 {
                let mut transparent = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = (x as i32 + dx) as u32;
                        let ny = (y as i32 + dy) as u32;
                        if out.get_pixel(nx, ny)[3] < 128 { transparent += 1; }
                    }
                }
                if transparent >= 6 {
                    out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
    }

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let alpha = out.get_pixel(x, y)[3];
            if alpha < 128 {
                let mut opaque = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = (x as i32 + dx) as u32;
                        let ny = (y as i32 + dy) as u32;
                        if out.get_pixel(nx, ny)[3] > 128 { opaque += 1; }
                    }
                }
                if opaque >= 6 {
                    let pixel = *rgba.get_pixel(x, y);
                    out.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], 255]));
                }
            }
        }
    }

    out.save(&output_path).map_err(|e| e.to_string())?;

    let fg_pixels = dist_map.iter().filter(|d| **d > threshold + transition_half).count();
    let pct = (fg_pixels as f64 / total_px as f64 * 100.0) as u32;
    let msg = format!("Background removed — {}% foreground (bg: [{},{},{}], thresh: {:.1}, otsu: {:.1})",
        pct, bg_color[0], bg_color[1], bg_color[2], threshold, otsu_thresh);
    Ok(ToolResult { success: true, output_path: Some(output_path), message: msg })
}

pub fn inpaint_image(
    input_path: String,
    output_path: String,
    regions: Vec<(u32, u32, u32, u32)>,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    for (rx, ry, rw, rh) in regions {
        let mut sum_r: u64 = 0;
        let mut sum_g: u64 = 0;
        let mut sum_b: u64 = 0;
        let mut count: u64 = 0;
        let border = 8u32;

        for sy in ry.saturating_sub(border)..(ry + rh + border).min(h) {
            for sx in rx.saturating_sub(border)..(rx + rw + border).min(w) {
                if sx >= rx && sx < rx + rw && sy >= ry && sy < ry + rh { continue; }
                let p = *rgba.get_pixel(sx, sy);
                sum_r += p[0] as u64;
                sum_g += p[1] as u64;
                sum_b += p[2] as u64;
                count += 1;
            }
        }

        if count > 0 {
            let avg = [(sum_r / count) as u8, (sum_g / count) as u8, (sum_b / count) as u8];
            for y in ry..(ry + rh).min(h) {
                for x in rx..(rx + rw).min(w) {
                    let dx_edge = (x as i32 - rx as i32).min((rx + rw) as i32 - x as i32 - 1) as f64;
                    let dy_edge = (y as i32 - ry as i32).min((ry + rh) as i32 - y as i32 - 1) as f64;
                    let edge_dist = dx_edge.min(dy_edge).min(border as f64);
                    let blend = (edge_dist / border as f64).min(1.0);
                    let p = *rgba.get_pixel(x, y);
                    let nr = (p[0] as f64 * (1.0 - blend) + avg[0] as f64 * blend) as u8;
                    let ng = (p[1] as f64 * (1.0 - blend) + avg[1] as f64 * blend) as u8;
                    let nb = (p[2] as f64 * (1.0 - blend) + avg[2] as f64 * blend) as u8;
                    rgba.put_pixel(x, y, Rgba([nr, ng, nb, 255]));
                }
            }
        }
    }

    rgba.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Inpainting completed".into() })
}

pub fn upscale_image(input_path: String, output_path: String, scale: u32) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let (w, h) = img.dimensions();
    let new_w = w * scale;
    let new_h = h * scale;
    let upscaled = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
    let sharpened = upscaled.unsharpen(1.0, 1);
    sharpened.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: format!("Upscaled {}x ({}x{} -> {}x{})", scale, w, h, new_w, new_h) })
}

pub fn sepia_filter(input_path: String, output_path: String) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let p = *rgba.get_pixel(x, y);
            let r = p[0] as f64;
            let g = p[1] as f64;
            let b = p[2] as f64;
            let sr = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0) as u8;
            let sg = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0) as u8;
            let sb = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0) as u8;
            out.put_pixel(x, y, Rgba([sr, sg, sb, 255]));
        }
    }

    out.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Sepia filter applied".into() })
}

pub fn smart_sharpen(input_path: String, output_path: String, strength: f32) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let blurred = img.blur((strength * 0.3) as f32);
    let blur_rgba = blurred.to_rgba8();
    let mut out = RgbaImage::new(w, h);

    let amount = (strength as f64).clamp(0.1, 5.0);

    for y in 0..h {
        for x in 0..w {
            let orig = *rgba.get_pixel(x, y);
            let blur_p = *blur_rgba.get_pixel(x, y);
            let orig_lum = 0.299 * orig[0] as f64 + 0.587 * orig[1] as f64 + 0.114 * orig[2] as f64;
            let blur_lum = 0.299 * blur_p[0] as f64 + 0.587 * blur_p[1] as f64 + 0.114 * blur_p[2] as f64;
            let detail = orig_lum - blur_lum;
            let edge_strength = (detail.abs() / 50.0).min(1.0);
            let sharpen = amount * (0.5 + 0.5 * edge_strength);

            let r = (orig[0] as f64 + detail * sharpen).clamp(0.0, 255.0) as u8;
            let g = (orig[1] as f64 + detail * sharpen).clamp(0.0, 255.0) as u8;
            let b = (orig[2] as f64 + detail * sharpen).clamp(0.0, 255.0) as u8;
            out.put_pixel(x, y, Rgba([r, g, b, orig[3]]));
        }
    }

    out.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: format!("Smart sharpen applied (strength: {})", strength) })
}

pub fn depth_blur(input_path: String, output_path: String, blur_strength: f32) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let (w, h) = img.dimensions();
    let blurred = img.blur(blur_strength as f32);
    let mut out = img.to_rgba8();
    let blurred_rgba = blurred.to_rgba8();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let blend = (dist * 1.5).min(1.0);
            let sharp = *out.get_pixel(x, y);
            let blur_p = *blurred_rgba.get_pixel(x, y);
            let r = (sharp[0] as f64 * (1.0 - blend) + blur_p[0] as f64 * blend) as u8;
            let g = (sharp[1] as f64 * (1.0 - blend) + blur_p[1] as f64 * blend) as u8;
            let b = (sharp[2] as f64 * (1.0 - blend) + blur_p[2] as f64 * blend) as u8;
            out.put_pixel(x, y, Rgba([r, g, b, sharp[3]]));
        }
    }

    out.save(&output_path).map_err(|e| e.to_string())?;
    Ok(ToolResult { success: true, output_path: Some(output_path), message: "Depth blur applied".into() })
}