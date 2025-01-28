//! データベースの操作を司る

use chrono::{Local, NaiveDate};
use log::error;
use serde::{Deserialize, Serialize};
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

    /// Todo項目を追加する。
    /// item引数のうち、id, update_date, doneは、無視される
    /// 各々、自動値・今日の日付・falseがはいる。
    /// start_date, end_dateのデフォルト値は、今日・NaiveDate::MAXである。
    pub async fn add_todo_item(&self, item: &ItemTodo) -> Result<(), DbError> {
        let sql = r#"
            insert into todo(user_name, title, work, update_date, start_date, end_date, done)
            values (?, ?, ?, curdate(), ?, ?, false);
        "#;
        let start_date = item.start_date.unwrap_or(Local::now().date_naive());
        let end_date = item
            .end_date
            .unwrap_or(NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
        query(sql)
            .bind(&item.user_name)
            .bind(&item.title)
            .bind(&item.work)
            .bind(start_date)
            .bind(end_date)
            .execute(&self.pool)
            .await
            .map_err(DbError::FailDbAccess)?;
        Ok(())
    }

    /// Todoの一覧を取得する。
    /// 基準日(ref_date)以降のアイテムを選別する。
    /// セッションIDを必要とする。
    /// 検索オプションのとり方は未確定。インターフェース変更の可能性大。
    pub async fn get_todo_item(
        &self,
        sess: Uuid,
        ref_date: NaiveDate,
        only_incomplete: bool,
    ) -> Result<Vec<ItemTodo>, DbError> {
        let sql1 = r#"
            select t.id, t.user_name, title, work, update_date, start_date, end_date, done 
            from todo t join sessions s on s.user_name = t.user_name 
            where s.id=? and t.start_date <= ? 
            "#;
        let sql2 = " and done = false";
        let sql = if only_incomplete {
            format!("{} {};", sql1, sql2)
        } else {
            format!("{} ;", sql1)
        };
        let items = query_as::<_, ItemTodo>(&sql)
            .bind(sess.to_string())
            .bind(ref_date)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::FailDbAccess)?;

        Ok(items)
    }

    /// 指定idのTodo項目を取得する。
    /// 有効なセッションが指定されていなければ、未発見とする。
    pub async fn get_todo_item_with_id(&self, id: u32, sess: Uuid) -> Result<ItemTodo, DbError> {
        let sql = r#"
            select t.id, t.user_name, t.title, t.work, t.update_date, t.start_date, t.end_date, t.done 
            from todo t join sessions s on s.user_name = t.user_name 
            where s.id=? and t.id=?
            "#;
        query_as::<_, ItemTodo>(sql)
            .bind(sess.to_string())
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => DbError::NotFoundTodo,
                e => DbError::FailDbAccess(e),
            })
    }

    /// Todoの完了状態を更新する。
    pub async fn change_done(&self, id: u32, done: bool) -> Result<(), DbError> {
        let sql = "update todo set done = ? where id = ?";
        let res = query(sql)
            .bind(done)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(DbError::FailDbAccess)?;
        if res.rows_affected() > 0 {
            Ok(())
        } else {
            Err(DbError::NotFoundTodo)
        }
    }

    /// Todoの項目編集
    pub async fn edit_todo(&self, item: &ItemTodo) -> Result<(), DbError> {
        let start_date = item.start_date.unwrap_or(Local::now().date_naive());
        let end_date = item
            .end_date
            .unwrap_or(NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());

        let sql = r#"
            update todo 
            set title=?, work=?, update_date=curdate(), start_date=?, end_date=? 
            where id=?;
            "#;
        let res = query(sql)
            .bind(&item.title)
            .bind(&item.work)
            .bind(start_date)
            .bind(end_date)
            .bind(item.id)
            .execute(&self.pool)
            .await
            .map_err(DbError::FailDbAccess)?;
        if res.rows_affected() > 0 {
            Ok(())
        } else {
            Err(DbError::NotFoundTodo)
        }
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

#[cfg(test)]
mod test {
    use chrono::Days;

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
            Ok(_) => unreachable!("結果が帰ってくるはずがない。"),
            Err(DbError::NotFoundUser) => { /* 正常 */ }
            Err(e) => unreachable!("このエラーはおかしい。{e}"),
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
        println!("セッション生成成功 id=[{}]", sess1);

        println!("次は、存在しないユーザーに対してセッションを生成してみる。");
        let sess2 = db.make_new_session("detarame").await;
        match sess2 {
            Ok(_) => unreachable!("このユーザーは存在しなかったはず。"),
            Err(DbError::NotFoundUser) => { /* 正常 */ }
            Err(e) => unreachable!("このエラーもおかしい。[{}]", e),
        }

        println!("普通に、セッションを更新してみる。");
        let sess3 = db.update_session(&sess1).await.unwrap();
        assert_ne!(sess1, sess3);

        println!("ないはずのセッションを更新しようとしてみる。");
        let sess4 = Uuid::now_v7();
        let sess5 = db.update_session(&sess4).await;
        match sess5 {
            Ok(_) => unreachable!("このセッションはないはずなのに。"),
            Err(DbError::NotFoundSession) => { /* 正常 */ }
            Err(e) => unreachable!("セッション更新2回め。失敗するにしてもこれはない{e}"),
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

    /// todoの書き込みと、単純な読み出しのテスト
    #[sqlx::test]
    async fn test_add_todo(pool: MySqlPool) {
        let db = Database::new_test(pool);
        let sess = login_for_test(&db).await;
        let name = db.get_user_from_sess(sess).await.unwrap().name;

        println!("テストデータをインサート");
        let mut item = ItemTodo {
            id: 0,
            user_name: name.to_string(),
            title: "インサートできるかな?".to_string(),
            work: Some("中身入り".to_string()),
            update_date: None,
            start_date: Some(Local::now().date_naive()),
            end_date: Some(Local::now().date_naive() + Days::new(3)),
            done: true,
        };
        db.add_todo_item(&item).await.unwrap();

        println!("テストデータを読み出す。一件しかないはず");
        let last_day = Local::now().date_naive() + Days::new(1);
        let res = db.get_todo_item(sess, last_day, true).await.unwrap();
        assert_eq!(res.len(), 1, "あれ?一件のはずだよ");
        item.id = res[0].id;
        item.update_date = Some(Local::now().date_naive());
        item.done = false;
    }

    /// todoの書き込みと読み出し。
    /// workが未入力の場合。
    #[sqlx::test]
    async fn test_add_todo_without_work(pool: MySqlPool) {
        let db = Database::new_test(pool);
        let sess = login_for_test(&db).await;
        let name = db.get_user_from_sess(sess).await.unwrap().name;

        println!("テストデータをインサート");
        let mut item = ItemTodo {
            id: 0,
            user_name: name.to_string(),
            title: "インサートできるかな?".to_string(),
            work: None,
            update_date: None,
            start_date: Some(Local::now().date_naive()),
            end_date: Some(Local::now().date_naive() + Days::new(3)),
            done: true,
        };
        db.add_todo_item(&item).await.unwrap();

        println!("テストデータを読み出す。一件しかないはず");
        let last_day = Local::now().date_naive() + Days::new(1);
        let res = db.get_todo_item(sess, last_day, true).await.unwrap();
        assert_eq!(res.len(), 1, "あれ?一件のはずだよ");
        item.id = res[0].id;
        item.update_date = Some(Local::now().date_naive());
        item.done = false;
    }

    /// todoの書き込みと読み出し
    /// done=trueとfalseの挙動テスト
    #[sqlx::test]
    async fn test_get_todo_done_param(pool: MySqlPool) {
        let db = Database::new_test(pool.clone());
        let sess = login_for_test(&db).await;
        let name = db.get_user_from_sess(sess).await.unwrap().name;

        println!("テストデータをインサート");
        let item = ItemTodo {
            id: 0,
            user_name: name.to_string(),
            title: "インサートできるかな?".to_string(),
            work: None,
            update_date: None,
            start_date: Some(Local::now().date_naive()),
            end_date: Some(Local::now().date_naive() + Days::new(3)),
            done: true,
        };
        db.add_todo_item(&item).await.unwrap();

        println!("テストデータを読み出す。一件しかないはず");
        let last_day = Local::now().date_naive() + Days::new(1);
        let res = db.get_todo_item(sess, last_day, false).await.unwrap();
        assert_eq!(res.len(), 1, "全部読み出しだけど一件あるはず。");
        let res = db.get_todo_item(sess, last_day, true).await.unwrap();
        assert_eq!(res.len(), 1, "未完了だけだけど、一件あるはず。");

        println!("今作ったjobを完了済みにする。");
        let sql = "update todo set done=true where id=?;";
        query(sql).bind(res[0].id).execute(&pool).await.unwrap();
        let res = db.get_todo_item(sess, last_day, false).await.unwrap();
        assert_eq!(res.len(), 1, "全部読み出しだけど一件あるはず。");
        let res = db.get_todo_item(sess, last_day, true).await.unwrap();
        assert_eq!(res.len(), 0, "未完了だけだけだから、なにもないはず。");
    }

    /// todoの書き込みと読み出し
    /// 基準日の挙動テスト
    #[sqlx::test]
    async fn test_get_todo_ref_date(pool: MySqlPool) {
        let db = Database::new_test(pool.clone());
        let sess = login_for_test(&db).await;
        let name = db.get_user_from_sess(sess).await.unwrap().name;

        println!("テストデータをインサート");
        let item = ItemTodo {
            id: 0,
            user_name: name.to_string(),
            title: "インサートできるかな?".to_string(),
            work: None,
            update_date: None,
            start_date: Some(Local::now().date_naive()),
            end_date: Some(Local::now().date_naive() + Days::new(3)),
            done: false,
        };
        db.add_todo_item(&item).await.unwrap();

        let ref_date = Local::now().date_naive();
        let res = db.get_todo_item(sess, ref_date, true).await.unwrap();
        assert_eq!(res.len(), 1, "基準日と開始日が同じだからみつかる。");
        let res = db
            .get_todo_item(sess, ref_date + Days::new(1), true)
            .await
            .unwrap();
        assert_eq!(res.len(), 1, "開始日の翌日が基準日だからみつかる。");
        let res = db
            .get_todo_item(sess, ref_date - Days::new(1), true)
            .await
            .unwrap();
        assert_eq!(res.len(), 0, "基準日が開始日の前日だからみつからない。");
        let res = db
            .get_todo_item(sess, ref_date + Days::new(4), true)
            .await
            .unwrap();
        assert_eq!(res.len(), 1, "基準日が期限を過ぎているけどみつかるの。");
    }

    #[sqlx::test]
    async fn test_get_user_from_sess(pool: MySqlPool) {
        let db = Database::new_test(pool.clone());

        let sess = login_for_test(&db).await;
        let name = db.get_user_from_sess(sess).await.unwrap().name;

        let user = db.get_user_from_sess(sess).await.unwrap();
        assert_eq!(user.name, name, "これはみつかるはず");
        let dummy_sess = Uuid::now_v7();
        let user = db.get_user_from_sess(dummy_sess).await;
        match user {
            Ok(_) => unreachable!("見つかるわけないでしょう。"),
            Err(DbError::NotFoundSession) => { /* 正常 */ }
            Err(e) => unreachable!("トラブルです。{e}"),
        };
    }

    #[sqlx::test]
    async fn test_change_done(pool: MySqlPool) {
        let db = Database::new_test(pool);
        let sess = login_for_test(&db).await;
        let ref_date = Local::now().date_naive();
        create_todo_for_test(&db, sess).await;

        let items = db.get_todo_item(sess, ref_date, true).await.unwrap();
        let item = items.iter().find(|&i| i.title.contains("二件目")).unwrap();
        db.change_done(item.id, true).await.unwrap();

        let items = db.get_todo_item(sess, ref_date, true).await.unwrap();
        let item = items.iter().find(|&i| i.title.contains("二件目"));
        assert!(item.is_none(), "状態を完了にしたので見つからないはず。");

        let items = db.get_todo_item(sess, ref_date, false).await.unwrap();
        let item = items.iter().find(|&i| i.title.contains("二件目"));
        match item {
            Some(i) => assert!(i.done, "完了済みになっているはずですね?"),
            None => unreachable!("状態を変えたら、レコードなくなった???"),
        }
        assert_eq!(
            items.len(),
            3,
            "全件見ているのでレコードは3件あるはずですが?"
        );
    }

    #[sqlx::test]
    async fn test_get_todo_with_id(pool: MySqlPool) {
        let db = Database::new_test(pool);
        let sess = login_for_test(&db).await;
        create_todo_for_test(&db, sess).await;

        let items = db
            .get_todo_item(sess, Local::now().date_naive(), false)
            .await
            .unwrap();
        let id = items
            .iter()
            .find(|&i| i.title.contains("一件目"))
            .expect("これはあるはず")
            .id;
        let non_exist_id = items.iter().max_by_key(|&i| i.id).unwrap().id + 1;

        // 正常な読み出し
        let res = db
            .get_todo_item_with_id(id, sess)
            .await
            .expect("これは正常に読み出せるはず。エラーはだめ");
        res.work
            .expect("このレーコードはworkを持つはずです。")
            .find("働いてます。")
            .expect("workの内容がおかしい。");

        // 間違ったid
        let res = db.get_todo_item_with_id(non_exist_id, sess).await;
        match res {
            Ok(_) => unreachable!("そんなIDは存在しなかったはずなのに。"),
            Err(DbError::NotFoundTodo) => { /* 正常 */ }
            Err(e) => unreachable!("データベースエラーだよ。({e})"),
        }

        // 間違ったセッション
        let res = db.get_todo_item_with_id(id, Uuid::now_v7()).await;
        match res {
            Ok(_) => unreachable!("そんなセッションはないはず。"),
            Err(DbError::NotFoundTodo) => { /* 正常 */ }
            Err(e) => unreachable!("データベースエラー発生。({e})"),
        }
    }

    #[sqlx::test]
    async fn test_edit(pool: MySqlPool) {
        let db = Database::new_test(pool);
        let sess = login_for_test(&db).await;
        create_todo_for_test(&db, sess).await;

        // 書き込みテスト用レコードの取得
        let today = Local::now().date_naive();
        let items = db.get_todo_item(sess, today, false).await.unwrap();
        let mut item = items
            .iter()
            .find(|&i| i.title.contains("一件目"))
            .expect("ないはずがない。")
            .clone();
        item.title = "更新しました。".to_string();
        item.work = Some("書き換え後".to_string());
        item.start_date = Some(today - Days::new(5));
        item.end_date = Some(today + Days::new(10));
        db.edit_todo(&item).await.expect("更新がエラーを起こした。");
        // 書き込み後の照合
        let items_new = db.get_todo_item(sess, today, false).await.unwrap();
        let item_new = items_new
            .iter()
            .find(|&i| i.title.contains("更新しました。"))
            .expect("更新されたレコードが存在しない。");
        assert_eq!(
            item_new.work,
            Some("書き換え後".to_string()),
            "更新後のworkがおかしい"
        );
        assert_eq!(
            item_new.start_date,
            Some(today - Days::new(5)),
            "更新後のstart_dateがおかしい"
        );
        assert_eq!(
            item_new.end_date,
            Some(today + Days::new(10)),
            "更新後のend_dateがおかしい"
        );

        // 存在しないレコードの更新
        let id_max_plus_one = items.iter().max_by_key(|&i| i.id).unwrap().id + 1;
        item.id = id_max_plus_one;
        let res = db.edit_todo(&item).await;
        match res {
            Ok(_) => unreachable!("更新できちゃだめっ"),
            Err(DbError::NotFoundTodo) => {}
            Err(e) => unreachable!("db_err: {e}"),
        }
    }
    async fn login_for_test(db: &Database) -> Uuid {
        println!("テスト用ユーザー及びセッションの生成");
        let name = "test";
        let pass = "test";
        db.add_user(name, pass).await.unwrap();
        db.make_new_session(name).await.unwrap()
    }

    async fn create_todo_for_test(db: &Database, sess: Uuid) {
        let name = db.get_user_from_sess(sess).await.unwrap().name;

        println!("テストデータをインサート");
        let item = ItemTodo {
            id: 0,
            user_name: name.to_string(),
            title: "一件目(work有り)".to_string(),
            work: Some("働いてます。".to_string()),
            update_date: None,
            start_date: Some(Local::now().date_naive()),
            end_date: Some(Local::now().date_naive() + Days::new(3)),
            done: false,
        };
        db.add_todo_item(&item).await.unwrap();

        let item = ItemTodo {
            id: 0,
            user_name: name.to_string(),
            title: "二件目(work無し)".to_string(),
            work: None,
            update_date: None,
            start_date: Some(Local::now().date_naive()),
            end_date: Some(Local::now().date_naive() + Days::new(3)),
            done: false,
        };
        db.add_todo_item(&item).await.unwrap();

        let item = ItemTodo {
            id: 0,
            user_name: name.to_string(),
            title: "三件目(work無し)".to_string(),
            work: None,
            update_date: None,
            start_date: Some(Local::now().date_naive()),
            end_date: Some(Local::now().date_naive() + Days::new(3)),
            done: false,
        };
        db.add_todo_item(&item).await.unwrap();
    }
}
