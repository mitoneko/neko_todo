//! データベースの操作を司る

use log::error;
use sqlx::{
    mysql::{MySqlPool, MySqlPoolOptions},
    prelude::*,
    query, query_as,
};
use thiserror::Error;
use uuid::Uuid;

/// neko_dbデータベース操作関数郡
#[derive(Clone, Debug)]
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    /// 新規生成。
    pub async fn new(host: &str, user: &str, pass: &str) -> Result<Self, DbError> {
        let db_url = format!("mariadb://{}:{}@{}/nekotodo", user, pass, host);
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .min_connections(3)
            .connect(&db_url)
            .await
            .map_err(DbError::FailConnect)?;
        Ok(Self { pool })
    }

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

#[derive(FromRow, Debug)]
pub struct User {
    pub name: String,
    pub password: String,
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
}

#[cfg(test)]
mod test {
    use super::*;

    /// テスト用のDatabase生成。テスト用Poolをインジェクション
    impl Database {
        pub(crate) fn new_test(pool: MySqlPool) -> Self {
            Self { pool }
        }
    }

    /// ユーザー生成のテスト
    #[sqlx::test]
    async fn test_add_user_and_get_user(pool: MySqlPool) {
        let db = Database::new_test(pool);
        db.add_user("hyara", "password").await.unwrap();
        let user = db.get_user("hyara").await.unwrap();
        assert_eq!(user.name, "hyara");
        assert_eq!(user.password, "password");
        let error_user = db.get_user("naiyo").await;
        match error_user {
            Ok(_) => assert!(false, "結果が帰ってくるはずがない。"),
            Err(DbError::NotFoundUser) => assert!(true),
            Err(e) => assert!(false, "このエラーはおかしい。{e}"),
        }
    }

    /// セッション生成関係の一連のテスト。
    #[sqlx::test]
    async fn test_make_new_session(pool: MySqlPool) {
        println!("まずはテスト用のユーザーの生成");
        let db = Database::new_test(pool);
        let user_name = "nekodayo";
        let password = "password";
        db.add_user(user_name, password).await.unwrap();

        println!("次に、普通にセッションを作ってみる。");
        let sess1 = db.make_new_session(user_name).await.unwrap();
        println!("セッション生成成功 id=[{}]", sess1.to_string());

        println!("次は、存在しないユーザーに対してセッションを生成してみる。");
        let sess2 = db.make_new_session("detarame").await;
        match sess2 {
            Ok(_) => assert!(false, "このユーザーは存在しなかったはず。"),
            Err(DbError::NotFoundUser) => assert!(true),
            Err(e) => assert!(false, "このエラーもおかしい。[{}]", e),
        }

        println!("普通に、セッションを更新してみる。");
        let sess3 = db.update_session(&sess1).await.unwrap();
        assert_ne!(sess1, sess3);

        println!("ないはずのセッションを更新しようとしてみる。");
        let sess4 = Uuid::now_v7();
        let sess5 = db.update_session(&sess4).await;
        match sess5 {
            Ok(_) => assert!(false, "このセッションはないはずなのに。"),
            Err(DbError::NotFoundSession) => assert!(true),
            Err(e) => assert!(false, "セッション更新2回め。失敗するにしてもこれはない{e}"),
        }
    }

    /// セッションが有効かどうかを確認するテスト
    #[sqlx::test]
    async fn test_is_session_valid(pool: MySqlPool) {
        let db = Database::new_test(pool);

        println!("テスト用ユーザーの作成");
        let name = "nekodayo";
        let pass = "nekodamon";
        db.add_user(name, pass).await.unwrap();

        println!("新規セッションを生成する。");
        let sess = db.make_new_session(name).await.unwrap();
        println!("生成したセッションIDは、[{}]です。", &sess);

        println!("今作ったセッションIDの妥当性を問い合わせてみる。");
        assert!(db.is_session_valid(&sess).await.unwrap());

        println!("偽セッションIDをいれて、問い合わせてみる。");
        assert!(!db.is_session_valid(&Uuid::now_v7()).await.unwrap());
    }
}
