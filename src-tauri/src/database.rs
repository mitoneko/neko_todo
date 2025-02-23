//! データベースの操作を司る
mod new;
mod session;
#[cfg(test)]
mod test;
mod todo;
mod user;

use chrono::NaiveDate;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{
    mysql::{MySqlPool, MySqlPoolOptions},
    prelude::*,
};
use thiserror::Error;

/// neko_dbデータベース操作関数郡
#[derive(Clone, Debug)]
pub struct Database {
    pool: MySqlPool,
}

#[derive(FromRow, Debug, PartialEq)]
pub struct User {
    pub name: String,
    pub password: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ItemTodo {
    pub id: u32,
    pub user_name: String,
    pub title: String,
    pub work: Option<String>,
    pub update_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub done: bool,
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("データベースへの接続に失敗。")]
    FailConnect(sqlx::Error),
    #[error("データベース操作失敗(一般)")]
    FailDbAccess(sqlx::Error),
    #[error("User挿入失敗(name重複)")]
    DuplicateUserName(sqlx::Error),
    #[error("ユーザーが見つかりません。")]
    NotFoundUser,
    #[error("指定されたセッションidが見つかりません。")]
    NotFoundSession,
    #[error("指定されたidのtodoが見つかりません。")]
    NotFoundTodo,
}
