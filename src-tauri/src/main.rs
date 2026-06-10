#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::{Clipboard, ImageData};
use chrono::Local;
use image::GenericImageView;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::{
  borrow::Cow,
  fs,
  path::{Path, PathBuf},
  sync::Mutex,
  thread,
  time::Duration,
};
use tauri::{GlobalShortcutManager, Manager};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShortcutStatus {
  accelerator: String,
  registered: bool,
  error: Option<String>,
}

struct ShortcutRegistry(Mutex<Vec<ShortcutStatus>>);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
  last_parent_folder: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TcItem {
  tc_name: String,
  folder: String,
  status: String,
  step_count: u32,
  updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionState {
  parent_folder: String,
  #[serde(default)]
  current_tc_name: String,
  #[serde(default)]
  current_tc_folder: String,
  #[serde(default)]
  current_step: u32,
  #[serde(default = "default_session_status")]
  status: String,
  #[serde(default)]
  tc_list: Vec<TcItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptureResult {
  tc_name: String,
  tc_folder: String,
  step: u32,
  file_name: String,
  file_path: String,
  timestamp: String,
  #[serde(default)]
  clipboard_copied: bool,
  #[serde(default)]
  clipboard_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TcMetadata {
  tc_name: String,
  #[serde(default = "default_tc_status")]
  status: String,
  created_at: String,
  updated_at: String,
  #[serde(default)]
  screenshots: Vec<CaptureResult>,
}

#[derive(Clone, Copy, Debug)]
struct CropRect {
  x: u32,
  y: u32,
  width: u32,
  height: u32,
}

fn default_session_status() -> String {
  "idle".to_string()
}

fn default_tc_status() -> String {
  "empty".to_string()
}

fn now_iso() -> String {
  Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn tc_name(number: u32) -> String {
  format!("TC{:03}", number)
}

fn session_path(parent: &Path) -> PathBuf {
  parent.join("session.json")
}

fn metadata_path(parent: &Path, tc_name: &str) -> PathBuf {
  parent.join(tc_name).join("metadata.json")
}

fn value_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
  for key in keys {
    if let Some(text) = value.get(*key).and_then(|item| item.as_str()) {
      if !text.trim().is_empty() {
        return Some(text.to_string());
      }
    }
  }
  None
}

fn value_u32(value: &serde_json::Value, keys: &[&str]) -> Option<u32> {
  for key in keys {
    if let Some(number) = value.get(*key).and_then(|item| item.as_u64()) {
      return Some(number as u32);
    }
  }
  None
}

fn read_session_file(parent: &Path) -> Option<SessionState> {
  let path = session_path(parent);
  let content = fs::read_to_string(path).ok()?;
  let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;

  // [SESSION_COMPAT_FIX]
  // Beberapa versi awal Electron/Tauri sempat memakai key session yang berbeda,
  // misalnya currentTC/currentTc. Setelah app di-update, session lama tetap harus bisa dibaca.
  let mut state = serde_json::from_value::<SessionState>(value.clone()).unwrap_or(SessionState {
    parent_folder: value_string(&value, &["parentFolder", "parent_folder"])
      .unwrap_or_else(|| parent.to_string_lossy().to_string()),
    current_tc_name: String::new(),
    current_tc_folder: String::new(),
    current_step: 0,
    status: "idle".to_string(),
    tc_list: Vec::new(),
  });

  if state.parent_folder.trim().is_empty() {
    state.parent_folder = value_string(&value, &["parentFolder", "parent_folder"])
      .unwrap_or_else(|| parent.to_string_lossy().to_string());
  }

  if state.current_tc_name.trim().is_empty() {
    state.current_tc_name = value_string(
      &value,
      &["currentTcName", "currentTCName", "currentTC", "currentTc", "current_tc_name"],
    )
    .unwrap_or_default();
  }

  if state.current_tc_folder.trim().is_empty() {
    state.current_tc_folder = value_string(
      &value,
      &["currentTcFolder", "currentTCFolder", "current_tc_folder"],
    )
    .unwrap_or_default();
  }

  if state.current_tc_folder.trim().is_empty() && !state.current_tc_name.trim().is_empty() {
    state.current_tc_folder = parent.join(&state.current_tc_name).to_string_lossy().to_string();
  }

  if state.current_step == 0 {
    state.current_step = value_u32(&value, &["currentStep", "current_step"]).unwrap_or(0);
  }

  if state.status.trim().is_empty() {
    state.status = if state.current_tc_name.trim().is_empty() {
      "idle".to_string()
    } else {
      "recording".to_string()
    };
  }

  Some(state)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("Gagal membuat folder: {e}"))?;
  }

  let content = serde_json::to_string_pretty(value).map_err(|e| format!("Gagal serialize JSON: {e}"))?;
  fs::write(path, content).map_err(|e| format!("Gagal menulis file JSON: {e}"))
}

fn save_session(state: &SessionState) -> Result<(), String> {
  write_json(&session_path(Path::new(&state.parent_folder)), state)
}

fn count_png_files(folder: &Path) -> u32 {
  fs::read_dir(folder)
    .map(|entries| {
      entries
        .flatten()
        .filter(|entry| {
          entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("png"))
            .unwrap_or(false)
        })
        .count() as u32
    })
    .unwrap_or(0)
}

