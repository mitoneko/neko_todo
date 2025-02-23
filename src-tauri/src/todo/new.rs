//! todo構造体新規作成
use super::*;

impl Todo {
    /// 初期化
    pub async fn new(host: &str, user: &str, pass: &str) -> Result<Self, TodoError> {
        let db = Database::new(host, user, pass).await.map_err(|e| match e {
            DbError::FailConnect(e2) => TodoError::DbInit(e2),
            e => unreachable!("[ToDo::new] Database::new()[{e}]"),
        })?;
        Ok(Self { database: db })
    }
}
