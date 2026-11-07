mod args;
mod commands;
mod options;
mod progress;
mod runner;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(runner::AppState::default())
        .setup(|app| {
            let path = runner::ensure_ytdlp(app.handle());
            *app.state::<runner::AppState>().ytdlp.lock().unwrap() = path;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_options,
            commands::preview_command,
            commands::start_download,
            commands::cancel_download,
            commands::get_queue,
            commands::default_folder,
            commands::update_ytdlp,
            commands::get_ytdlp_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
