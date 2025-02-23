//! アプリケーション状態の管理

use super::*;
use crate::database::*;
use bcrypt::verify;
use uuid::Uuid;

impl Todo {
    /// ログイン処理を行う。
    pub async fn login(&self, name: &str, password: &str) -> Result<Uuid, TodoError> {
        // 認証
        let user = self.database.get_user(name).await.map_err(|e| match e {
            DbError::NotFoundUser => TodoError::NotFoundUser,
            DbError::FailDbAccess(e) => TodoError::FailDbAccess(e),
            e => unreachable!("[ToDo::login] Database::get_user:[{e}]"),
        })?;
        if !verify(password, &user.password)? {
            return Err(TodoError::WrongPassword);
        }
        // セッションの生成
        let session = self
            .database
            .make_new_session(&user.name)
            .await
            .map_err(|e| match e {
                DbError::NotFoundUser => TodoError::NotFoundUser,
                DbError::FailDbAccess(e) => TodoError::FailDbAccess(e),
                e => {
                    unreachable!("[Todo::login] Database::make_new_session:[{e}]")
                }
            })?;
        Ok(session)
    }

    /// 現在のログインの有効性を確認し、セッションIDを更新する。
    /// もし指定されたセッションIDが無効な場合は、Noneを返す。
    /// セッションが有効な場合は、更新されたセッションIDを返す。
    pub async fn is_valid_session(&self, sess: &Uuid) -> Result<Option<Uuid>, TodoError> {
        let is_valid = self
            .database
            .is_session_valid(sess)
            .await
            .map_err(|e| match e {
                DbError::FailDbAccess(e) => TodoError::FailDbAccess(e),
                e => {
                    unreachable!("[Todo::is_valid_session]is_session_valid:[{e}]")
                }
            })?;
        if is_valid {
            match self.database.update_session(sess).await {
                Ok(s) => Ok(Some(s)),
                Err(DbError::NotFoundSession) => Ok(None),
                Err(DbError::FailDbAccess(e)) => Err(TodoError::FailDbAccess(e)),
                Err(e) => {
                    unreachable!("[Todo::is_valid_session]update_session:[{e}]")
                }
            }
        } else {
            Ok(None)
        }
    }
}
