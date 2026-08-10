use image_core::{
    add_image_watermark, add_watermark, compress_image, convert_format, convert_heic,
    depth_blur, expand_canvas, flip_image, grayscale, inpaint_image, process_image,
    raster_to_svg, redact_regions, remove_background, rotate_image, sharpen_image,
    sepia_filter, smart_crop, smart_sharpen, split_image, stitch_images, strip_metadata,
    upscale_image, blur_image, resize_image, ToolResult,
};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
struct WasmResult {
    success: bool,
    output_path: Option<String>,
    message: String,
}

impl From<ToolResult> for WasmResult {
    fn from(r: ToolResult) -> Self {
        WasmResult {
            success: r.success,
            output_path: r.output_path,
            message: r.message,
        }
    }
}

#[wasm_bindgen]
pub fn wasm_compress_image(input_path: String, output_path: String, quality: u8) -> String {
    match compress_image(input_path, output_path, quality) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_process_image(input_path: String, output_path: String, operation: String, params: Option<String>) -> String {
    match process_image(input_path, output_path, operation, params) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_remove_background(input_path: String, output_path: String) -> String {
    match remove_background(input_path, output_path) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_inpaint_image(input_path: String, output_path: String, regions_json: String) -> String {
    let regions: Vec<(u32, u32, u32, u32)> = serde_json::from_str(&regions_json).unwrap_or_default();
    match inpaint_image(input_path, output_path, regions) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_upscale_image(input_path: String, output_path: String, scale: u32) -> String {
    match upscale_image(input_path, output_path, scale) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_sepia_filter(input_path: String, output_path: String) -> String {
    match sepia_filter(input_path, output_path) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_smart_sharpen(input_path: String, output_path: String, strength: f32) -> String {
    match smart_sharpen(input_path, output_path, strength) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_depth_blur(input_path: String, output_path: String, blur_strength: f32) -> String {
    match depth_blur(input_path, output_path, blur_strength) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_strip_metadata(input_path: String, output_path: String) -> String {
    match strip_metadata(input_path, output_path) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_redact_regions(input_path: String, output_path: String, regions_json: String, method: String) -> String {
    let regions: Vec<(u32, u32, u32, u32)> = serde_json::from_str(&regions_json).unwrap_or_default();
    match redact_regions(input_path, output_path, regions, method) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_add_watermark(input_path: String, output_path: String, text: String, opacity: u8, position: String) -> String {
    match add_watermark(input_path, output_path, text, opacity, position) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_smart_crop(input_path: String, output_path: String, width: u32, height: u32, gravity: String) -> String {
    match smart_crop(input_path, output_path, width, height, gravity) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_expand_canvas(input_path: String, output_path: String, top: u32, bottom: u32, left: u32, right: u32, color: String) -> String {
    match expand_canvas(input_path, output_path, top, bottom, left, right, color) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_split_image(input_path: String, output_dir: String, rows: u32, cols: u32) -> String {
    match split_image(input_path, output_dir, rows, cols) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_stitch_images(paths_json: String, output_path: String, direction: String) -> String {
    let paths: Vec<String> = serde_json::from_str(&paths_json).unwrap_or_default();
    match stitch_images(paths, output_path, direction) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_convert_format(input_path: String, output_path: String) -> String {
    match convert_format(input_path, output_path) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_rotate_image(input_path: String, output_path: String, degrees: u32) -> String {
    match rotate_image(input_path, output_path, degrees) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_grayscale(input_path: String, output_path: String) -> String {
    match grayscale(input_path, output_path) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_blur_image(input_path: String, output_path: String, sigma: f32) -> String {
    match blur_image(input_path, output_path, sigma) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_sharpen_image(input_path: String, output_path: String, amount: f32, radius: u32) -> String {
    match sharpen_image(input_path, output_path, amount, radius) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_flip_image(input_path: String, output_path: String, direction: String) -> String {
    match flip_image(input_path, output_path, direction) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}

#[wasm_bindgen]
pub fn wasm_resize_image(input_path: String, output_path: String, max_w: u32, max_h: u32) -> String {
    match resize_image(input_path, output_path, max_w, max_h) {
        Ok(r) => serde_json::to_string(&WasmResult::from(r)).unwrap(),
        Err(e) => serde_json::to_string(&WasmResult { success: false, output_path: None, message: e }).unwrap(),
    }
}