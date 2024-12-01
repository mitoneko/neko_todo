//! tauri メインプロセス
mod config;
mod todo;

use todo::{Todo, TodoItem};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, get_todo_list])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_todo_list() -> Vec<TodoItem> {
    println!("todo取得");
    Todo::new().get_todo_list().unwrap()
}
