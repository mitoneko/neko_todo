//! セッション情報の操作
use super::*;
use sqlx::{prelude::*, query};
use uuid::Uuid;

impl Database {
    /// セッション情報を新規作成する。
    /// 　生成したuuidを返す。
    pub async fn make_new_session(&self, user_name: &str) -> Result<Uuid, DbError> {
        let sql = "insert into sessions(id, user_name) values (?,?);";
        // キー情報の作成
        let id = Uuid::now_v7();

        query(sql)
            .bind(id.to_string())
            .bind(user_name)
            .execute(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref e) => {
                    if e.is_foreign_key_violation() {
                        // 外部キーエラー。存在しないユーザーを指定した。
                        return DbError::NotFoundUser;
                    }
                    DbError::FailDbAccess(err)
                }
                _ => DbError::FailDbAccess(err),
            })?;

        Ok(id)
    }

    /// 指定されたセッションを新規セッションに更新する。
    /// 指定されたセッションは削除され、新たなセッションidを発行する。
    pub async fn update_session(&self, id: &uuid::Uuid) -> Result<Uuid, DbError> {
        let mut tr = self.pool.begin().await.map_err(DbError::FailDbAccess)?;
        // 期限切れのセッション削除
        let sql_old_del = "delete from sessions where expired < now();";
        query(sql_old_del)
            .execute(&mut *tr)
            .await
            .map_err(DbError::FailDbAccess)?;

        // ユーザーIDの特定
        let sql_query_user = "select user_name from sessions where id=?;";
        let user: String = query(sql_query_user)
            .bind(id.to_string())
            .fetch_one(&mut *tr)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => DbError::NotFoundSession,
                e => DbError::FailDbAccess(e),
            })?
            .get("user_name");

        // 旧セッションの削除
        let sql_del_curr_sess = "delete from sessions where id = ?;";
        query(sql_del_curr_sess)
            .bind(id.to_string())
            .execute(&mut *tr)
            .await
            .map_err(DbError::FailDbAccess)?;

        // 新セッションの生成
        let sql_create_sess = "insert into sessions(id, user_name) values (?, ?);";
        let id = Uuid::now_v7();
        query(sql_create_sess)
            .bind(id.to_string())
            .bind(user)
            .execute(&mut *tr)
            .await
            .map_err(DbError::FailDbAccess)?;

        tr.commit().await.map_err(DbError::FailDbAccess)?;
        Ok(id)
    }

    /// 指定されたセッションIDが有効であるか確認する。
    /// データベースエラーが発生した場合は、Err(DbError::FailDbAccess)を返す。
    pub async fn is_session_valid(&self, sess: &Uuid) -> Result<bool, DbError> {
        // 期限切れのセッションを削除する。
        let sql_old_del = "delete from sessions where expired < now();";
        query(sql_old_del)
            .execute(&self.pool)
            .await
            .map_err(DbError::FailDbAccess)?;
        // 指定セッションIDの有無を確認する。
        let sql_find_sess = "select count(*) as cnt from sessions where id = ?;";
        let sess_cnt: i64 = query(sql_find_sess)
            .bind(sess.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(DbError::FailDbAccess)?
            .get("cnt");
        if sess_cnt == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
