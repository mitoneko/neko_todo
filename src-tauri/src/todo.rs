use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Days, Local, NaiveDate};
use log::error;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::database::*;

/// todo一件の内容
#[derive(Serialize)]
pub struct TodoItem {
    title: String,
    work: String,
    update: NaiveDate,
    start: NaiveDate,
    end: NaiveDate,
    done: bool,
}

/// todoリストの処理全般
pub struct Todo {
    database: Database,
}

impl Todo {
    /// 初期化
    pub async fn new(host: &str, user: &str, pass: &str) -> Result<Self, TodoError> {
        let db = Database::new(host, user, pass).await.map_err(|e| match e {
            DbError::FailConnect(e2) => TodoError::DbInit(e2),
            e => unimplemented!("[ToDo::new] Database::new()[{e}]"),
        })?;
        Ok(Self { database: db })
    }

    /// todoの一覧を取得する。(仮実装。インターフェース未確定)
    pub fn get_todo_list(&self) -> Result<Vec<TodoItem>, String> {
        let now_date = Local::now().naive_local().date();
        let todo = TodoItem {
            title: "テスト1".to_string(),
            work: "なにしようかな".to_string(),
            update: now_date,
            start: now_date + Days::new(1),
            end: now_date + Days::new(5),
            done: false,
        };
        let todo2 = TodoItem {
            title: "テスト2".to_string(),
            work: "こんどはなにしよう。".to_string(),
            update: now_date + Days::new(1),
            start: now_date + Days::new(5),
            end: now_date + Days::new(20),
            done: true,
        };
        let ret = vec![todo, todo2];
        Ok(ret)
    }

    /// ユーザーの追加を行う。
    pub async fn add_user(&self, name: &str, password: &str) -> Result<(), TodoError> {
        let hashed_pass = hash(password, DEFAULT_COST)?;
        if let Err(e) = self.database.add_user(name, &hashed_pass).await {
            match e {
                DbError::DuplicateUserName(e) => return Err(TodoError::DuplicateUser(e)),
                DbError::FailDbAccess(e) => {
                    error!("[Todo::add_user]Database::add_user:[{e}]");
                    return Err(TodoError::AddUser(e));
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// ログイン処理を行う。
    pub async fn login(&self, name: &str, password: &str) -> Result<Uuid, TodoError> {
        // 認証
        let user = self.database.get_user(name).await.map_err(|e| match e {
            DbError::NotFoundUser => TodoError::NotFoundUser,
            DbError::FailDbAccess(e) => TodoError::FailLogin(e),
            e => unimplemented!("[ToDo::login] Database::get_user:[{e}]"),
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
                DbError::FailDbAccess(e) => TodoError::FailLogin(e),
                e => {
                    unimplemented!("[Todo::login] Database::make_new_session:[{e}]")
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
                DbError::FailDbAccess(e) => TodoError::CheckValidSession(e),
                e => {
                    unimplemented!("[Todo::is_valid_session]is_session_valid:[{e}]")
                }
            })?;
        if is_valid {
            match self.database.update_session(sess).await {
                Ok(s) => Ok(Some(s)),
                Err(DbError::NotFoundSession) => Ok(None),
                Err(DbError::FailDbAccess(e)) => Err(TodoError::CheckValidSession(e)),
                Err(e) => {
                    unimplemented!("[Todo::is_valid_session]update_session:[{e}]")
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("データベース初期化失敗")]
    DbInit(sqlx::Error),
    #[error("新規ユーザーの登録に失敗")]
    AddUser(sqlx::Error),
    #[error("すでに、このユーザー名は使用されています。")]
    DuplicateUser(sqlx::Error),
    #[error("ユーザーパスワードのハッシュに失敗。")]
    HashUserPassword(#[from] bcrypt::BcryptError),
    #[error("ユーザーが見つかりません。")]
    NotFoundUser,
    #[error("パスワードが違います。")]
    WrongPassword,
    #[error("ログイン失敗(汎用)")]
    FailLogin(sqlx::Error),
    #[error("[is_valid_session]データベースアクセスに失敗")]
    CheckValidSession(sqlx::Error),
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::MySqlPool;

    impl Todo {
        fn test_new(pool: MySqlPool) -> Self {
            Self {
                database: Database::new_test(pool),
            }
        }
    }

    #[sqlx::test]
    async fn new_user_and_login(pool: MySqlPool) {
        let todo = Todo::test_new(pool);
        // ユーザー生成
        let user_name = "testdayo";
        let user_pass = "passnano";
        todo.add_user(user_name, user_pass).await.unwrap();

        // 正しいユーザーでログイン
        let _sess = todo.login(user_name, user_pass).await.unwrap();

        // 間違ったユーザー名でログイン
        let res = todo.login("detarame", user_pass).await;
        match res {
            Ok(_) => assert!(false, "こんなユーザーいないのに、なんでログインできたの?"),
            Err(TodoError::NotFoundUser) => {}
            Err(e) => assert!(false, "おなしなエラーが帰ってきた。{e}"),
        }

        // 間違ったパスワードでログイン
        let res = todo.login(user_name, "detarame").await;
        match res {
            Ok(_) => assert!(false, "間違ったパスワードでログインできちゃだめ"),
            Err(TodoError::WrongPassword) => {}
            Err(e) => assert!(false, "こんなえらーだめです。{e}"),
        }
    }

    #[sqlx::test]
    async fn is_valid_session_test(pool: MySqlPool) {
        let todo = Todo::test_new(pool);

        // テスト用ユーザーの生成及び、ログイン
        let user_name = "testdayo";
        let user_pass = "passwordnano";

        todo.add_user(user_name, &user_pass).await.unwrap();
        let sess = todo.login(user_name, user_pass).await.unwrap();

        // 正しいセッションを検索する。
        let new_sess = todo.is_valid_session(&sess).await.unwrap();
        match new_sess {
            Some(s) => assert_ne!(s, sess, "ログイン後のセッションが更新されていない。"),
            None => assert!(false, "正しいセッションが見つからなかった。"),
        };

        // 間違ったセッションを検索する。
        let none_sess = todo.is_valid_session(&Uuid::now_v7()).await.unwrap();
        match none_sess {
            Some(_) => assert!(false, "こんなセッションがあるわけがない。"),
            None => {}
        }
    }
}
