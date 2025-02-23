//! ユーザー情報の操作

use super::*;
use bcrypt::{hash, DEFAULT_COST};
use log::error;

use crate::database::*;

impl Todo {
    /// ユーザーの追加を行う。
    pub async fn add_user(&self, name: &str, password: &str) -> Result<(), TodoError> {
        let hashed_pass = hash(password, DEFAULT_COST)?;
        if let Err(e) = self.database.add_user(name, &hashed_pass).await {
            match e {
                DbError::DuplicateUserName(e) => return Err(TodoError::DuplicateUser(e)),
                DbError::FailDbAccess(e) => {
                    error!("[Todo::add_user]Database::add_user:[{e}]");
                    return Err(TodoError::FailDbAccess(e));
                }
                _ => {}
            }
        }
        Ok(())
    }
}