fn parse_tc_number(name: &str) -> Option<u32> {
  let suffix = name.strip_prefix("TC")?;
  suffix.parse::<u32>().ok()
}

fn find_max_tc_number(parent: &Path) -> u32 {
  fs::read_dir(parent)
    .map(|entries| {
      entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
          let file_name = entry.file_name();
          let name = file_name.to_string_lossy();
          parse_tc_number(&name)
        })
        .max()
        .unwrap_or(0)
    })
    .unwrap_or(0)
}

fn load_or_init_metadata(parent: &Path, tc_name: &str) -> TcMetadata {
  let path = metadata_path(parent, tc_name);
  if let Ok(content) = fs::read_to_string(path) {
    if let Ok(mut metadata) = serde_json::from_str::<TcMetadata>(&content) {
      if metadata.status.trim().is_empty() {
        metadata.status = default_tc_status();
      }
      return metadata;
    }
  }

  TcMetadata {
    tc_name: tc_name.to_string(),
    status: default_tc_status(),
    created_at: now_iso(),
    updated_at: now_iso(),
    screenshots: Vec::new(),
  }
}

fn save_tc_metadata_status(parent: &Path, tc_name: &str, status: &str) -> Result<(), String> {
  let mut metadata = load_or_init_metadata(parent, tc_name);
  metadata.status = status.to_string();
  metadata.updated_at = now_iso();
  write_json(&metadata_path(parent, tc_name), &metadata)
}

fn sort_tc_list(tc_list: &mut Vec<TcItem>) {
  tc_list.sort_by_key(|item| parse_tc_number(&item.tc_name).unwrap_or(u32::MAX));
}

fn upsert_tc_item(state: &mut SessionState, tc_name: &str, status: &str) {
  let parent = Path::new(&state.parent_folder);
  let folder = parent.join(tc_name);
  let folder_string = folder.to_string_lossy().to_string();
  let step_count = count_png_files(&folder);
  let updated_at = now_iso();

  if let Some(item) = state.tc_list.iter_mut().find(|item| item.tc_name == tc_name) {
    item.folder = folder_string;
    item.status = status.to_string();
    item.step_count = step_count;
    item.updated_at = updated_at;
  } else {
    state.tc_list.push(TcItem {
      tc_name: tc_name.to_string(),
      folder: folder_string,
      status: status.to_string(),
      step_count,
      updated_at,
    });
  }

  sort_tc_list(&mut state.tc_list);
}

