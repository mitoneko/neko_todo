//! tauri メインプロセス
mod app_status;
mod config;
mod database;
mod setup;
mod todo;

use app_status::AppStatus;
use log::error;
use setup::setup;
use tauri::{command, State};
use todo::{TodoError, TodoItem};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_status = match setup() {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1)
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_status)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_todo_list,
            regist_user,
            login,
            is_valid_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_todo_list(app_status: State<'_, AppStatus>) -> Result<Vec<TodoItem>, String> {
    println!("todo取得");

    Ok(app_status.todo().get_todo_list().unwrap())
}

#[tauri::command]
async fn regist_user(
    app_status: State<'_, AppStatus>,
    name: String,
    password: String,
) -> Result<(), String> {
    match app_status.todo().add_user(&name, &password).await {
        Ok(_) => Ok(()),
        Err(e) => match e {
            TodoError::DuplicateUser(_) => Err("DuplicateUser".to_string()),
            TodoError::HashUserPassword(e) => Err(format!("InvalidPassword.[{}]", e)),
            TodoError::AddUser(e) => Err(e.to_string()),
            _ => unimplemented!("[lib.rs regist_user] add_user返り値異常"),
        },
    }
}

#[command]
async fn login(
    app_status: State<'_, AppStatus>,
    name: String,
    password: String,
) -> Result<String, String> {
    let session = app_status
        .todo()
        .login(&name, &password)
        .await
        .map_err(|e| match e {
            TodoError::NotFoundUser => "NotFoundUser.".to_string(),
            TodoError::HashUserPassword(e) => format!("InvalidPassword.[{}]", e),
            TodoError::WrongPassword => "WrongPassword".to_string(),
            TodoError::FailLogin(e) => format!("OtherError:{}", e),
            _ => unimplemented!("[lib.rs::login] loginから予期しないエラー"),
        })?;

    let mut cnf = app_status.config().lock().unwrap();
    cnf.set_session_id(&session);
    cnf.save().map_err(|e| format!("OtherError:{}", e))?;
    Ok(session.to_string())
}

#[command]
async fn is_valid_session(app_status: State<'_, AppStatus>) -> Result<bool, String> {
    let cur_session;
    {
        let cnf = app_status.config().lock().unwrap();
        cur_session = cnf.get_session_id();
    }
    let Some(cur_session) = cur_session else {
        return Ok(false);
    };

    match app_status.todo().is_valid_session(&cur_session).await {
        Ok(Some(s)) => {
            // 更新されたセッションを再登録
            let mut cnf = app_status.config().lock().unwrap();
            cnf.set_session_id(&s);
            cnf.save().map_err(|e| format!("FailSession:{e}"))?;
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(e) => Err(format!("FailSession:{e}")),
    }
}
