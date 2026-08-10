import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

export interface FileInfo {
  name: string;
  path: string;
  size: number;
  extension: string;
}

async function pathToFileInfo(p: string): Promise<FileInfo> {
  const name = p.split(/[\\/]/).pop() ?? "";
  const ext = name.includes(".") ? name.split(".").pop()! : "";
  let size = 0;
  try {
    const metadata = await invoke<{ size?: number }>("get_file_info", { path: p });
    size = metadata?.size ?? 0;
  } catch { /* ignore */ }
  return { name, path: p, size, extension: ext };
}

export async function pickFile(filters?: { name: string; extensions: string[] }[]): Promise<FileInfo | null> {
  const selected = await open({ multiple: false, filters });
  if (!selected) return null;
  const p = typeof selected === "string" ? selected : selected;
  return pathToFileInfo(p as string);
}

export async function pickFiles(filters?: { name: string; extensions: string[] }[]): Promise<FileInfo[]> {
  const selected = await open({ multiple: true, filters });
  if (!selected) return [];
  const paths = Array.isArray(selected) ? selected : [selected];
  return Promise.all(paths.map(p => pathToFileInfo(p as string)));
}

export async function pickDirectory(): Promise<string | null> {
  const selected = await open({ directory: true });
  if (!selected) return null;
  return selected as string;
}

export async function saveFile(defaultPath?: string, filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
  return save({ defaultPath, filters });
}

export interface ToolResult {
  success: boolean;
  output_path: string | null;
  message: string;
}

export interface BatchResult {
  success: boolean;
  processed: number;
  failed: number;
  output_dir: string;
  message: string;
}

export async function processImage(input: string, output: string, operation: string, params?: Record<string, unknown>): Promise<ToolResult> {
  return invoke<ToolResult>("process_image", { inputPath: input, outputPath: output, operation, params: params ? JSON.stringify(params) : null });
}
export async function removeBackground(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("remove_background", { inputPath: input, outputPath: output });
}
export async function inpaintImage(input: string, output: string, regions: [number, number, number, number][]): Promise<ToolResult> {
  return invoke<ToolResult>("inpaint_image", { inputPath: input, outputPath: output, regions });
}
export async function upscaleImage(input: string, output: string, scale: number): Promise<ToolResult> {
  return invoke<ToolResult>("upscale_image", { inputPath: input, outputPath: output, scale });
}
export async function sepiaFilter(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("sepia_filter", { inputPath: input, outputPath: output });
}
export async function smartSharpen(input: string, output: string, strength: number): Promise<ToolResult> {
  return invoke<ToolResult>("smart_sharpen", { inputPath: input, outputPath: output, strength });
}
export async function depthBlur(input: string, output: string, blurStrength: number): Promise<ToolResult> {
  return invoke<ToolResult>("depth_blur", { inputPath: input, outputPath: output, blurStrength });
}

// Privacy
export async function stripMetadata(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("strip_metadata", { inputPath: input, outputPath: output });
}
export async function redactRegions(input: string, output: string, regions: [number, number, number, number][], method: string): Promise<ToolResult> {
  return invoke<ToolResult>("redact_regions", { inputPath: input, outputPath: output, regions, method });
}
export async function addWatermark(input: string, output: string, text: string, opacity: number, position: string): Promise<ToolResult> {
  return invoke<ToolResult>("add_watermark", { inputPath: input, outputPath: output, text, opacity, position });
}
export async function addImageWatermark(input: string, watermark: string, output: string, opacity: number, scale: number, position?: string): Promise<ToolResult> {
  return invoke<ToolResult>("add_image_watermark", { inputPath: input, watermarkPath: watermark, outputPath: output, opacity, scale, position: position ?? null });
}

