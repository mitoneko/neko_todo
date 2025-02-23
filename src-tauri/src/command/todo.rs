//! todoリスト操作インターフェース

use super::session::{get_cur_session_with_update, get_curr_session};
use crate::app_status::AppStatus;
use crate::database::ItemTodo;
use log::{debug, info};
use serde::Deserialize;
use tauri::State;

/// todoのリストを取得する。
#[tauri::command]
pub async fn get_todo_list(app_status: State<'_, AppStatus>) -> Result<Vec<ItemTodo>, String> {
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

    let ret = app_status
        .todo()
        .get_todo_list(sess, is_incomplete, sort_order)
        .await
        .map_err(|e| e.to_string())?;
    info!("todoリスト、{}件、取得完了", ret.len());
    Ok(ret)
}

/// todoアイテムを取得する
#[tauri::command]
pub async fn get_todo_with_id(
    app_status: State<'_, AppStatus>,
    id: u32,
) -> Result<ItemTodo, String> {
    let Some(sess) = get_curr_session(&app_status) else {
        return Err("NotLogin".to_string());
    };

    let ret = app_status
        .todo()
        .get_todo_with_id(id, sess)
        .await
        .map_err(|e| e.to_string())?;
    info!("todo一件の取得完了　id=>{}", id);
    Ok(ret)
}

/// todoを追加する。
#[tauri::command]
pub async fn add_todo(app_status: State<'_, AppStatus>, item: FormTodo) -> Result<(), String> {
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
        .map_err(|e| e.to_string())?;
    info!("todoの追加完了");
    Ok(())
}

/// todoの完了状態を変更する。
#[tauri::command]
pub async fn update_done(
    app_status: State<'_, AppStatus>,
    id: u32,
    done: bool,
) -> Result<(), String> {
    let sess = match get_cur_session_with_update(&app_status).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err("NotLogin".to_string()),
        Err(e) => return Err(e),
    };
    app_status
        .todo()
        .change_done(id, sess, done)
        .await
        .map_err(|e| e.to_string())?;
    info!(
        "todoの状態を変更。id=>{}, state=>{}",
        id,
        if done { "完了" } else { "未完了" }
    );
    Ok(())
}

/// todoの編集を行う。
#[tauri::command]
pub async fn edit_todo(
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
        .map_err(|e| e.to_string())?;
    info!("アイテム編集完了 id=>{}", id);
    Ok(())
}

/// Todo項目追加画面データ取得用
#[derive(Deserialize, Debug, Clone)]
pub struct FormTodo {
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
