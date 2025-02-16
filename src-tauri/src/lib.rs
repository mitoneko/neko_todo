//! tauri メインプロセス
mod app_status;
mod config;
mod database;
mod setup;
mod todo;

use app_status::AppStatus;
use config::ItemSortOrder;
use database::ItemTodo;
use log::{debug, error, info};
use serde::Deserialize;
use setup::setup;
use tauri::{command, Manager, State};
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

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_status)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_todo_list,
            get_todo_with_id,
            regist_user,
            login,
            is_valid_session,
            add_todo,
            update_done,
            edit_todo,
            set_is_incomplete,
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

/// todoのリストを取得する。
#[tauri::command]
async fn get_todo_list(app_status: State<'_, AppStatus>) -> Result<Vec<ItemTodo>, String> {
    let sess = match get_curr_session(&app_status) {
        Some(u) => u,
        None => return Err("NotLogin".to_string()),
    };

    let is_incomplete;
    let sort_order;
    {
        let conf = app_status.config().lock().unwrap();
        is_incomplete = conf.get_is_incomplete();
        sort_order = conf.get_item_sort_order();
    }

    app_status
        .todo()
        .get_todo_list(sess, is_incomplete, sort_order)
        .await
        .map_err(Into::into)
}

/// todoアイテムを取得する
#[tauri::command]
async fn get_todo_with_id(app_status: State<'_, AppStatus>, id: u32) -> Result<ItemTodo, String> {
    let Some(sess) = get_curr_session(&app_status) else {
        return Err("NotLogin".to_string());
    };

    app_status
        .todo()
        .get_todo_with_id(id, sess)
        .await
        .map_err(Into::into)
}

/// todoを追加する。
#[tauri::command]
async fn add_todo(app_status: State<'_, AppStatus>, item: FormTodo) -> Result<(), String> {
    let sess = match get_cur_session_with_update(&app_status).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err("NotLogin".to_string()),
        Err(e) => return Err(e),
    };

    debug!("input = {:?}", &item);
    app_status
        .todo()
        .add_todo(sess, &item.into())
        .await
        .map_err(Into::into)
}

/// todoの完了状態を変更する。
#[tauri::command]
async fn update_done(app_status: State<'_, AppStatus>, id: u32, done: bool) -> Result<(), String> {
    let sess = match get_cur_session_with_update(&app_status).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err("NotLogin".to_string()),
        Err(e) => return Err(e),
    };
    app_status
        .todo()
        .change_done(id, sess, done)
        .await
        .map_err(Into::into)
}

/// todoの編集を行う。
#[tauri::command]
async fn edit_todo(
    app_status: State<'_, AppStatus>,
    id: u32,
    item: FormTodo,
) -> Result<(), String> {
    let sess = match get_cur_session_with_update(&app_status).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err("NotLogin".to_string()),
        Err(e) => return Err(e),
    };

    debug!("input => id: {},  item: {:?}", id, &item);
    let mut item: ItemTodo = item.into();
    item.id = id;
    app_status
        .todo()
        .edit_todo(&item, sess)
        .await
        .map_err(Into::into)
}

/// 完了済みのみを表示するかどうかを設定する。
#[tauri::command]
fn set_is_incomplete(app_status: State<'_, AppStatus>, is_incomplete: bool) {
    let mut conf = app_status.config().lock().unwrap();
    conf.set_is_incomplete(is_incomplete);
}

/// ユーザー登録
#[tauri::command]
async fn regist_user(
    app_status: State<'_, AppStatus>,
    name: String,
    password: String,
) -> Result<(), String> {
    app_status
        .todo()
        .add_user(&name, &password)
        .await
        .map_err(Into::into)
}

/// ログイン
#[command]
async fn login(
    app_status: State<'_, AppStatus>,
    name: String,
    password: String,
) -> Result<String, String> {
    let session = app_status.todo().login(&name, &password).await?;

    let mut cnf = app_status.config().lock().unwrap();
    cnf.set_session_id(&session);
    //cnf.save().map_err(|e| format!("OtherError:{}", e))?;
    Ok(session.to_string())
}

/// 現在、有効なセッションが存在するかどうか確認。(ユーザI/F用)
#[command]
async fn is_valid_session(app_status: State<'_, AppStatus>) -> Result<bool, String> {
    let sess = get_cur_session_with_update(&app_status)
        .await
        .map(|i| i.is_some());
    match sess {
        Ok(sess) => info!("セッション確認({})", if sess { "有効" } else { "無効" }),
        Err(ref e) => info!("セション確認エラー({})", e),
    }
    sess
}

/// 現在のアイテムリストのソート方法を返す
#[command]
fn get_item_sort_order(app_status: State<'_, AppStatus>) -> String {
    app_status
        .config()
        .lock()
        .unwrap()
        .get_item_sort_order()
        .to_string()
}

/// アイテムリストのソート方法を設定する
#[command]
fn set_item_sort_order(app_status: State<'_, AppStatus>, sort_order: String) -> Result<(), String> {
    let sort_order = sort_order
        .parse::<ItemSortOrder>()
        .map_err(|e| e.to_string())?;
    app_status
        .config()
        .lock()
        .unwrap()
        .set_item_sort_order(sort_order);
    info!("ソートオーダー更新 => {}", sort_order.to_string());
    Ok(())
}

/// 現在、有効なセッションを返す。
/// 有効なセッションが存在すれば、セッションの更新を行い、期限を延長する。
async fn get_cur_session_with_update(app_status: &AppStatus) -> Result<Option<Uuid>, String> {
    let cur_session = get_curr_session(app_status);
    let Some(cur_session) = cur_session else {
        return Ok(None);
    };

    match app_status.todo().is_valid_session(&cur_session).await {
        Ok(Some(s)) => {
            // 更新されたセッションを再登録
            let mut cnf = app_status.config().lock().unwrap();
            cnf.set_session_id(&s);
            //cnf.save().map_err(|e| format!("FailSession:{e}"))?;
            Ok(Some(s))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("FailSession:{e}")),
    }
}

/// 現在のセッションを取得する。
fn get_curr_session(app_status: &AppStatus) -> Option<Uuid> {
    let conf = app_status.config().lock().unwrap();
    conf.get_session_id()
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
