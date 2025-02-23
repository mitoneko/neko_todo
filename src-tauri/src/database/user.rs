//! ユーザーデータの操作
use super::*;
use sqlx::{query, query_as};
use uuid::Uuid;

impl Database {
    /// ユーザーの追加
    pub async fn add_user(&self, name: &str, pass: &str) -> Result<(), DbError> {
        let sql = "insert into users(name, password) values (?, ?);";
        query(sql)
            .bind(name)
            .bind(pass)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(ref db_err) => {
                    if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation {
                        DbError::DuplicateUserName(e)
                    } else {
                        DbError::FailDbAccess(e)
                    }
                }
                _ => DbError::FailDbAccess(e),
            })?;
        Ok(())
    }

    /// ユーザー名をキーとして、ユーザー情報を取得
    pub async fn get_user(&self, name: &str) -> Result<User, DbError> {
        let sql = "select name, password from users where name = ?;";
        query_as(sql)
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => DbError::NotFoundUser,
                e => DbError::FailDbAccess(e),
            })
    }

    /// セッションIDをキーにしてユーザー情報を取得
    pub async fn get_user_from_sess(&self, sess: Uuid) -> Result<User, DbError> {
        let sql = r#"
            select u.name, u.password 
            from users u join sessions s on u.name=s.user_name 
            where s.id = ?;
            "#;

        query_as(sql)
            .bind(sess.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => DbError::NotFoundSession,
                e => DbError::FailDbAccess(e),
            })
    }
}
