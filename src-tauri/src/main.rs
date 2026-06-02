#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Local;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::{
  fs,
  path::{Path, PathBuf},
  sync::Mutex,
  thread,
  time::Duration,
};
use tauri::{api::dialog::blocking::FileDialogBuilder, Manager};
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
struct SessionState {
  parent_folder: String,
  current_tc_name: String,
  current_tc_folder: String,
  current_step: u32,
  status: String,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TcMetadata {
  tc_name: String,
  created_at: String,
  updated_at: String,
  screenshots: Vec<CaptureResult>,
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

fn read_session_file(parent: &Path) -> Option<SessionState> {
  let path = session_path(parent);
  let content = fs::read_to_string(path).ok()?;
  serde_json::from_str::<SessionState>(&content).ok()
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

fn create_tc_state(parent_folder: &str, tc_number: u32) -> Result<SessionState, String> {
  let parent = Path::new(parent_folder);
  let tc = tc_name(tc_number);
  let tc_folder = parent.join(&tc);

  fs::create_dir_all(&tc_folder).map_err(|e| format!("Gagal membuat folder {tc}: {e}"))?;

  let state = SessionState {
    parent_folder: parent_folder.to_string(),
    current_tc_name: tc,
    current_tc_folder: tc_folder.to_string_lossy().to_string(),
    current_step: 0,
    status: "recording".to_string(),
  };

  save_session(&state)?;
  Ok(state)
}

fn load_or_init_metadata(parent: &Path, tc_name: &str) -> TcMetadata {
  let path = metadata_path(parent, tc_name);
  if let Ok(content) = fs::read_to_string(path) {
    if let Ok(metadata) = serde_json::from_str::<TcMetadata>(&content) {
      return metadata;
    }
  }

  TcMetadata {
    tc_name: tc_name.to_string(),
    created_at: now_iso(),
    updated_at: now_iso(),
    screenshots: Vec::new(),
  }
}

#[tauri::command]
fn choose_parent_folder() -> Option<String> {
  FileDialogBuilder::new()
    .set_title("Pilih Parent Folder")
    .pick_folder()
    .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn read_session(parent_folder: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);

  if let Some(mut state) = read_session_file(parent) {
    // Saat file di folder TC berubah manual, jumlah step dihitung ulang agar UI tidak menyesatkan.
    if !state.current_tc_folder.is_empty() {
      state.current_step = count_png_files(Path::new(&state.current_tc_folder));
    }
    return Ok(state);
  }

  Ok(SessionState {
    parent_folder,
    current_tc_name: String::new(),
    current_tc_folder: String::new(),
    current_step: 0,
    status: "idle".to_string(),
  })
}

#[tauri::command]
fn start_or_resume_flow(parent_folder: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);
  fs::create_dir_all(parent).map_err(|e| format!("Gagal membuat parent folder: {e}"))?;

  if let Some(mut state) = read_session_file(parent) {
    if state.status == "recording" && !state.current_tc_name.is_empty() {
      state.current_step = count_png_files(Path::new(&state.current_tc_folder));
      save_session(&state)?;
      return Ok(state);
    }
  }

  // Kalau session belum ada atau sudah finished, buat TC berikutnya tanpa overwrite folder lama.
  let next_number = find_max_tc_number(parent) + 1;
  create_tc_state(&parent_folder, next_number.max(1))
}

#[tauri::command]
fn next_tc(parent_folder: String, current_tc_name: String) -> Result<SessionState, String> {
  let parent = Path::new(&parent_folder);
  fs::create_dir_all(parent).map_err(|e| format!("Gagal membuat parent folder: {e}"))?;

  // Next TC dibuat berdasarkan angka terbesar yang sudah ada, bukan sekadar current+1.
  // Ini mencegah overwrite kalau folder TC sudah pernah dibuat sebelumnya.
  let current_number = parse_tc_number(&current_tc_name).unwrap_or(0);
  let next_number = find_max_tc_number(parent).max(current_number) + 1;
  create_tc_state(&parent_folder, next_number)
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

  let step = current_step + 1;
  let timestamp_for_file = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
  let unique_id = Uuid::new_v4().simple().to_string()[0..4].to_string();
  let file_name = format!("{:03}_{}_{}.png", step, timestamp_for_file, unique_id);
  let file_path = tc_folder.join(&file_name);

  let screens = Screen::all().map_err(|e| format!("Gagal membaca layar: {e}"))?;
  let screen = screens.first().ok_or_else(|| "Tidak ada layar yang bisa dibaca.".to_string())?;

  // MVP: capture monitor pertama/primary. Multi monitor selector bisa ditambahkan di versi berikutnya.
  let image = screen.capture().map_err(|e| format!("Gagal mengambil screenshot: {e}"))?;
  image
    .save(&file_path)
    .map_err(|e| format!("Gagal menyimpan screenshot: {e}"))?;

  let capture = CaptureResult {
    tc_name: current_tc_name.clone(),
    tc_folder: tc_folder.to_string_lossy().to_string(),
    step,
    file_name,
    file_path: file_path.to_string_lossy().to_string(),
    timestamp: now_iso(),
  };

  let mut metadata = load_or_init_metadata(parent, &current_tc_name);
  metadata.updated_at = now_iso();
  metadata.screenshots.push(capture.clone());
  write_json(&metadata_path(parent, &current_tc_name), &metadata)?;

  let session = SessionState {
    parent_folder,
    current_tc_name,
    current_tc_folder: tc_folder.to_string_lossy().to_string(),
    current_step: step,
    status: "recording".to_string(),
  };
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
    thread::sleep(Duration::from_millis(280));
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
  });

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
      choose_parent_folder,
      read_session,
      start_or_resume_flow,
      next_tc,
      capture_screenshot,
      finish_flow,
      open_path,
      get_shortcut_status
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