fn sync_tc_list_with_folders(state: &mut SessionState) {
  let parent = Path::new(&state.parent_folder);

  if let Ok(entries) = fs::read_dir(parent) {
    for entry in entries.flatten() {
      if !entry.path().is_dir() {
        continue;
      }

      let name = entry.file_name().to_string_lossy().to_string();
      if parse_tc_number(&name).is_none() {
        continue;
      }

      if !state.tc_list.iter().any(|item| item.tc_name == name) {
        let step_count = count_png_files(&entry.path());
        let status = if state.status == "recording" && state.current_tc_name == name {
          "in_progress"
        } else if step_count > 0 {
          "done"
        } else {
          "empty"
        };

        state.tc_list.push(TcItem {
          tc_name: name.clone(),
          folder: entry.path().to_string_lossy().to_string(),
          status: status.to_string(),
          step_count,
          updated_at: now_iso(),
        });
      }
    }
  }

  for item in state.tc_list.iter_mut() {
    let folder = parent.join(&item.tc_name);
    item.folder = folder.to_string_lossy().to_string();
    item.step_count = count_png_files(&folder);
    if item.status.trim().is_empty() {
      item.status = if item.step_count > 0 { "done" } else { "empty" }.to_string();
    }
  }

  if !state.current_tc_name.is_empty() && state.status == "recording" {
    let current_tc = state.current_tc_name.clone();
    upsert_tc_item(state, &current_tc, "in_progress");
  }

  sort_tc_list(&mut state.tc_list);
}

fn create_or_switch_tc_state(
  parent_folder: &str,
  mut state: SessionState,
  tc_number: u32,
  new_status: &str,
) -> Result<SessionState, String> {
  let parent = Path::new(parent_folder);
  let tc = tc_name(tc_number);
  let tc_folder = parent.join(&tc);

  fs::create_dir_all(&tc_folder).map_err(|e| format!("Gagal membuat folder {tc}: {e}"))?;

  state.parent_folder = parent_folder.to_string();
  state.current_tc_name = tc.clone();
  state.current_tc_folder = tc_folder.to_string_lossy().to_string();
  state.current_step = count_png_files(&tc_folder);
  state.status = "recording".to_string();
  upsert_tc_item(&mut state, &tc, new_status);
  save_tc_metadata_status(parent, &tc, new_status)?;
  save_session(&state)?;

  Ok(state)
}

fn normalize_session(mut state: SessionState) -> SessionState {
  if !state.current_tc_folder.is_empty() {
    state.current_step = count_png_files(Path::new(&state.current_tc_folder));
  }
  sync_tc_list_with_folders(&mut state);
  state
}

#[cfg(target_os = "windows")]
fn active_window_bounds() -> Option<(i32, i32, i32, i32)> {
  use windows_sys::Win32::Foundation::RECT;
  use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, IsIconic};

  unsafe {
    let hwnd = GetForegroundWindow();
    if hwnd.is_null() || IsIconic(hwnd) != 0 {
      return None;
    }

    let mut rect = RECT {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
    };

    // Build fix Windows:
    // Gunakan GetWindowRect saja agar kompatibel dengan windows-sys di GitHub Actions.
    // Ini tetap mengambil batas window aktif, sehingga taskbar Windows tidak ikut terscreenshot.
    if GetWindowRect(hwnd, &mut rect) == 0 {
      return None;
    }

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 120 || height <= 120 {
      return None;
    }

    Some((rect.left, rect.top, rect.right, rect.bottom))
  }
}

#[cfg(not(target_os = "windows"))]
fn active_window_bounds() -> Option<(i32, i32, i32, i32)> {
  None
}

