//! アプリケーションの全体ステータスの取得・設定用インターフェース

use crate::app_status::AppStatus;
use crate::config::ItemSortOrder;
use log::info;
use tauri::{command, State};

/// 完了済みのみを表示するかどうかを設定する。
#[tauri::command]
pub fn set_is_incomplete(app_status: State<'_, AppStatus>, is_incomplete: bool) {
    let mut conf = app_status.config().lock().unwrap();
    conf.set_is_incomplete(is_incomplete);
    info!("未完了のみ表示モードを{}にセット", is_incomplete);
}

/// 完了済みのみ表示モードの現在の値を取得する。
#[tauri::command]
pub fn get_is_incomplete(app_status: State<'_, AppStatus>) -> bool {
    app_status.config().lock().unwrap().get_is_incomplete()
}

/// 現在のアイテムリストのソート方法を返す
#[command]
pub fn get_item_sort_order(app_status: State<'_, AppStatus>) -> String {
    app_status
        .config()
        .lock()
        .unwrap()
        .get_item_sort_order()
        .to_string()
}

/// アイテムリストのソート方法を設定する
#[command]
pub fn set_item_sort_order(
    app_status: State<'_, AppStatus>,
    sort_order: String,
) -> Result<(), String> {
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
