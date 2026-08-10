use axum::body::Body;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use tokio_stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use crate::p2p::{encryption, get_incoming_transfers, now_secs, IncomingTransfer};
use std::path::PathBuf;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tower_http::services::ServeDir;

fn install_tls_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[derive(Clone)]
pub struct ServerState {
    pub transfers: Arc<Mutex<HashMap<String, PortalTransfer>>>,
    pub downloads_dir: Arc<Mutex<String>>,
    pub download_limits: Arc<Mutex<HashMap<String, DownloadLimit>>>,
    pub receive_password: Option<String>,
    pub receive_only: bool,
}

#[derive(Clone)]
pub struct DownloadLimit {
    pub window_started: std::time::Instant,
    pub attempts: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PortalTransfer {
    #[serde(skip_serializing, skip_deserializing)]
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    #[serde(skip_serializing, skip_deserializing)]
    pub encrypted_data: Option<Vec<u8>>,
    pub encryption_salt: Option<String>,
    pub encryption_nonce: Option<String>,
    pub encryption_iterations: Option<u32>,
    pub started_at: u64,
}

#[derive(Serialize)]
struct PortalInfoResponse {
    file: Option<PortalTransfer>,
    receive_password_required: bool,
}

// ── Download page (existing) ────────────────────────────────────────────

async fn portal_page(State(state): State<ServerState>) -> Html<String> {
    if state.receive_only {
        return Html(RECEIVE_HTML.to_string());
    }
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>TinyTools - File Transfer</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#0f0f0f;color:#fff;min-height:100vh;display:flex;align-items:center;justify-content:center}}
.container{{max-width:420px;width:100%;padding:2rem;text-align:center}}
h1{{font-size:1.5rem;font-weight:600;margin-bottom:.5rem}}
.subtitle{{color:rgba(255,255,255,.4);font-size:.875rem;margin-bottom:2rem}}
.file-icon{{width:80px;height:80px;border-radius:1.25rem;background:rgba(96,165,250,.1);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem}}
.file-icon svg{{width:40px;height:40px;color:rgba(96,165,250,.7)}}
.file-name{{font-size:1rem;font-weight:500;margin-bottom:.25rem;word-break:break-all}}
.file-size{{color:rgba(255,255,255,.4);font-size:.8rem;margin-bottom:1.5rem}}
.password-section{{margin-bottom:1.5rem}}
.password-section input{{width:100%;padding:.75rem 1rem;border-radius:.75rem;border:1px solid rgba(255,255,255,.1);background:rgba(255,255,255,.05);color:#fff;font-size:.875rem;outline:none;transition:border-color .2s}}
.password-section input:focus{{border-color:rgba(96,165,250,.5)}}
.password-section p{{color:rgba(255,255,255,.3);font-size:.75rem;margin-top:.5rem}}
.encrypted-badge{{display:inline-flex;align-items:center;gap:.4rem;padding:.25rem .75rem;border-radius:1rem;border:1px solid rgba(250,204,21,.3);background:rgba(250,204,21,.08);color:#facc15;font-size:.7rem;margin-bottom:1.5rem}}
.encrypted-badge svg{{width:12px;height:12px}}
.btn{{width:100%;padding:.75rem;border-radius:.75rem;border:none;font-size:.875rem;font-weight:500;cursor:pointer;transition:all .2s}}
.btn-primary{{background:rgba(96,165,250,.2);color:#60a5fa;border:1px solid rgba(96,165,250,.3)}}
.btn-primary:hover{{background:rgba(96,165,250,.3)}}
.btn-primary:disabled{{opacity:.4;cursor:not-allowed}}
.btn-download{{background:rgba(74,222,128,.2);color:#4ade80;border:1px solid rgba(74,222,128,.3)}}
.btn-download:hover{{background:rgba(74,222,128,.3)}}
.progress{{display:none;margin-top:1.5rem}}
.progress-bar{{height:6px;background:rgba(255,255,255,.1);border-radius:3px;overflow:hidden;margin-bottom:.5rem}}
.progress-fill{{height:100%;background:#4ade80;border-radius:3px;transition:width .3s;width:0}}
.progress-text{{font-size:.75rem;color:rgba(255,255,255,.4)}}
.error{{color:#f87171;font-size:.8rem;margin-top:1rem;display:none}}
.success{{color:#4ade80;font-size:.8rem;margin-top:1rem;display:none}}
.loading{{color:rgba(255,255,255,.3);font-size:.8rem;margin-top:1rem}}
</style>
</head>
<body>
<div class="container">
<h1>TinyTools</h1>
<p class="subtitle">Secure File Transfer</p>
<div class="file-icon"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg></div>
<div class="file-name" id="fileName">Loading...</div>
<div class="file-size" id="fileSize"></div>
<div class="encrypted-badge" id="encryptedBadge" style="display:none"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg> Encrypted</div>
<div class="password-section" id="passwordSection" style="display:none">
<input type="password" id="passwordInput" placeholder="Enter password to download">
<p>This file is password-protected</p>
</div>
<button class="btn btn-download" id="downloadBtn" onclick="startDownload()">Download File</button>
<div class="progress" id="progressSection">
<div class="progress-bar"><div class="progress-fill" id="progressFill"></div></div>
<div class="progress-text" id="progressText">0%</div>
</div>
<div class="error" id="errorText"></div>
<div class="success" id="successText">Download complete!</div>
<div class="loading" id="loadingIndicator">Loading file info...</div>
</div>
<script>
let fileInfo={{}};
let failedPasswordAttempts=0, passwordBlockedUntil=0;
async function init(){{try{{const r=await fetch("/api/info");if(!r.ok)throw new Error("No file available");const data=await r.json();fileInfo=data.file||data;document.getElementById("loadingIndicator").style.display="none";document.getElementById("fileName").textContent=fileInfo.file_name;document.getElementById("fileSize").textContent=formatSize(fileInfo.file_size);if(fileInfo.encryption_salt){{document.getElementById("passwordSection").style.display="block";document.getElementById("encryptedBadge").style.display="inline-flex"}}else{{document.getElementById("encryptedBadge").style.display="none"}}}}catch(e){{document.getElementById("loadingIndicator").textContent=e.message;document.getElementById("fileName").textContent="No file available";document.getElementById("downloadBtn").disabled=true}}}}
function formatSize(b){{if(b<1024)return b+" B";if(b<1048576)return(b/1024).toFixed(1)+" KB";if(b<1073741824)return(b/1048576).toFixed(1)+" MB";return(b/1073741824).toFixed(2)+" GB"}}
function base64Bytes(value){{const raw=atob(value);return Uint8Array.from(raw,c=>c.charCodeAt(0));}}
async function decryptLocally(ciphertext,password){{if(!window.isSecureContext||!window.crypto?.subtle)throw new Error("This browser requires a secure HTTPS connection for local decryption. The portal must be opened over HTTPS.");const keyMaterial=await crypto.subtle.importKey("raw",new TextEncoder().encode(password),"PBKDF2",false,["deriveKey"]);const key=await crypto.subtle.deriveKey({{name:"PBKDF2",salt:base64Bytes(fileInfo.encryption_salt),iterations:fileInfo.encryption_iterations,hash:"SHA-256"}},keyMaterial,{{name:"AES-GCM",length:256}},false,["decrypt"]);return crypto.subtle.decrypt({{name:"AES-GCM",iv:base64Bytes(fileInfo.encryption_nonce)}},key,ciphertext);}}
async function startDownload(){{const btn=document.getElementById("downloadBtn");if(Date.now()<passwordBlockedUntil){{const error=document.getElementById("errorText");error.textContent="Too many incorrect passwords. Try again in "+Math.ceil((passwordBlockedUntil-Date.now())/1000)+" seconds.";error.style.display="block";return}}btn.disabled=true;btn.textContent="Downloading...";document.getElementById("progressSection").style.display="block";document.getElementById("errorText").style.display="none";document.getElementById("successText").style.display="none";try{{const resp=await fetch("/api/download");if(!resp.ok){{const err=await resp.json().catch(()=>({{error:"Download failed"}}));throw new Error(err.error||resp.statusText||"Download failed")}}const ct=resp.headers.get("content-length")||"0";const total=parseInt(ct);const reader=resp.body.getReader();const chunks=[];let received=0;while(true){{const{{done,value}}=await reader.read();if(done)break;chunks.push(value);received+=value.length;if(total>0){{const pct=Math.round(received/total*100);document.getElementById("progressFill").style.width=pct+"%";document.getElementById("progressText").textContent=received>1024*1024?formatSize(received)+" / "+formatSize(total)+" ("+pct+"%)":pct+"%"}}}}let blob=new Blob(chunks);if(fileInfo.encryption_salt){{try{{const password=document.getElementById("passwordInput").value;if(!password)throw new Error("Enter the password to decrypt this file.");const plaintext=await decryptLocally(await blob.arrayBuffer(),password);blob=new Blob([plaintext])}}catch(e){{failedPasswordAttempts++;if(failedPasswordAttempts>=5){{passwordBlockedUntil=Date.now()+30000;failedPasswordAttempts=0;throw new Error("Too many incorrect passwords. Try again in 30 seconds.")}}throw new Error("Incorrect password ("+(5-failedPasswordAttempts)+" attempts remaining).")}}}}const a=document.createElement("a");a.href=URL.createObjectURL(blob);a.download=fileInfo.file_name;document.body.appendChild(a);a.click();document.body.removeChild(a);URL.revokeObjectURL(a.href);document.getElementById("successText").style.display="block";btn.textContent="Download Complete"}}catch(e){{const error=document.getElementById("errorText");error.textContent=e.message;error.style.display="block";btn.disabled=false;btn.textContent="Download File";document.getElementById("progressSection").style.display="none"}}}}
if(!window.isSecureContext||!window.crypto?.subtle){{
  const error=document.getElementById("errorText");
  error.textContent="Local password decryption requires an HTTPS connection. This HTTP portal cannot decrypt securely in this browser.";
  error.style.display="block";
  document.getElementById("downloadBtn").disabled=true;
}}
init();
</script>
</body></html>"#,
    ))
}

// ── Receive / Upload page (new) ─────────────────────────────────────────

static RECEIVE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>TinyTools - Send File</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#0f0f0f;color:#fff;min-height:100vh;display:flex;align-items:center;justify-content:center}
.container{max-width:420px;width:100%;padding:2rem;text-align:center}
h1{font-size:1.5rem;font-weight:600;margin-bottom:.5rem}
.subtitle{color:rgba(255,255,255,.4);font-size:.875rem;margin-bottom:2rem}
.upload-zone{border:2px dashed rgba(255,255,255,.15);border-radius:1.25rem;padding:2rem 1.5rem;cursor:pointer;transition:border-color .2s;margin-bottom:1.5rem}
.upload-zone:hover{border-color:rgba(96,165,250,.4)}
.upload-zone.dragover{border-color:#60a5fa;background:rgba(96,165,250,.05)}
.upload-zone svg{width:48px;height:48px;color:rgba(255,255,255,.15);margin-bottom:.75rem}
.upload-zone p{font-size:.875rem;color:rgba(255,255,255,.5)}
.upload-zone .selected{font-size:.8rem;color:#60a5fa;margin-top:.5rem;display:none}
.password-section{margin-bottom:1.5rem;display:none}
.password-section input{width:100%;padding:.75rem 1rem;border-radius:.75rem;border:1px solid rgba(255,255,255,.1);background:rgba(255,255,255,.05);color:#fff;font-size:.875rem;outline:none;transition:border-color .2s}
.password-section input:focus{border-color:rgba(96,165,250,.5)}
.password-section p{color:rgba(255,255,255,.3);font-size:.75rem;margin-top:.5rem}
.btn{width:100%;padding:.75rem;border-radius:.75rem;border:none;font-size:.875rem;font-weight:500;cursor:pointer;transition:all .2s}
.btn-upload{background:rgba(96,165,250,.2);color:#60a5fa;border:1px solid rgba(96,165,250,.3)}
.btn-upload:hover{background:rgba(96,165,250,.3)}
.btn-upload:disabled{opacity:.4;cursor:not-allowed}
.progress{display:none;margin-top:1.5rem}
.progress-bar{height:6px;background:rgba(255,255,255,.1);border-radius:3px;overflow:hidden;margin-bottom:.5rem}
.progress-fill{height:100%;background:#60a5fa;border-radius:3px;transition:width .3s;width:0}
.progress-text{font-size:.75rem;color:rgba(255,255,255,.4)}
.status{font-size:.8rem;margin-top:1rem;display:none}
.error{color:#f87171;font-size:.8rem;margin-top:1rem;display:none}
.success{color:#4ade80;font-size:.8rem;margin-top:1rem}
</style>
</head>
<body>
<div class="container">
<h1>TinyTools</h1>
<p class="subtitle">Send a File</p>
<div class="upload-zone" id="uploadZone" onclick="document.getElementById('fileInput').click()">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
<p id="uploadText">Click or drop a file here</p>
<p class="selected" id="selectedFile"></p>
</div>
<input type="file" id="fileInput" style="display:none" onchange="onFileSelect()">
<div class="password-section" id="passwordSection">
<input type="password" id="passwordInput" placeholder="Enter receive password">
<p>This device requires a password to receive files</p>
</div>
<button class="btn btn-upload" id="uploadBtn" onclick="startUpload()" disabled>Select a file</button>
<div class="progress" id="progressSection">
<div class="progress-bar"><div class="progress-fill" id="progressFill"></div></div>
<div class="progress-text" id="progressText">0%</div>
</div>
<div class="status" id="statusText"></div>
<div class="error" id="errorText"></div>
</div>
<script>
let selectedFile=null, transferId=null, pollTimer=null;
async function init(){try{const r=await fetch("/api/info");if(!r.ok)throw new Error();const data=await r.json();if(data.receive_password_required){document.getElementById("passwordSection").style.display="block"}else{document.getElementById("passwordSection").style.display="none"}}catch(e){document.getElementById("passwordSection").style.display="none"}}
function onFileSelect(){const input=document.getElementById("fileInput");if(input.files.length>0){selectedFile=input.files[0];document.getElementById("selectedFile").textContent=selectedFile.name+" ("+formatSize(selectedFile.size)+")";document.getElementById("selectedFile").style.display="block";document.getElementById("uploadText").textContent="File selected";document.getElementById("uploadBtn").disabled=false;document.getElementById("uploadBtn").textContent="Send"}}
function formatSize(b){if(b<1024)return b+" B";if(b<1048576)return(b/1024).toFixed(1)+" KB";if(b<1073741824)return(b/1048576).toFixed(1)+" MB";return(b/1073741824).toFixed(2)+" GB"}

// Phase 1: announce metadata only — no data leaves the browser
function startUpload(){if(!selectedFile)return;const btn=document.getElementById("uploadBtn");btn.disabled=true;btn.textContent="Announcing...";document.getElementById("errorText").style.display="none";document.getElementById("statusText").style.display="none";document.getElementById("progressSection").style.display="block";document.getElementById("progressFill").style.width="0%";document.getElementById("progressText").textContent="Sending file info...";const password=document.getElementById("passwordInput")?.value||"";const xhr=new XMLHttpRequest();xhr.open("POST","/api/announce",true);xhr.setRequestHeader("X-TinyTools-Filename",encodeURIComponent(selectedFile.name));xhr.setRequestHeader("X-TinyTools-FileSize",selectedFile.size.toString());if(password)xhr.setRequestHeader("X-TinyTools-Password",password);xhr.onload=function(){if(xhr.status>=200&&xhr.status<300){try{const data=JSON.parse(xhr.responseText);transferId=data.transfer_id;document.getElementById("progressText").textContent="Waiting for device to accept...";document.getElementById("statusText").textContent="File info sent! Waiting for download confirmation...";document.getElementById("statusText").style.display="block";document.getElementById("statusText").style.color="rgba(255,255,255,.5)";pollTimer=setInterval(pollStatus,2000)}catch(e){document.getElementById("errorText").textContent="Invalid response";document.getElementById("errorText").style.display="block";btn.disabled=false;btn.textContent="Send"}}else{let msg="Failed to send file info";try{const err=JSON.parse(xhr.responseText);msg=err.error||msg}catch(e){}document.getElementById("errorText").textContent=msg;document.getElementById("errorText").style.display="block";btn.disabled=false;btn.textContent="Send";document.getElementById("progressSection").style.display="none"}};xhr.onerror=function(){document.getElementById("errorText").textContent="Network error";document.getElementById("errorText").style.display="block";btn.disabled=false;btn.textContent="Send";document.getElementById("progressSection").style.display="none"};xhr.send()}

async function encryptLocally(plaintext,password){const salt=crypto.getRandomValues(new Uint8Array(16));const nonce=crypto.getRandomValues(new Uint8Array(12));const iterations=310000;const keyMaterial=await crypto.subtle.importKey("raw",new TextEncoder().encode(password),"PBKDF2",false,["deriveKey"]);const key=await crypto.subtle.deriveKey({name:"PBKDF2",salt,iterations,hash:"SHA-256"},keyMaterial,{name:"AES-GCM",length:256},false,["encrypt"]);const ciphertext=await crypto.subtle.encrypt({name:"AES-GCM",iv:nonce},key,plaintext);return{ciphertext,saltBase64:btoa(String.fromCharCode(...salt)),nonceBase64:btoa(String.fromCharCode(...nonce)),iterations}}

// Phase 2: when device accepts, upload the actual file data
async function uploadData(){if(!selectedFile||!transferId)return;const btn=document.getElementById("uploadBtn");btn.disabled=true;btn.textContent="Encrypting...";document.getElementById("progressFill").style.width="0%";document.getElementById("progressText").textContent="0%";try{const password=document.getElementById("passwordInput")?.value||"";let uploadBody;let saltHeader="",nonceHeader="",iterHeader="";if(password){const buffer=await selectedFile.arrayBuffer();const enc=await encryptLocally(buffer,password);uploadBody=new Blob([enc.ciphertext]);saltHeader=enc.saltBase64;nonceHeader=enc.nonceBase64;iterHeader=enc.iterations.toString()}else{uploadBody=selectedFile}btn.textContent="Uploading...";const xhr=new XMLHttpRequest();xhr.open("POST","/api/upload-data/"+transferId,true);xhr.setRequestHeader("Content-Type","application/octet-stream");if(password){xhr.setRequestHeader("X-TinyTools-Salt",saltHeader);xhr.setRequestHeader("X-TinyTools-Nonce",nonceHeader);xhr.setRequestHeader("X-TinyTools-Iterations",iterHeader)}xhr.upload.onprogress=function(e){if(e.lengthComputable){const pct=Math.round(e.loaded/e.total*100);document.getElementById("progressFill").style.width=pct+"%";document.getElementById("progressText").textContent=pct+"% ("+formatSize(e.loaded)+" / "+formatSize(e.total)+")"}};xhr.onload=function(){if(xhr.status>=200&&xhr.status<300){document.getElementById("progressFill").style.width="100%";document.getElementById("progressText").textContent="File delivered!";document.getElementById("statusText").textContent="File delivered!";document.getElementById("statusText").style.color="#4ade80";btn.textContent="Done";btn.disabled=false}else{let msg="Upload failed";try{const err=JSON.parse(xhr.responseText);msg=err.error||msg}catch(e){}document.getElementById("errorText").textContent=msg;document.getElementById("errorText").style.display="block";btn.disabled=false;btn.textContent="Retry"}};xhr.onerror=function(){document.getElementById("errorText").textContent="Network error";document.getElementById("errorText").style.display="block";btn.disabled=false;btn.textContent="Retry"};xhr.send(uploadBody)}catch(e){document.getElementById("errorText").textContent="Encryption error: "+e.message;document.getElementById("errorText").style.display="block";btn.disabled=false;btn.textContent="Retry"}}

async function pollStatus(){if(!transferId)return;try{const resp=await fetch("/api/upload-status/"+transferId);if(!resp.ok){if(resp.status===404){document.getElementById("errorText").textContent="Transfer not found on server (may have been cleared)";document.getElementById("errorText").style.display="block"}return}const data=await resp.json();if(data.status==="ready"){clearInterval(pollTimer);document.getElementById("statusText").textContent="Device accepted! Uploading file...";document.getElementById("statusText").style.color="#60a5fa";uploadData()}else if(data.status==="rejected"){clearInterval(pollTimer);document.getElementById("statusText").textContent="Transfer was rejected";document.getElementById("statusText").style.color="#f87171";document.getElementById("uploadBtn").textContent="Try again";document.getElementById("uploadBtn").disabled=false}else if(data.status==="not_found"){clearInterval(pollTimer);document.getElementById("statusText").textContent="Transfer expired or cancelled";document.getElementById("statusText").style.color="#f87171";document.getElementById("uploadBtn").textContent="Try again";document.getElementById("uploadBtn").disabled=false}}catch(e){document.getElementById("errorText").textContent="Poll error: "+e.message;document.getElementById("errorText").style.display="block"}}
document.addEventListener("dragover",function(e){e.preventDefault();document.getElementById("uploadZone").classList.add("dragover")});
document.addEventListener("dragleave",function(e){e.preventDefault();document.getElementById("uploadZone").classList.remove("dragover")});
document.addEventListener("drop",function(e){e.preventDefault();document.getElementById("uploadZone").classList.remove("dragover");const files=e.dataTransfer.files;if(files.length>0){const input=document.getElementById("fileInput");const dt=new DataTransfer();dt.items.add(files[0]);input.files=dt.files;onFileSelect()}});
init();
</script>
</body></html>"##;

async fn receive_page() -> Html<String> {
    Html(RECEIVE_HTML.to_string())
}

// ── API: Combined portal info ───────────────────────────────────────────

async fn get_portal_info(
    State(state): State<ServerState>,
) -> Json<PortalInfoResponse> {
    let transfers = state.transfers.lock().await;
    let file = transfers.values().next().cloned();
    Json(PortalInfoResponse {
        file,
        receive_password_required: state.receive_password.is_some(),
    })
}

// ── API: Download file (existing) ──────────────────────────────────────

async fn download_file(
    State(state): State<ServerState>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let transfers = state.transfers.lock().await;
    let transfer = transfers.values().next().ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "No file available"})),
        )
    })?;

    let file_path = transfer.file_path.clone();
    let file_size = transfer.file_size;
    let file_name = transfer.file_name.clone();
    let encrypted_data = transfer.encrypted_data.clone();
    drop(transfers);

    let mut limits = state.download_limits.lock().await;
    let entry = limits.entry(remote_addr.ip().to_string()).or_insert(DownloadLimit {
        window_started: std::time::Instant::now(),
        attempts: 0,
    });
    if entry.window_started.elapsed() >= std::time::Duration::from_secs(60) {
        entry.window_started = std::time::Instant::now();
        entry.attempts = 0;
    }
    if entry.attempts >= 10 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "Too many download attempts. Try again in a minute."})),
        ));
    }
    entry.attempts += 1;
    drop(limits);

    if let Some(data) = encrypted_data {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/octet-stream".parse().unwrap());
        headers.insert("content-length", data.len().to_string().parse().unwrap());
        headers.insert("x-tinytools-encrypted", "true".parse().unwrap());
        return Ok((headers, Body::from(data)).into_response());
    }

    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ))?;

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        "content-disposition",
        format!("attachment; filename=\"{}\"", file_name).parse().unwrap(),
    );
    headers.insert(
        "content-length",
        file_size.to_string().parse().unwrap(),
    );

    Ok((headers, body).into_response())
}

// ── API: Announce file (metadata only ─ no data touches device) ──────

#[derive(Serialize)]
struct AnnounceResponse {
    transfer_id: String,
}

async fn handle_announce(
    State(state): State<ServerState>,
    headers: HeaderMap,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<AnnounceResponse>, (StatusCode, Json<serde_json::Value>)> {
    let filename = headers
        .get("X-TinyTools-Filename")
        .and_then(|v| v.to_str().ok())
        .map(|v| urlencoding::decode(v).unwrap_or_default().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let file_size: u64 = headers
        .get("X-TinyTools-FileSize")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if let Some(ref expected_pwd) = state.receive_password {
        let provided = headers
            .get("X-TinyTools-Password")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if provided != expected_pwd {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Incorrect password"})),
            ));
        }
    }

    let transfer_id = uuid_simple();
    let sender_ip = remote_addr.ip().to_string();

    let incoming = IncomingTransfer {
        id: transfer_id.clone(),
        file_name: filename,
        file_size,
        sender_ip,
        temp_path: None,
        save_path: None,
        received_bytes: 0,
        encrypted: false,
        encryption_salt: None,
        encryption_nonce: None,
        status: "announced".to_string(),
        created_at: now_secs(),
    };

    {
        let mut transfers = get_incoming_transfers().lock().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
        transfers.insert(transfer_id.clone(), incoming);
    }

    Ok(Json(AnnounceResponse { transfer_id }))
}

// ── API: Upload data (only after user accepts download) ───────────────

async fn handle_upload_data(
    State(state): State<ServerState>,
    Path(transfer_id): Path<String>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let save_path = {
        let transfers = get_incoming_transfers().lock().map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;
        let t = transfers.get(&transfer_id).ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Transfer not found"})))
        })?;
        if t.status != "ready" {
            return Err((StatusCode::PRECONDITION_FAILED, Json(serde_json::json!({"error": "Transfer not ready"}))));
        }
        t.save_path.clone().ok_or_else(|| {
            (StatusCode::PRECONDITION_FAILED, Json(serde_json::json!({"error": "No save path set"})))
        })?
    };

    let save_path_buf = PathBuf::from(&save_path);

    // Set status to "receiving" so Tauri UI can show progress
    {
        let mut transfers = get_incoming_transfers().lock().map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;
        if let Some(t) = transfers.get_mut(&transfer_id) {
            t.status = "receiving".to_string();
        }
    }

    // Check if the upload is encrypted (browser sends encryption headers when password is set)
    let salt_header = headers.get("X-TinyTools-Salt").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let nonce_header = headers.get("X-TinyTools-Nonce").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let iterations_header = headers.get("X-TinyTools-Iterations").and_then(|v| v.to_str().ok()).and_then(|s| s.parse::<u32>().ok());

    let is_encrypted = salt_header.is_some();

    // If receive_password is set but upload is not encrypted, reject
    if state.receive_password.is_some() && !is_encrypted {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Encrypted upload required when receive password is set"}))));
    }

    if is_encrypted {
        // Buffered path: collect all bytes, decrypt, then write
        let password = state.receive_password.as_ref().ok_or_else(|| {
            (StatusCode::PRECONDITION_FAILED, Json(serde_json::json!({"error": "No receive password set for decryption"})))
        })?;
        let salt_b64 = salt_header.unwrap();
        let nonce_b64 = nonce_header.ok_or_else(|| {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Missing X-TinyTools-Nonce header"})))
        })?;
        let iterations = iterations_header.unwrap_or(encryption::PORTAL_PBKDF2_ITERATIONS);

        let salt = STANDARD.decode(&salt_b64)
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid salt encoding: {}", e)}))))?;
        let nonce = STANDARD.decode(&nonce_b64)
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid nonce encoding: {}", e)}))))?;

        let mut ciphertext = Vec::new();
        let mut stream = body.into_data_stream();
        let mut received: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
            })?;
            ciphertext.extend_from_slice(&chunk);
            received += chunk.len() as u64;

            let mut transfers = get_incoming_transfers().lock().map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
            })?;
            if let Some(t) = transfers.get_mut(&transfer_id) {
                t.received_bytes = received;
            }
        }

        let plaintext = encryption::decrypt_for_web_portal(password, &ciphertext, &salt, &nonce, iterations)
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Decryption failed: {}", e)}))))?;

        tokio::fs::write(&save_path_buf, &plaintext).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;

        let mut transfers = get_incoming_transfers().lock().map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;
        if let Some(t) = transfers.get_mut(&transfer_id) {
            if received >= t.file_size {
                t.status = "accepted".to_string();
            }
        }
    } else {
        // Stream directly to the chosen save path (plaintext)
        let mut file = tokio::fs::File::create(&save_path_buf).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;
        let mut stream = body.into_data_stream();
        let mut received: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
            })?;
            file.write_all(&chunk).await.map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
            })?;
            received += chunk.len() as u64;

            let mut transfers = get_incoming_transfers().lock().map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
            })?;
            if let Some(t) = transfers.get_mut(&transfer_id) {
                t.received_bytes = received;
                if received >= t.file_size {
                    t.status = "accepted".to_string();
                }
            }
        }
    }

    Ok(StatusCode::OK.into_response())
}

