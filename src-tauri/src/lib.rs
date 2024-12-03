//! tauri メインプロセス
mod app_status;
mod config;
mod todo;

use app_status::AppStatus;
use tauri::State;
use todo::TodoItem;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_status = AppStatus::new();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_status)
        .invoke_handler(tauri::generate_handler![greet, get_todo_list])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_todo_list(app_status: State<'_, AppStatus>) -> Result<Vec<TodoItem>, String> {
    println!("todo取得");

    Ok(app_status.todo().get_todo_list().unwrap())
}