// Editing
export async function smartCrop(input: string, output: string, width: number, height: number, gravity: string): Promise<ToolResult> {
  return invoke<ToolResult>("smart_crop", { inputPath: input, outputPath: output, width, height, gravity });
}
export async function expandCanvas(input: string, output: string, top: number, bottom: number, left: number, right: number, color: string): Promise<ToolResult> {
  return invoke<ToolResult>("expand_canvas", { inputPath: input, outputPath: output, top, bottom, left, right, color });
}
export async function splitImage(input: string, outputDir: string, rows: number, cols: number): Promise<ToolResult> {
  return invoke<ToolResult>("split_image", { inputPath: input, outputDir: outputDir, rows, cols });
}
export async function stitchImages(paths: string[], output: string, direction: string): Promise<ToolResult> {
  return invoke<ToolResult>("stitch_images", { paths, outputPath: output, direction });
}

// Compression & Conversion
export async function smartCompress(input: string, output: string, quality: number, targetSizeKb?: number): Promise<ToolResult> {
  return invoke<ToolResult>("smart_compress", { inputPath: input, outputPath: output, quality, targetSizeKb: targetSizeKb ?? null });
}
export async function convertFormat(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("convert_format", { inputPath: input, outputPath: output });
}
export async function convertHeic(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("convert_heic", { inputPath: input, outputPath: output });
}
export async function rasterToSvg(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("raster_to_svg", { inputPath: input, outputPath: output });
}

// Batch
export async function batchCompress(paths: string[], outputDir: string, quality: number, targetSizeKb?: number): Promise<BatchResult> {
  return invoke<BatchResult>("batch_compress", { inputPaths: paths, outputDir, quality, targetSizeKb: targetSizeKb ?? null });
}
export async function batchResize(paths: string[], outputDir: string, width: number, height: number): Promise<BatchResult> {
  return invoke<BatchResult>("batch_resize", { inputPaths: paths, outputDir, width, height });
}
export async function batchConvert(paths: string[], outputDir: string, targetFormat: string): Promise<BatchResult> {
  return invoke<BatchResult>("batch_convert", { inputPaths: paths, outputDir, targetFormat });
}
export async function batchWatermark(paths: string[], outputDir: string, text: string, opacity: number): Promise<BatchResult> {
  return invoke<BatchResult>("batch_watermark", { inputPaths: paths, outputDir, text, opacity });
}

// PDF Tools
export async function getPdfInfo(input: string): Promise<ToolResult> {
  return invoke<ToolResult>("get_pdf_info", { inputPath: input });
}
export async function mergePdfs(inputs: string[], output: string): Promise<ToolResult> {
  return invoke<ToolResult>("merge_pdfs", { inputPaths: inputs, outputPath: output });
}
export async function splitPdf(input: string, outputDir: string, pages?: string): Promise<ToolResult> {
  return invoke<ToolResult>("split_pdf", { inputPath: input, outputDir, pages: pages ?? null });
}
export async function reorderPages(input: string, output: string, newOrder: number[]): Promise<ToolResult> {
  return invoke<ToolResult>("reorder_pages", { inputPath: input, outputPath: output, newOrder });
}
export async function rotatePages(input: string, output: string, pages?: string, angle: number = 90): Promise<ToolResult> {
  return invoke<ToolResult>("rotate_pages", { inputPath: input, outputPath: output, pages: pages ?? null, angle });
}
export async function cropPages(input: string, output: string, pages?: string, top: number = 0, bottom: number = 0, left: number = 0, right: number = 0): Promise<ToolResult> {
  return invoke<ToolResult>("crop_pages", { inputPath: input, outputPath: output, pages: pages ?? null, top, bottom, left, right });
}
export async function deletePages(input: string, output: string, pagesToDelete: number[]): Promise<ToolResult> {
  return invoke<ToolResult>("delete_pages", { inputPath: input, outputPath: output, pagesToDelete });
}
export async function imagesToPdf(inputs: string[], output: string, margin: number = 20): Promise<ToolResult> {
  return invoke<ToolResult>("images_to_pdf", { inputPaths: inputs, outputPath: output, margin });
}
export async function extractPdfText(input: string): Promise<ToolResult> {
  return invoke<ToolResult>("extract_text", { inputPath: input });
}
export async function encryptPdf(input: string, output: string, userPassword: string, ownerPassword: string): Promise<ToolResult> {
  return invoke<ToolResult>("encrypt_pdf", { inputPath: input, outputPath: output, userPassword, ownerPassword });
}
export async function decryptPdf(input: string, output: string, password: string): Promise<ToolResult> {
  return invoke<ToolResult>("decrypt_pdf", { inputPath: input, outputPath: output, password });
}
export async function unwrapPdf(input: string, output: string, password: string): Promise<ToolResult> {
  return invoke<ToolResult>("unwrap_pdf", { inputPath: input, outputPath: output, password });
}
export async function compressPdf(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("compress_pdf", { inputPath: input, outputPath: output });
}
export async function flattenPdf(input: string, output: string): Promise<ToolResult> {
  return invoke<ToolResult>("flatten_pdf", { inputPath: input, outputPath: output });
}
export async function addPdfWatermark(input: string, output: string, text: string, fontSize: number, opacity: number, angle: number): Promise<ToolResult> {
  return invoke<ToolResult>("add_pdf_watermark", { inputPath: input, outputPath: output, text, fontSize, opacity, angle });
}
export async function addPageNumbers(input: string, output: string, fontSize: number = 12, position: string = "bottom-center"): Promise<ToolResult> {
  return invoke<ToolResult>("add_page_numbers", { inputPath: input, outputPath: output, fontSize, position });
}