fn browser_crop_insets() -> (u32, u32) {
  // Mode Browser Area:
  // - Windows: crop dari area address/search bar pada window aktif. Taskbar tidak ikut karena yang dicrop adalah window aktif.
  // - macOS/Linux fallback: crop dari screenshot monitor utama. Angka ini bisa disesuaikan kalau browser theme berbeda.
  if cfg!(target_os = "windows") {
    (48, 0)
  } else if cfg!(target_os = "macos") {
    // [MAC_BROWSER_TAB_CROP_FIX]
    // Crop lebih tinggi agar tab strip Chrome tidak ikut. Targetnya hasil mulai dari area address/search bar.
    (72, 72)
  } else {
    (52, 0)
  }
}

fn clamp_crop_rect(rect: CropRect, image_width: u32, image_height: u32) -> Option<CropRect> {
  if image_width == 0 || image_height == 0 {
    return None;
  }

  let x = rect.x.min(image_width.saturating_sub(1));
  let y = rect.y.min(image_height.saturating_sub(1));
  let max_width = image_width.saturating_sub(x);
  let max_height = image_height.saturating_sub(y);
  let width = rect.width.min(max_width);
  let height = rect.height.min(max_height);

  if width < 80 || height < 80 {
    return None;
  }

  Some(CropRect { x, y, width, height })
}

fn is_fullscreen_like_window(left: i32, top: i32, right: i32, bottom: i32, image_width: u32, image_height: u32) -> bool {
  if image_width < 120 || image_height < 120 || right <= left || bottom <= top {
    return false;
  }

  let visible_left = left.max(0) as u32;
  let visible_top = top.max(0) as u32;
  let visible_right = (right.max(0) as u32).min(image_width);
  let visible_bottom = (bottom.max(0) as u32).min(image_height);

  if visible_right <= visible_left || visible_bottom <= visible_top {
    return false;
  }

  let visible_width = visible_right.saturating_sub(visible_left);
  let visible_height = visible_bottom.saturating_sub(visible_top);
  let near_left_edge = left <= 12 && visible_left <= 12;
  let near_top_edge = top <= 12 && visible_top <= 12;
  let covers_width = visible_width >= image_width.saturating_sub(16);
  let covers_height = visible_height >= image_height.saturating_sub(16);

  near_left_edge && near_top_edge && covers_width && covers_height
}

fn copy_image_to_clipboard(image: &image::DynamicImage) -> Result<(), String> {
  let rgba = image.to_rgba8();
  let width = rgba.width() as usize;
  let height = rgba.height() as usize;
  let bytes = rgba.into_raw();

  // [CLIPBOARD_CAPTURE_FIX]
  // Setiap screenshot tetap disimpan sebagai file PNG, lalu image final yang sama juga disalin ke clipboard.
  // Clipboard failure tidak boleh menggagalkan capture karena evidence file tetap lebih penting.
  let mut clipboard = Clipboard::new().map_err(|e| format!("Gagal membuka clipboard: {e}"))?;
  clipboard
    .set_image(ImageData {
      width,
      height,
      bytes: Cow::Owned(bytes),
    })
    .map_err(|e| format!("Gagal menyalin screenshot ke clipboard: {e}"))
}

fn save_captured_image(image: image::DynamicImage, file_path: &Path) -> Result<(bool, Option<String>), String> {
  let (image_width, image_height) = image.dimensions();

  let final_image = if let Some(crop) = calculate_browser_crop(image_width, image_height) {
    image.crop_imm(crop.x, crop.y, crop.width, crop.height)
  } else {
    image
  };

  final_image
    .save(file_path)
    .map_err(|e| format!("Gagal menyimpan screenshot: {e}"))?;

  let clipboard_status = match copy_image_to_clipboard(&final_image) {
    Ok(()) => (true, None),
    Err(error) => (false, Some(error)),
  };

  Ok(clipboard_status)
}