// ── API: Download a received transfer (stream from temp file) ────────

async fn download_transfer(
    Path(transfer_id): Path<String>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let (temp_path, file_name) = {
        let transfers = get_incoming_transfers().lock().map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;
        let t = transfers.get(&transfer_id).ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Transfer not found"})))
        })?;
        let fp = t.temp_path.clone().ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "File not available"})))
        })?;
        (fp, t.file_name.clone())
    };

    let file = tokio::fs::File::open(&temp_path).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
    })?;
    let meta = file.metadata().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
    })?;
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/octet-stream".parse().unwrap());
    headers.insert("content-disposition", format!("attachment; filename=\"{}\"", file_name).parse().unwrap());
    headers.insert("content-length", meta.len().to_string().parse().unwrap());
    Ok((headers, body).into_response())
}

// ── API: Upload status poll ───────────────────────────────────────────

#[derive(Serialize)]
struct UploadStatusResponse {
    status: String,
    received_bytes: u64,
    file_size: u64,
}

async fn upload_status(
    Path(transfer_id): Path<String>,
) -> Result<Json<UploadStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let transfers = get_incoming_transfers().lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    let (status, received_bytes, file_size) = transfers
        .get(&transfer_id)
        .map(|t| (t.status.clone(), t.received_bytes, t.file_size))
        .unwrap_or_else(|| ("not_found".to_string(), 0, 0));

    Ok(Json(UploadStatusResponse { status, received_bytes, file_size }))
}

