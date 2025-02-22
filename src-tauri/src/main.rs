// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_status;
mod command;
mod config;
mod database;
mod setup;
mod todo;

use app_status::AppStatus;
use command::app_state::{
    get_is_incomplete, get_item_sort_order, set_is_incomplete, set_item_sort_order,
};
use command::session::is_valid_session;
use command::todo::{add_todo, edit_todo, get_todo_list, get_todo_with_id, update_done};
use command::user::{login, regist_user};
use directories::ProjectDirs;
use log::{error, info};
use setup::setup;
use tauri::Manager;

fn main() {
    setup_log();
    run()
}

/// ロギング機構のセットアップ
fn setup_log() {
    let mut log_file: std::path::PathBuf = ProjectDirs::from("jp", "laki", "nekotodo")
        .unwrap()
        .config_dir()
        .into();
    if !log_file.exists() {
        std::fs::create_dir_all(&log_file).unwrap();
    }
    log_file.push("nekotodo.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}:{} {}",
                chrono::Local::now().format("%Y/%m/%d %H:%M:%S"),
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        //.level(log::LevelFilter::Debug)
        .chain(std::io::stderr())
        .chain(fern::log_file(log_file).unwrap())
        .apply()
        .unwrap();
}

/// アプリケーション本体部分
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn run() {
    let app_status = match setup() {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1)
        }
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_status)
        .invoke_handler(tauri::generate_handler![
            get_todo_list,
            get_todo_with_id,
            regist_user,
            login,
            is_valid_session,
            add_todo,
            update_done,
            edit_todo,
            set_is_incomplete,
            get_is_incomplete,
            set_item_sort_order,
            get_item_sort_order,
        ])
        .build(tauri::generate_context!())
        .expect("error thile build tauri application");
    app.run(|app, event| {
        if let tauri::RunEvent::Exit = event {
            info!("終了処理開始");
            let state = app.state::<AppStatus>();
            state.config().lock().unwrap().save().unwrap();
        }
    });
}