#[cfg(target_os = "macos")]
fn capture_display_image(screen: &Screen, file_path: &Path) -> Result<image::DynamicImage, String> {
  use std::process::Command;

  let temp_name = file_path
    .file_name()
    .and_then(|name| name.to_str())
    .map(|name| format!(".{name}.native-capture.tmp.png"))
    .unwrap_or_else(|| ".native-capture.tmp.png".to_string());
  let temp_path = file_path.with_file_name(temp_name);

  // [FULLSCREEN_CAPTURE_FIX]
  // macOS fullscreen apps live in a separate Space and the screenshots crate can fail/return a stale desktop
  // in some setups. The native screencapture command is more reliable for fullscreen windows, so try it first
  // and fall back to the Rust crate if the command is unavailable or blocked by Screen Recording permission.
  let native_result = Command::new("screencapture").arg("-x").arg(&temp_path).output();

  if let Ok(output) = native_result {
    if output.status.success() && temp_path.exists() {
      match image::open(&temp_path) {
        Ok(image) => {
          let _ = fs::remove_file(&temp_path);
          return Ok(image);
        }
        Err(_) => {
          let _ = fs::remove_file(&temp_path);
        }
      }
    } else {
      let _ = fs::remove_file(&temp_path);
    }
  }

  let image = screen.capture().map_err(|e| format!("Gagal mengambil screenshot: {e}"))?;
  Ok(image::DynamicImage::ImageRgba8(image))
}

#[cfg(not(target_os = "macos"))]
fn capture_display_image(screen: &Screen, _file_path: &Path) -> Result<image::DynamicImage, String> {
  let image = screen.capture().map_err(|e| format!("Gagal mengambil screenshot: {e}"))?;
  Ok(image::DynamicImage::ImageRgba8(image))
}

fn calculate_browser_crop(image_width: u32, image_height: u32) -> Option<CropRect> {
  let (top_crop, bottom_crop) = browser_crop_insets();

  if let Some((left, top, right, bottom)) = active_window_bounds() {
    // [FULLSCREEN_CAPTURE_FIX]
    // Kalau browser/app sedang F11/fullscreen, address bar biasanya tidak tampil.
    // Jangan pakai crop browser-area karena top crop 48/72px membuat hasil seperti gagal/terpotong.
    if is_fullscreen_like_window(left, top, right, bottom, image_width, image_height) {
      return clamp_crop_rect(
        CropRect {
          x: 0,
          y: 0,
          width: image_width,
          height: image_height,
        },
        image_width,
        image_height,
      );
    }

    let x = left.max(0) as u32;
    let y = top.max(0) as u32 + top_crop;
    let right = right.max(0) as u32;
    let bottom = bottom.max(0) as u32;
    let width = right.saturating_sub(x);
    let height = bottom.saturating_sub(y);
    return clamp_crop_rect(CropRect { x, y, width, height }, image_width, image_height);
  }

  // Fallback untuk macOS/Linux: crop monitor utama dengan offset tetap.
  let y = top_crop.min(image_height.saturating_sub(1));
  let height = image_height.saturating_sub(y).saturating_sub(bottom_crop);
  clamp_crop_rect(
    CropRect {
      x: 0,
      y,
      width: image_width,
      height,
    },
    image_width,
    image_height,
  )
}

fn app_config_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path_resolver()
    .app_config_dir()
    .ok_or_else(|| "Gagal membaca folder config aplikasi.".to_string())?;

  fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat folder config aplikasi: {e}"))?;
  Ok(dir.join("app-config.json"))
}

#[tauri::command]
fn save_last_parent_folder(app: tauri::AppHandle, parent_folder: String) -> Result<(), String> {
  if parent_folder.trim().is_empty() {
    return Ok(());
  }

  // [LAST_PARENT_FOLDER_FIX]
  // Simpan parent folder terakhir di app config agar setelah aplikasi di-update,
  // user tidak perlu Browse ulang hanya untuk membaca session.json yang sudah ada.
  let config = AppConfig { last_parent_folder: parent_folder };
  write_json(&app_config_file(&app)?, &config)
}