// Password Generator
export interface PasswordRequest {
  mode: string;
  length?: number;
  word_count?: number;
  count?: number;
  uppercase?: boolean;
  lowercase?: boolean;
  digits?: boolean;
  symbols?: boolean;
  exclude_ambiguous?: boolean;
  custom_symbols?: string;
  separator?: string;
  pattern?: string;
  capitalize?: boolean;
  append_digit?: boolean;
  append_symbol?: boolean;
  syllable_separator?: string;
}

export interface GeneratedPassword {
  password: string;
  entropy_bits: number;
  strength_label: string;
  charset_size: number;
  length: number;
}

export interface BulkPasswordResult {
  passwords: GeneratedPassword[];
  count: number;
  exported_path: string | null;
}

export async function generatePassword(req: PasswordRequest): Promise<GeneratedPassword> {
  return invoke<GeneratedPassword>("generate_password", { req });
}

export async function generateBulkPasswords(req: PasswordRequest): Promise<BulkPasswordResult> {
  return invoke<BulkPasswordResult>("generate_bulk", { req });
}

export async function exportPasswords(passwords: string[], format: string, outputPath: string): Promise<string> {
  return invoke<string>("export_passwords", { passwords, format, outputPath });
}

// ── Encoder/Decoder ────────────────────────────────────────────
export async function encodeBase64(input: string): Promise<string> {
  return invoke<string>("encode_base64", { input });
}
export async function decodeBase64(input: string): Promise<string> {
  return invoke<string>("decode_base64", { input });
}
export async function encodeBase64Url(input: string): Promise<string> {
  return invoke<string>("encode_base64url", { input });
}
export async function decodeBase64Url(input: string): Promise<string> {
  return invoke<string>("decode_base64url", { input });
}
export async function encodeBase32(input: string): Promise<string> {
  return invoke<string>("encode_base32", { input });
}
export async function decodeBase32(input: string): Promise<string> {
  return invoke<string>("decode_base32", { input });
}
export async function encodeBase58(input: string): Promise<string> {
  return invoke<string>("encode_base58", { input });
}
export async function decodeBase58(input: string): Promise<string> {
  return invoke<string>("decode_base58", { input });
}
export async function encodeHex(input: string): Promise<string> {
  return invoke<string>("encode_hex", { input });
}
export async function decodeHex(input: string): Promise<string> {
  return invoke<string>("decode_hex", { input });
}
export async function encodeUrl(input: string): Promise<string> {
  return invoke<string>("encode_url", { input });
}
export async function decodeUrl(input: string): Promise<string> {
  return invoke<string>("decode_url", { input });
}
export async function encodeHtml(input: string): Promise<string> {
  return invoke<string>("encode_html", { input });
}
export async function decodeHtml(input: string): Promise<string> {
  return invoke<string>("decode_html", { input });
}
export async function encodeUnicode(input: string): Promise<string> {
  return invoke<string>("encode_unicode", { input });
}
export async function decodeUnicode(input: string): Promise<string> {
  return invoke<string>("decode_unicode", { input });
}

