//! ユーザー操作インターフェース

use crate::app_status::AppStatus;
use log::info;
use tauri::{command, State};

// ユーザー登録
#[command]
pub async fn regist_user(
    app_status: State<'_, AppStatus>,
    name: String,
    password: String,
) -> Result<(), String> {
    app_status
        .todo()
        .add_user(&name, &password)
        .await
        .map_err(|e| e.to_string())?;
    info!("ユーザー登録完了:user->{}", &name);
    Ok(())
}

/// ログイン
#[command]
pub async fn login(
    app_status: State<'_, AppStatus>,
    name: String,
    password: String,
) -> Result<String, String> {
    let session = app_status.todo().login(&name, &password).await?;

    let mut cnf = app_status.config().lock().unwrap();
    cnf.set_session_id(&session);
    //cnf.save().map_err(|e| format!("OtherError:{}", e))?;
    info!("ログイン完了:user->{}", &name);
    Ok(session.to_string())
}