#[tauri::command]
fn load_last_parent_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
  let path = app_config_file(&app)?;
  if !path.exists() {
    return Ok(None);
  }

  let content = fs::read_to_string(path).map_err(|e| format!("Gagal membaca config aplikasi: {e}"))?;
  let config = serde_json::from_str::<AppConfig>(&content)
    .map_err(|e| format!("Gagal membaca format config aplikasi: {e}"))?;

  if config.last_parent_folder.trim().is_empty() {
    return Ok(None);
  }

  if Path::new(&config.last_parent_folder).exists() {
    Ok(Some(config.last_parent_folder))
  } else {
    Ok(None)
  }
}

#[tauri::command]
fn read_session(parent_folder: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);

  if let Some(state) = read_session_file(parent) {
    return Ok(normalize_session(state));
  }

  Ok(SessionState {
    parent_folder,
    current_tc_name: String::new(),
    current_tc_folder: String::new(),
    current_step: 0,
    status: "idle".to_string(),
    tc_list: Vec::new(),
  })
}

#[tauri::command]
fn start_or_resume_flow(parent_folder: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);
  fs::create_dir_all(parent).map_err(|e| format!("Gagal membuat parent folder: {e}"))?;

  if let Some(state) = read_session_file(parent) {
    let state = normalize_session(state);
    if state.status == "recording" && !state.current_tc_name.is_empty() {
      save_session(&state)?;
      return Ok(state);
    }

    // Kalau session sudah finished, mulai TC baru tanpa overwrite folder lama.
    let next_number = find_max_tc_number(parent) + 1;
    return create_or_switch_tc_state(&parent_folder, state, next_number.max(1), "in_progress");
  }

  let state = SessionState {
    parent_folder: parent_folder.clone(),
    current_tc_name: String::new(),
    current_tc_folder: String::new(),
    current_step: 0,
    status: "idle".to_string(),
    tc_list: Vec::new(),
  };

  create_or_switch_tc_state(&parent_folder, state, 1, "in_progress")
}

#[tauri::command]
fn next_tc(parent_folder: String, current_tc_name: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);
  fs::create_dir_all(parent).map_err(|e| format!("Gagal membuat parent folder: {e}"))?;

  let mut state = read_session_file(parent).unwrap_or(SessionState {
    parent_folder: parent_folder.clone(),
    current_tc_name: current_tc_name.clone(),
    current_tc_folder: parent.join(&current_tc_name).to_string_lossy().to_string(),
    current_step: 0,
    status: "recording".to_string(),
    tc_list: Vec::new(),
  });

  state = normalize_session(state);

  if !current_tc_name.trim().is_empty() {
    upsert_tc_item(&mut state, &current_tc_name, "done");
    save_tc_metadata_status(parent, &current_tc_name, "done")?;
  }

  let current_number = parse_tc_number(&current_tc_name).unwrap_or(0);
  let next_number = find_max_tc_number(parent).max(current_number) + 1;
  create_or_switch_tc_state(&parent_folder, state, next_number, "in_progress")
}

#[tauri::command]
fn mark_pending(parent_folder: String, current_tc_name: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);
  fs::create_dir_all(parent).map_err(|e| format!("Gagal membuat parent folder: {e}"))?;

  if current_tc_name.trim().is_empty() {
    return Err("Belum ada TC aktif untuk ditandai Pending.".to_string());
  }

  let mut state = read_session_file(parent).unwrap_or(SessionState {
    parent_folder: parent_folder.clone(),
    current_tc_name: current_tc_name.clone(),
    current_tc_folder: parent.join(&current_tc_name).to_string_lossy().to_string(),
    current_step: 0,
    status: "recording".to_string(),
    tc_list: Vec::new(),
  });

  state = normalize_session(state);
  upsert_tc_item(&mut state, &current_tc_name, "pending");
  save_tc_metadata_status(parent, &current_tc_name, "pending")?;

  let current_number = parse_tc_number(&current_tc_name).unwrap_or(0);
  let next_number = find_max_tc_number(parent).max(current_number) + 1;
  create_or_switch_tc_state(&parent_folder, state, next_number, "in_progress")
}