export interface JwtParts {
  header: string;
  payload: string;
  signature: string;
  valid_json: boolean;
}
export async function decodeJwt(token: string): Promise<JwtParts> {
  return invoke<JwtParts>("decode_jwt", { token });
}

export async function textToMorse(input: string): Promise<string> {
  return invoke<string>("text_to_morse", { input });
}
export async function morseToText(input: string): Promise<string> {
  return invoke<string>("morse_to_text", { input });
}
export async function textToBinary(input: string): Promise<string> {
  return invoke<string>("text_to_binary", { input });
}
export async function binaryToText(input: string): Promise<string> {
  return invoke<string>("binary_to_text", { input });
}
export async function textToOctal(input: string): Promise<string> {
  return invoke<string>("text_to_octal", { input });
}
export async function octalToText(input: string): Promise<string> {
  return invoke<string>("octal_to_text", { input });
}

export async function encodeFile(inputPath: string, encoding: string): Promise<string> {
  return invoke<string>("encode_file", { inputPath, encoding });
}

export async function decodeFile(inputPath: string, outputPath: string, encoding: string): Promise<string> {
  return invoke<string>("decode_file", { inputPath, outputPath, encoding });
}

export async function decodeTextToFile(input: string, outputPath: string, encoding: string): Promise<string> {
  return invoke<string>("decode_text_to_file", { input, outputPath, encoding });
}

// ── Hasher ─────────────────────────────────────────────────────
export async function hashText(input: string, algorithm: string): Promise<string> {
  return invoke<string>("hash_text", { input, algorithm });
}

export interface HashResult {
  algorithm: string;
  hash: string;
  file_size: number;
}
export async function hashFile(inputPath: string, algorithm: string): Promise<HashResult> {
  return invoke<HashResult>("hash_file", { inputPath, algorithm });
}

export interface MultiHashResult {
  md5: string;
  sha1: string;
  sha256: string;
  sha512: string;
  blake3: string;
  crc32: string;
  file_size: number;
}
export async function hashFileAll(inputPath: string): Promise<MultiHashResult> {
  return invoke<MultiHashResult>("hash_file_all", { inputPath });
}
export async function hashTextAll(input: string): Promise<MultiHashResult> {
  return invoke<MultiHashResult>("hash_text_all", { input });
}

export interface VerifyResult {
  matches: boolean;
  computed: string;
  expected: string;
}
export async function verifyFileHash(inputPath: string, algorithm: string, expectedHash: string): Promise<VerifyResult> {
  return invoke<VerifyResult>("verify_file_hash", { inputPath, algorithm, expectedHash });
}
export async function verifyTextHash(input: string, algorithm: string, expectedHash: string): Promise<VerifyResult> {
  return invoke<VerifyResult>("verify_text_hash", { input, algorithm, expectedHash });
}