// ── Server setup ────────────────────────────────────────────────────────

pub async fn start_server(
    state: ServerState,
    local_ip: &str,
) -> Result<(u16, tokio::task::JoinHandle<()>), String> {
    install_tls_provider();
    let listener = std::net::TcpListener::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind server: {}", e))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let certified_key = rcgen::generate_simple_self_signed(vec![
        "localhost".to_string(),
        local_ip.to_string(),
    ])
    .map_err(|e| format!("Failed to generate portal certificate: {}", e))?;
    let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem(
        certified_key.cert.pem().into_bytes(),
        certified_key.key_pair.serialize_pem().into_bytes(),
    )
    .await
    .map_err(|e| format!("Failed to configure HTTPS: {}", e))?;

    let app = Router::new()
        .route("/", get(portal_page))
        .route("/receive", get(receive_page))
        .route("/api/info", get(get_portal_info))
        .route("/api/download", get(download_file))
        .route("/api/announce", post(handle_announce))
        .route("/api/upload-data/:id", post(handle_upload_data))
        .route("/api/upload-status/:id", get(upload_status))
        .route("/api/download-transfer/:id", get(download_transfer))
        .with_state(state);

    let handle = tokio::spawn(async move {
        let _ = axum_server::from_tcp_rustls(listener, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await;
    });

    Ok((port, handle))
}