#[tauri::command]
fn resume_tc(parent_folder: String, target_tc_name: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);
  let target_folder = parent.join(&target_tc_name);

  if parse_tc_number(&target_tc_name).is_none() || !target_folder.exists() {
    return Err(format!("{target_tc_name} tidak ditemukan di parent folder."));
  }

  let mut state = read_session_file(parent).unwrap_or(SessionState {
    parent_folder: parent_folder.clone(),
    current_tc_name: String::new(),
    current_tc_folder: String::new(),
    current_step: 0,
    status: "idle".to_string(),
    tc_list: Vec::new(),
  });

  state = normalize_session(state);

  // Saat user resume TC lama, TC aktif sebelumnya ditandai Pending agar tidak hilang dari alur kerja.
  if state.status == "recording" && !state.current_tc_name.is_empty() && state.current_tc_name != target_tc_name {
    let previous_tc = state.current_tc_name.clone();
    upsert_tc_item(&mut state, &previous_tc, "pending");
    save_tc_metadata_status(parent, &previous_tc, "pending")?;
  }

  state.current_tc_name = target_tc_name.clone();
  state.current_tc_folder = target_folder.to_string_lossy().to_string();
  state.current_step = count_png_files(&target_folder);
  state.status = "recording".to_string();
  upsert_tc_item(&mut state, &target_tc_name, "in_progress");
  save_tc_metadata_status(parent, &target_tc_name, "in_progress")?;
  save_session(&state)?;

  Ok(state)
}

fn do_capture(parent_folder: String, current_tc_name: String, current_step: u32) -> Result<CaptureResult, String> {
  if parent_folder.trim().is_empty() {
    return Err("Parent folder belum dipilih.".to_string());
  }

  if current_tc_name.trim().is_empty() {
    return Err("Flow belum dimulai. Klik Start / Resume Flow dulu.".to_string());
  }

  let parent = Path::new(&parent_folder);
  let tc_folder = parent.join(&current_tc_name);
  fs::create_dir_all(&tc_folder).map_err(|e| format!("Gagal membuat folder TC: {e}"))?;

  let existing_count = count_png_files(&tc_folder);
  let step = current_step.max(existing_count) + 1;
  let timestamp_for_file = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
  let unique_id = Uuid::new_v4().simple().to_string()[0..4].to_string();
  let file_name = format!("{:03}_{}_{}.png", step, timestamp_for_file, unique_id);
  let file_path = tc_folder.join(&file_name);

  let screens = Screen::all().map_err(|e| format!("Gagal membaca layar: {e}"))?;
  let screen = screens.first().ok_or_else(|| "Tidak ada layar yang bisa dibaca.".to_string())?;

  // Browser Area Mode:
  // Ambil layar utama lalu crop agar hasil dimulai dari area address/search bar.
  // Di Windows, crop memakai batas window aktif sehingga taskbar tidak ikut.
  // Di macOS, capture native diprioritaskan agar window fullscreen di Space aktif tetap bisa diambil.
  let image = capture_display_image(screen, &file_path)?;
  let (clipboard_copied, clipboard_error) = save_captured_image(image, &file_path)?;

  let capture = CaptureResult {
    tc_name: current_tc_name.clone(),
    tc_folder: tc_folder.to_string_lossy().to_string(),
    step,
    file_name,
    file_path: file_path.to_string_lossy().to_string(),
    timestamp: now_iso(),
    clipboard_copied,
    clipboard_error,
  };

  let mut metadata = load_or_init_metadata(parent, &current_tc_name);
  metadata.status = "in_progress".to_string();
  metadata.updated_at = now_iso();
  metadata.screenshots.push(capture.clone());
  write_json(&metadata_path(parent, &current_tc_name), &metadata)?;

  let mut session = read_session_file(parent).unwrap_or(SessionState {
    parent_folder: parent_folder.clone(),
    current_tc_name: current_tc_name.clone(),
    current_tc_folder: tc_folder.to_string_lossy().to_string(),
    current_step: step,
    status: "recording".to_string(),
    tc_list: Vec::new(),
  });

  session.current_tc_name = current_tc_name;
  session.current_tc_folder = tc_folder.to_string_lossy().to_string();
  session.current_step = step;
  session.status = "recording".to_string();
  let session_tc = session.current_tc_name.clone();
  upsert_tc_item(&mut session, &session_tc, "in_progress");
  save_session(&session)?;

  Ok(capture)
}