// ── Encryption ─────────────────────────────────────────────────
export async function encryptTextAes(input: string, passphrase: string, kdf: string = "argon2"): Promise<string> {
  return invoke<string>("encrypt_text_aes", { input, passphrase, kdf });
}
export async function decryptTextAes(input: string, passphrase: string): Promise<string> {
  return invoke<string>("decrypt_text_aes", { input, passphrase });
}
export async function encryptTextChacha(input: string, passphrase: string, kdf: string = "argon2"): Promise<string> {
  return invoke<string>("encrypt_text_chacha", { input, passphrase, kdf });
}
export async function decryptTextChacha(input: string, passphrase: string): Promise<string> {
  return invoke<string>("decrypt_text_chacha", { input, passphrase });
}
export async function encryptRot13(input: string): Promise<string> {
  return invoke<string>("encrypt_rot13", { input });
}
export async function encryptCaesar(input: string, shift: number): Promise<string> {
  return invoke<string>("encrypt_caesar", { input, shift });
}
export async function encryptVigenere(input: string, key: string): Promise<string> {
  return invoke<string>("encrypt_vigenere", { input, key });
}
export async function encryptXor(input: string, key: string, encoding: string = "raw"): Promise<string> {
  return invoke<string>("encrypt_xor", { input, key, encoding });
}
export async function decryptXor(input: string, key: string, encoding: string = "raw"): Promise<string> {
  return invoke<string>("decrypt_xor", { input, key, encoding });
}
export async function encryptFileAes(inputPath: string, outputPath: string, passphrase: string, kdf: string = "argon2"): Promise<string> {
  return invoke<string>("encrypt_file_aes", { inputPath, outputPath, passphrase, kdf });
}
export async function decryptFileAes(inputPath: string, outputPath: string, passphrase: string): Promise<string> {
  return invoke<string>("decrypt_file_aes", { inputPath, outputPath, passphrase });
}
export async function encryptFileChacha(inputPath: string, outputPath: string, passphrase: string, kdf: string = "argon2"): Promise<string> {
  return invoke<string>("encrypt_file_chacha", { inputPath, outputPath, passphrase, kdf });
}
export async function decryptFileChacha(inputPath: string, outputPath: string, passphrase: string): Promise<string> {
  return invoke<string>("decrypt_file_chacha", { inputPath, outputPath, passphrase });
}

// ── Video Tools ────────────────────────────────────────────────
export interface VideoInfo {
  duration: number;
  width: number;
  height: number;
  codec: string;
  audio_codec: string;
  bitrate: number;
  fps: number;
  file_size: number;
  format: string;
}

export async function getVideoInfo(input: string): Promise<ToolResult> {
  return invoke<ToolResult>("get_video_info", { input });
}
export async function compressVideo(input: string, quality: number, targetSizeKb?: number): Promise<ToolResult> {
  return invoke<ToolResult>("compress_video", { input, quality, targetSizeKb: targetSizeKb ?? null });
}
export async function resizeVideo(input: string, width: number, height: number): Promise<ToolResult> {
  return invoke<ToolResult>("resize_video", { input, width, height });
}
export async function convertAspectRatio(input: string, target: string): Promise<ToolResult> {
  return invoke<ToolResult>("convert_aspect_ratio", { input, target });
}
export async function trimVideo(input: string, start: number, end: number): Promise<ToolResult> {
  return invoke<ToolResult>("trim_video", { input, start, end });
}
export async function mergeVideos(inputs: string[], outputDir: string): Promise<ToolResult> {
  return invoke<ToolResult>("merge_videos", { inputs, outputDir });
}
export async function cropVideo(input: string, x: number, y: number, width: number, height: number): Promise<ToolResult> {
  return invoke<ToolResult>("crop_video", { input, x, y, width, height });
}
export async function rotateVideo(input: string, angle: number): Promise<ToolResult> {
  return invoke<ToolResult>("rotate_video", { input, angle });
}
export async function mirrorVideo(input: string, direction: string): Promise<ToolResult> {
  return invoke<ToolResult>("mirror_video", { input, direction });
}
export async function convertVideoFormat(input: string, format: string): Promise<ToolResult> {
  return invoke<ToolResult>("convert_video_format", { input, format });
}
export async function extractAudio(input: string, format: string): Promise<ToolResult> {
  return invoke<ToolResult>("extract_audio", { input, format });
}
export async function muteVideo(input: string): Promise<ToolResult> {
  return invoke<ToolResult>("mute_video", { input });
}
export async function replaceAudio(video: string, audio: string): Promise<ToolResult> {
  return invoke<ToolResult>("replace_audio", { video, audio });
}
export async function videoToGif(input: string, fps: number, width: number): Promise<ToolResult> {
  return invoke<ToolResult>("video_to_gif", { input, fps, width });
}
export async function gifToVideo(input: string): Promise<ToolResult> {
  return invoke<ToolResult>("gif_to_video", { input });
}
export async function changeSpeed(input: string, speed: number): Promise<ToolResult> {
  return invoke<ToolResult>("change_speed", { input, speed });
}
export async function addVideoWatermark(input: string, text: string, position: string, fontSize: number): Promise<ToolResult> {
  return invoke<ToolResult>("add_video_watermark", { input, text, position, fontSize });
}
export async function burnSubtitles(input: string, subtitlePath: string): Promise<ToolResult> {
  return invoke<ToolResult>("burn_subtitles", { input, subtitlePath });
}
export async function extractFrames(input: string, outputDir: string, timestamp?: number): Promise<ToolResult> {
  return invoke<ToolResult>("extract_frames", { input, outputDir, timestamp: timestamp ?? null });
}

