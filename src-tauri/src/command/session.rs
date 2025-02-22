//! セッション関係の関数及びインターフェース

use crate::app_status::AppStatus;
use log::info;
use tauri::{command, State};
use uuid::Uuid;

/// 現在、有効なセッションが存在するかどうか確認。(ユーザI/F用)
#[command]
pub async fn is_valid_session(app_status: State<'_, AppStatus>) -> Result<bool, String> {
    let sess = get_cur_session_with_update(&app_status)
        .await
        .map(|i| i.is_some());
    match sess {
        Ok(sess) => info!("セッション確認({})", if sess { "有効" } else { "無効" }),
        Err(ref e) => info!("セション確認エラー({})", e),
    }
    sess
}

/// 現在、有効なセッションを返す。
/// 有効なセッションが存在すれば、セッションの更新を行い、期限を延長する。
pub async fn get_cur_session_with_update(app_status: &AppStatus) -> Result<Option<Uuid>, String> {
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
pub fn get_curr_session(app_status: &AppStatus) -> Option<Uuid> {
    let conf = app_status.config().lock().unwrap();
    conf.get_session_id()
}
