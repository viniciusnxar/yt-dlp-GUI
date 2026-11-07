// Funções expostas ao front.
use crate::args::{self, Selected};
use crate::options::{self, OptionDef};
use crate::runner::{self, AppState, JobInfo};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_options() -> Vec<OptionDef> {
    options::all()
}

#[tauri::command]
pub fn preview_command(selected: Selected) -> Result<String, String> {
    args::preview(&selected)
}

#[tauri::command]
pub fn start_download(app: AppHandle, url: String, selected: Selected, folder: String) -> Result<String, String> {
    let folder = if folder.trim().is_empty() { default_folder(app.clone())? } else { folder };
    let ffmpeg_dir = runner::sidecar_dir().to_string_lossy().into_owned();
    let a = args::full_args(&selected, &url, &folder, &ffmpeg_dir)?;
    Ok(runner::enqueue(&app, url.trim().to_string(), a))
}

#[tauri::command]
pub fn cancel_download(app: AppHandle, job_id: String) {
    runner::cancel(&app, &job_id)
}

#[tauri::command]
pub fn get_queue(state: State<AppState>) -> Vec<JobInfo> {
    state.snapshot()
}

#[tauri::command]
pub fn default_folder(app: AppHandle) -> Result<String, String> {
    app.path().download_dir().map(|p| p.to_string_lossy().into_owned()).map_err(|e| e.to_string())
}

async fn run_ytdlp(app: &AppHandle, arg: &str) -> Result<String, String> {
    let out = runner::ytdlp_command(app).args([arg]).output().await.map_err(|e| e.to_string())?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    Ok(text.trim().to_string())
}

#[tauri::command]
pub async fn update_ytdlp(app: AppHandle) -> Result<String, String> {
    run_ytdlp(&app, "-U").await
}

#[tauri::command]
pub async fn get_ytdlp_version(app: AppHandle) -> Result<String, String> {
    run_ytdlp(&app, "--version").await
}