// -- Tauri detection --
export function isTauri(): boolean {
  const w = window as any;
  // __TAURI__ is only injected when withGlobalTauri=true; __TAURI_INTERNALS__
  // (the IPC bridge) is always present inside the Tauri webview.
  return typeof window !== 'undefined' && !!(
    w.__TAURI__ ||
    w.__TAURI_INTERNALS__ ||
    (typeof import.meta !== 'undefined' && import.meta.env?.TAURI_TARGET_TRIPLE)
  );
}

// -- WASM fallback --
let wasmModule: any = null;

async function loadWasm(): Promise<any> {
  if (wasmModule) return wasmModule;
  try {
    // @ts-ignore - WASM module built separately with wasm-pack
    wasmModule = await import('./wasm');
    return wasmModule;
  } catch (e) {
    console.warn('WASM module not available:', e);
    return null;
  }
}

export async function wasmProcessImage(
  input: string,
  output: string,
  operation: string,
  params?: Record<string, unknown>,
): Promise<ToolResult> {
  const wasm = await loadWasm();
  if (!wasm) throw new Error('WASM module not available');
  const paramsJson = params ? JSON.stringify(params) : null;
  const result = await wasm.wasm_process_image(input, output, operation, paramsJson);
  return JSON.parse(result) as ToolResult;
}

export async function wasmRemoveBackground(input: string, output: string): Promise<ToolResult> {
  const wasm = await loadWasm();
  if (!wasm) throw new Error('WASM module not available');
  const result = await wasm.wasm_remove_background(input, output);
  return JSON.parse(result) as ToolResult;
}

export async function wasmStripMetadata(input: string, output: string): Promise<ToolResult> {
  const wasm = await loadWasm();
  if (!wasm) throw new Error('WASM module not available');
  const result = await wasm.wasm_strip_metadata(input, output);
  return JSON.parse(result) as ToolResult;
}

export async function wasmRedactRegions(
  input: string,
  output: string,
  regions: [number, number, number, number][],
  method: string,
): Promise<ToolResult> {
  const wasm = await loadWasm();
  if (!wasm) throw new Error('WASM module not available');
  const regionsJson = JSON.stringify(regions);
  const result = await wasm.wasm_redact_regions(input, output, regionsJson, method);
  return JSON.parse(result) as ToolResult;
}

export async function wasmAddWatermark(
  input: string,
  output: string,
  text: string,
  opacity: number,
  position: string,
): Promise<ToolResult> {
  const wasm = await loadWasm();
  if (!wasm) throw new Error('WASM module not available');
  const result = await wasm.wasm_add_watermark(input, output, text, opacity, position);
  return JSON.parse(result) as ToolResult;
}
// -- Metadata --
export interface MetadataEntry {
  tag: string;
  value: string;
}