fn uuid_simple() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

use axum::routing::post;

fn get_dist_dir() -> std::path::PathBuf {
    let cwd_dist = std::path::PathBuf::from("dist");
    if cwd_dist.join("index.html").exists() {
        return cwd_dist;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_dist = parent.join("dist");
            if exe_dist.join("index.html").exists() {
                return exe_dist;
            }
            if parent.join("index.html").exists() {
                return parent.to_path_buf();
            }
        }
    }
    cwd_dist
}

async fn spa_fallback() -> Html<String> {
    let dist_dir = get_dist_dir();
    let index_path = dist_dir.join("index.html");
    let index_html = tokio::fs::read_to_string(&index_path)
        .await
        .unwrap_or_else(|_| "<html><body>App index.html not found</body></html>".to_string());
    Html(index_html)
}

pub async fn start_homelab_server(
    local_ip: &str,
) -> Result<(u16, tokio::task::JoinHandle<()>), String> {
    install_tls_provider();

    let port_env = std::env::var("TINYTOOLS_PORT").ok().and_then(|p| p.parse::<u16>().ok());
    let listener = if let Some(req_port) = port_env {
        std::net::TcpListener::bind(format!("0.0.0.0:{}", req_port))
            .map_err(|e| format!("Failed to bind to requested port {}: {}", req_port, e))?
    } else {
        std::net::TcpListener::bind("0.0.0.0:8443")
            .or_else(|_| std::net::TcpListener::bind("0.0.0.0:0"))
            .map_err(|e| format!("Failed to bind server listener: {}", e))?
    };
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let certified_key = rcgen::generate_simple_self_signed(vec![
        "localhost".to_string(),
        local_ip.to_string(),
    ])
    .map_err(|e| format!("Failed to generate portal certificate: {}", e))?;
    let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem(
        certified_key.cert.pem().into_bytes(),
        certified_key.key_pair.serialize_pem().into_bytes(),
    )
    .await
    .map_err(|e| format!("Failed to configure HTTPS: {}", e))?;

    let dist_dir = get_dist_dir();
    let assets_dir = dist_dir.join("assets");

    let state = ServerState {
        transfers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        downloads_dir: Arc::new(tokio::sync::Mutex::new(
            dirs_next::download_dir()
                .or_else(|| dirs_next::home_dir())
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
        )),
        download_limits: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        receive_password: None,
        receive_only: false,
    };

    let app = Router::new()
        .route("/api/info", get(get_portal_info))
        .route("/api/download", get(download_file))
        .route("/api/announce", post(handle_announce))
        .route("/api/upload-data/:id", post(handle_upload_data))
        .route("/api/upload-status/:id", get(upload_status))
        .route("/api/download-transfer/:id", get(download_transfer))
        .route("/receive", get(receive_page))
        .nest_service("/assets", ServeDir::new(assets_dir))
        .fallback(spa_fallback)
        .merge(crate::chat::server::chat_routes().with_state(()))
        .layer(axum::extract::DefaultBodyLimit::max(
            crate::chat::MAX_FILE_BYTES as usize + (1 << 20),
        ))
        .with_state(state);

    let handle = tokio::spawn(async move {
        let _ = axum_server::from_tcp_rustls(listener, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await;
    });

    Ok((port, handle))
}

