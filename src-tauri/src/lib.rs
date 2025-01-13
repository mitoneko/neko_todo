//! tauri メインプロセス
mod app_status;
mod config;
mod database;
mod setup;
mod todo;

use app_status::AppStatus;
use database::ItemTodo;
use log::error;
use serde::Deserialize;
use setup::setup;
use tauri::{command, State};
use todo::TodoError;
use uuid::Uuid;

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
            is_valid_session,
            add_todo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_todo_list(app_status: State<'_, AppStatus>) -> Result<Vec<ItemTodo>, String> {
    println!("todo取得");
    let Some(sess) = get_curr_session(&app_status) else {
        return Err("NotLogin".to_string());
    };

    app_status
        .todo()
        .get_todo_list(sess)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_todo(app_status: State<'_, AppStatus>, item: FormTodo) -> Result<(), String> {
    let Some(sess) = get_curr_session(&app_status) else {
        return Err("NotLogin".to_string());
    };

    eprintln!("input = {:?}", &item);
    app_status
        .todo()
        .add_todo(sess, &item.into())
        .await
        .map_err(|e| match e {
            todo::TodoError::NotFoundSession => "NotFoundSession".to_string(),
            e => format!("OtherError:[{e}]"),
        })
}

/// Todo項目追加画面データ取得用
#[derive(Deserialize, Debug, Clone)]
struct FormTodo {
    title: String,
    work: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

impl From<FormTodo> for ItemTodo {
    fn from(val: FormTodo) -> Self {
        let start = val.start.map(|d| d.replace("/", "-").parse().unwrap());
        let end = val.end.map(|d| d.replace("/", "-").parse().unwrap());
        ItemTodo {
            id: 0,
            user_name: "".to_string(),
            title: val.title,
            work: val.work,
            update_date: None,
            start_date: start,
            end_date: end,
            done: false,
        }
    }
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
            TodoError::FailDbAccess(e) => Err(e.to_string()),
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
            TodoError::FailDbAccess(e) => format!("OtherError:{}", e),
            _ => unimplemented!("[lib.rs::login] loginから予期しないエラー"),
        })?;

    let mut cnf = app_status.config().lock().unwrap();
    cnf.set_session_id(&session);
    cnf.save().map_err(|e| format!("OtherError:{}", e))?;
    Ok(session.to_string())
}

#[command]
async fn is_valid_session(app_status: State<'_, AppStatus>) -> Result<bool, String> {
    let cur_session = get_curr_session(&app_status);
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

fn get_curr_session(app_status: &AppStatus) -> Option<Uuid> {
    let conf = app_status.config().lock().unwrap();
    conf.get_session_id()
}
