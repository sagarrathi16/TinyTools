use serde::{Deserialize, Serialize};

use crate::types::ToolResult;

pub fn process_image(
    input_path: String,
    output_path: String,
    operation: String,
    params: Option<String>,
) -> Result<ToolResult, String> {
    let img = image::open(&input_path).map_err(|e| e.to_string())?;
    let parsed: serde_json::Value = params
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(serde_json::Value::Null);

    match operation.as_str() {
        "resize" => {
            let max_w = parsed["width"].as_u64().unwrap_or(800) as u32;
            let max_h = parsed["height"].as_u64().unwrap_or(800) as u32;
            let ratio = (max_w as f64 / img.width() as f64).min(max_h as f64 / img.height() as f64).min(1.0);
            let new_w = (img.width() as f64 * ratio) as u32;
            let new_h = (img.height() as f64 * ratio) as u32;
            let resized = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
            resized.save(&output_path).map_err(|e| e.to_string())?;
        }
        "grayscale" => {
            let gray = img.grayscale();
            gray.save(&output_path).map_err(|e| e.to_string())?;
        }
        "rotate" => {
            let degrees = parsed["degrees"].as_u64().unwrap_or(90);
            let rotated = match degrees % 360 {
                90 => img.rotate90(),
                180 => img.rotate180(),
                270 => img.rotate270(),
                _ => img,
            };
            rotated.save(&output_path).map_err(|e| e.to_string())?;
        }
        "flip" => {
            let direction = parsed["direction"].as_str().unwrap_or("horizontal");
            let flipped = if direction == "vertical" { img.flipv() } else { img.fliph() };
            flipped.save(&output_path).map_err(|e| e.to_string())?;
        }
        "blur" => {
            let sigma = parsed["sigma"].as_f64().unwrap_or(3.0) as f32;
            let blurred = img.blur(sigma as f32);
            blurred.save(&output_path).map_err(|e| e.to_string())?;
        }
        "sharpen" => {
            let amount = parsed["amount"].as_f64().unwrap_or(1.0) as f32;
            let radius = parsed["radius"].as_i64().unwrap_or(1) as u32;
            let sharpened = img.unsharpen(amount as f32, radius as i32);
            sharpened.save(&output_path).map_err(|e| e.to_string())?;
        }
        _ => {
            return Ok(ToolResult {
                success: false,
                output_path: None,
                message: format!("Unknown operation: {}", operation),
            });
        }
    }

    Ok(ToolResult {
        success: true,
        output_path: Some(output_path),
        message: format!("Operation '{}' completed successfully", operation),
    })
}