#[tauri::command]
fn capture_screenshot(
  window: tauri::Window,
  parent_folder: String,
  current_tc_name: String,
  current_step: u32,
  hide_window: bool,
) -> Result<CaptureResult, String> {
  if hide_window {
    let _ = window.hide();
    // [FULLSCREEN_CAPTURE_FIX]
    // Fullscreen window kadang butuh sedikit waktu untuk repaint/focus kembali setelah app recorder disembunyikan.
    // Delay ini mencegah hasil blank/stale terutama saat capture dari tombol app, bukan dari shortcut.
    thread::sleep(Duration::from_millis(650));
  }

  let result = do_capture(parent_folder, current_tc_name, current_step);

  if hide_window {
    let _ = window.show();
    let _ = window.set_focus();
  }

  result
}

#[tauri::command]
fn finish_flow(parent_folder: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);
  let mut state = read_session_file(parent).unwrap_or(SessionState {
    parent_folder: parent_folder.clone(),
    current_tc_name: String::new(),
    current_tc_folder: String::new(),
    current_step: 0,
    status: "idle".to_string(),
    tc_list: Vec::new(),
  });

  state = normalize_session(state);
  if !state.current_tc_name.is_empty() {
    let current_tc = state.current_tc_name.clone();
    upsert_tc_item(&mut state, &current_tc, "done");
    save_tc_metadata_status(parent, &current_tc, "done")?;
  }

  state.status = "finished".to_string();
  save_session(&state)?;
  Ok(state)
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
  if path.trim().is_empty() {
    return Err("Path masih kosong.".to_string());
  }

  open::that(path).map_err(|e| format!("Gagal membuka folder: {e}"))
}

#[tauri::command]
fn get_shortcut_status(state: tauri::State<ShortcutRegistry>) -> Result<Vec<ShortcutStatus>, String> {
  state
    .0
    .lock()
    .map(|items| items.clone())
    .map_err(|_| "Gagal membaca status shortcut.".to_string())
}

fn main() {
  tauri::Builder::default()
    .manage(ShortcutRegistry(Mutex::new(Vec::new())))
    .setup(|app| {
      let shortcuts = [
        ("CommandOrControl+Shift+S", "shortcut-capture"),
        ("CommandOrControl+Shift+N", "shortcut-next-tc"),
        ("CommandOrControl+Shift+P", "shortcut-mark-pending"),
        ("CommandOrControl+Shift+F", "shortcut-finish"),
      ];

      for (accelerator, event_name) in shortcuts {
        let app_handle = app.handle();
        let event_name = event_name.to_string();
        let accelerator_for_status = accelerator.to_string();

        let register_result = app.global_shortcut_manager().register(accelerator, move || {
          let _ = app_handle.emit_all(&event_name, ());
        });

        let status = ShortcutStatus {
          accelerator: accelerator_for_status,
          registered: register_result.is_ok(),
          error: register_result.err().map(|e| e.to_string()),
        };

        if let Ok(mut items) = app.state::<ShortcutRegistry>().0.lock() {
          items.push(status);
        }
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      save_last_parent_folder,
      load_last_parent_folder,
      read_session,
      start_or_resume_flow,
      next_tc,
      mark_pending,
      resume_tc,
      capture_screenshot,
      finish_flow,
      open_path,
      get_shortcut_status
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
