//! Todoアプリのビジネスロジック実装
mod app_state;
mod edit_todo;
mod get_todo;
mod new;
#[cfg(test)]
mod test;
mod user;

use crate::database::*;
use log::error;
use thiserror::Error;

/// todoアプリのビジネスロジック実装
pub struct Todo {
    database: Database,
}

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("FailInitDatabase")]
    DbInit(sqlx::Error),
    #[error("DuplicateUserName")]
    DuplicateUser(sqlx::Error),
    #[error("InvalidPassword:{0}")]
    HashUserPassword(#[from] bcrypt::BcryptError),
    #[error("NotFoundUser")]
    NotFoundUser,
    #[error("WrongPassword")]
    WrongPassword,
    #[error("NotFoundSession")]
    NotFoundSession,
    #[error("NotFoundTodo")]
    NotFoundTodo,
    #[error("DatabaseError:{0}")]
    FailDbAccess(sqlx::Error),
}

impl From<TodoError> for String {
    fn from(value: TodoError) -> Self {
        value.to_string()
    }
}
