use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Local;
use log::error;
use thiserror::Error;
use uuid::Uuid;

use crate::database::*;

/// todoリストの処理全般
pub struct Todo {
    database: Database,
}

impl Todo {
    /// 初期化
    pub async fn new(host: &str, user: &str, pass: &str) -> Result<Self, TodoError> {
        let db = Database::new(host, user, pass).await.map_err(|e| match e {
            DbError::FailConnect(e2) => TodoError::DbInit(e2),
            e => unreachable!("[ToDo::new] Database::new()[{e}]"),
        })?;
        Ok(Self { database: db })
    }

    /// todoの一覧を取得する。(仮実装。インターフェース未確定)
    pub async fn get_todo_list(
        &self,
        sess: Uuid,
        only_imcomplete: bool,
    ) -> Result<Vec<ItemTodo>, TodoError> {
        let ref_date = Local::now().date_naive();
        self.database
            .get_todo_item(sess, ref_date, only_imcomplete)
            .await
            .map_err(|e| match e {
                DbError::FailDbAccess(e) => TodoError::FailDbAccess(e),
                e => unreachable!("[get_todo_list]get_todo_item[{e}]"),
            })
    }

    /// 新規のtodoを追加する
    /// 引数itemのid, user_name, update_date, update_dateは無視される。
    pub async fn add_todo(&self, sess: Uuid, item: &ItemTodo) -> Result<(), TodoError> {
        // ユーザー名を取得
        let user = self
            .database
            .get_user_from_sess(sess)
            .await
            .map_err(|e| match e {
                DbError::NotFoundSession => TodoError::NotFoundSession,
                DbError::FailDbAccess(e) => {
                    error!("[Todo::add_todo]get_user_from_sess:[{e}]");
                    TodoError::FailDbAccess(e)
                }
                e => unreachable!("[add_todo]get_user_from_sess[{e}]"),
            })?;
        // アイテムを登録
        let mut item = item.clone();
        item.user_name = user.name.clone();
        if let Some(ref s) = item.work {
            if s.trim().is_empty() {
                item.work = None;
            }
        }
        self.database
            .add_todo_item(&item)
            .await
            .map_err(|e| match e {
                DbError::FailDbAccess(e) => {
                    error!("[Todo::add_todo]add_todo_item:[{e}]");
                    TodoError::FailDbAccess(e)
                }
                e => unreachable!("[add_todo]add_todo_item[{e}]"),
            })
    }

    /// idとsessを指定してtodoを取得する。
    /// 一致するtodoがなければ、エラー、TodoError::NotFoundTodoを返す。
    pub async fn get_todo_with_id(&self, id: u32, sess: Uuid) -> Result<ItemTodo, TodoError> {
        self.database
            .get_todo_item_with_id(id, sess)
            .await
            .map_err(|e| match e {
                DbError::NotFoundTodo => TodoError::NotFoundTodo,
                DbError::FailDbAccess(e) => {
                    error!("[Todo::get_todo_with_id]get_todo_item_with_id:[{e}])");
                    TodoError::FailDbAccess(e)
                }
                e => unreachable!("[Todo::get_todo_with_id]get_todo_item_with_id[{e}]"),
            })
    }

    /// Todoの完了状態を変更する
    pub async fn change_done(&self, id: u32, sess: Uuid, done: bool) -> Result<(), TodoError> {
        self.get_todo_with_id(id, sess).await?;
        self.database
            .change_done(id, done)
            .await
            .map_err(|e| match e {
                DbError::FailDbAccess(e) => {
                    error!("[Todo::change_done]change_done:[{e}]");
                    TodoError::FailDbAccess(e)
                }
                DbError::NotFoundTodo => TodoError::NotFoundTodo,
                e => unreachable!("[change_done]change_done[{e}]"),
            })
    }

    /// Todoの編集を行う。
    pub async fn edit_todo(&self, item: &ItemTodo, sess: Uuid) -> Result<(), TodoError> {
        let mut item = item.clone();
        if let Some(ref s) = item.work {
            if s.trim().is_empty() {
                item.work = None;
            }
        }
        self.get_todo_with_id(item.id, sess).await?;
        self.database.edit_todo(&item).await.map_err(|e| match e {
            DbError::FailDbAccess(e) => {
                error!("[Todo::edit_todo]edit_todo:[{e}]");
                TodoError::FailDbAccess(e)
            }
            DbError::NotFoundTodo => TodoError::NotFoundTodo,
            e => unreachable!("[edit_todo]edit_todo[{e}]"),
        })
    }

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
            Ok(_) => unreachable!("こんなユーザーいないのに、なんでログインできたの?"),
            Err(TodoError::NotFoundUser) => {}
            Err(e) => unreachable!("おなしなエラーが帰ってきた。{e}"),
        }

        // 間違ったパスワードでログイン
        let res = todo.login(user_name, "detarame").await;
        match res {
            Ok(_) => unreachable!("間違ったパスワードでログインできちゃだめ"),
            Err(TodoError::WrongPassword) => {}
            Err(e) => unreachable!("こんなえらーだめです。{e}"),
        }
    }

    #[sqlx::test]
    async fn is_valid_session_test(pool: MySqlPool) {
        let todo = Todo::test_new(pool);

        // テスト用ユーザーの生成及び、ログイン
        let user_name = "testdayo";
        let user_pass = "passwordnano";

        todo.add_user(user_name, user_pass).await.unwrap();
        let sess = todo.login(user_name, user_pass).await.unwrap();

        // 正しいセッションを検索する。
        let new_sess = todo.is_valid_session(&sess).await.unwrap();
        match new_sess {
            Some(s) => assert_ne!(s, sess, "ログイン後のセッションが更新されていない。"),
            None => unreachable!("正しいセッションが見つからなかった。"),
        };

        // 間違ったセッションを検索する。
        let none_sess = todo.is_valid_session(&Uuid::now_v7()).await.unwrap();
        if none_sess.is_some() {
            unreachable!("こんなセッションがあるわけがない。");
        }
    }

    #[sqlx::test]
    async fn add_todo_test(pool: MySqlPool) {
        use chrono::Days;

        let todo = Todo::test_new(pool);
        let sess = login_for_test(&todo).await;

        let item1 = ItemTodo {
            id: 100,
            user_name: "kore_naihazu".to_string(),
            title: "テストアイテム1件目".to_string(),
            work: Some("これは、中身を入れる。".to_string()),
            update_date: None,
            start_date: Some(Local::now().date_naive() - Days::new(1)),
            end_date: Some(Local::now().date_naive() + Days::new(5)),
            done: true,
        };
        let item2 = ItemTodo {
            id: 100,
            user_name: "kore_naihazu".to_string(),
            title: "テストアイテム2件目(work=null)".to_string(),
            work: Some("".to_string()),
            update_date: None,
            start_date: Some(Local::now().date_naive() - Days::new(1)),
            end_date: Some(Local::now().date_naive() + Days::new(5)),
            done: true,
        };
        let item3 = ItemTodo {
            id: 100,
            user_name: "kore_naihazu".to_string(),
            title: "テストアイテム3件目(work=space)".to_string(),
            work: Some(" \t　".to_string()),
            update_date: None,
            start_date: Some(Local::now().date_naive() - Days::new(1)),
            end_date: Some(Local::now().date_naive() + Days::new(5)),
            done: true,
        };
        todo.add_todo(sess, &item1)
            .await
            .expect("1件目の追加に失敗");
        let res = todo
            .get_todo_list(sess, true)
            .await
            .expect("1件目の取得に失敗");
        assert_eq!(res.len(), 1, "一件目が取得できなかった?");
        assert_eq!(res[0].title, item1.title, "一件目のtitleが違う");
        assert_eq!(res[0].work, item1.work, "一件目のworkが違う");
        assert_eq!(res[0].user_name, "testdayo", "一件目のuser_nameが違う");
        assert_eq!(
            res[0].update_date,
            Some(Local::now().date_naive()),
            "一件目のupdate_dateが違う"
        );
        assert_eq!(res[0].start_date, item1.start_date, "一件目の開始日が違う");
        assert_eq!(res[0].end_date, item1.end_date, "一件目の終了日が違う");
        assert!(!res[0].done, "一件目の完了マークが違う");

        todo.add_todo(sess, &item2)
            .await
            .expect("二件目の追加に失敗");
        let res = todo
            .get_todo_list(sess, true)
            .await
            .expect("二件目の取得に失敗");
        assert_eq!(res.len(), 2, "二件あるはずなんだけど");
        assert!(
            res.iter()
                .find(|&x| match x.title.find("work=null") {
                    Some(n) => n > 0,
                    None => false,
                })
                .expect("二件目に追加したデータがない")
                .work
                .is_none(),
            "二件目のworkはNoneのはず"
        );
        todo.add_todo(sess, &item3)
            .await
            .expect("三件目の追加に失敗");
        let res = todo
            .get_todo_list(sess, true)
            .await
            .expect("三件目の取得に失敗");
        assert_eq!(res.len(), 3, "三件あるはずですよ。");
        assert!(
            res.iter()
                .find(|&x| match x.title.find("work=space") {
                    Some(n) => n > 0,
                    None => false,
                })
                .expect("三件目のデータがないよ?")
                .work
                .is_none(),
            "三件目のデータはNoneに変換してくれてるはず。"
        );
    }

    #[sqlx::test]
    async fn change_done_test(pool: MySqlPool) {
        let todo = Todo::test_new(pool);
        let sess = login_for_test(&todo).await;
        create_todo_for_test(&todo, sess).await;

        let items = todo.get_todo_list(sess, true).await.unwrap();
        let item = items
            .iter()
            .find(|&i| i.title.contains("1件目"))
            .expect("「1件目」を含むアイテムは必ずあるはず");
        assert!(!item.done, "まだ、未完了のはずです。");
        let id = item.id;
        todo.change_done(id, sess, true)
            .await
            .expect("状態更新に失敗。あってはならない。");
        let items = todo.get_todo_list(sess, true).await.unwrap();
        assert_eq!(
            items.len(),
            2,
            "一件完了済みにしたので、このリストは2件しかない。"
        );
        let items = todo.get_todo_list(sess, false).await.unwrap();
        assert_eq!(items.len(), 3, "完了済みを含むので、3件になる。");
        let item = items
            .iter()
            .find(|&i| i.id == id)
            .expect("さっきあったidだから必ずある。");
        assert!(item.done, "さっき完了済みに変更した。");

        let max_id = items.iter().max_by_key(|&x| x.id).unwrap().id;
        let res = todo.change_done(max_id + 1, sess, false).await;
        match res {
            Ok(_) => unreachable!("このidのtodoがあるはずがない。"),
            Err(TodoError::NotFoundTodo) => {}
            Err(e) => unreachable!("このエラーもありえない。[{e}]"),
        };

        // 間違ったセッションのテスト
        let res = todo.change_done(id, Uuid::now_v7(), true).await;
        match res {
            Ok(_) => unreachable!("このセッションでは、更新を許してはいけない。"),
            Err(TodoError::NotFoundTodo) => { /* 正常 */ }
            Err(e) => unreachable!("このエラーもおかしい。[{e}]"),
        }
    }

    #[sqlx::test]
    async fn edit_todo_test(pool: MySqlPool) {
        let todo = Todo::test_new(pool);
        let sess = login_for_test(&todo).await;
        create_todo_for_test(&todo, sess).await;

        let items = todo.get_todo_list(sess, false).await.unwrap();
        let mut item = items
            .iter()
            .find(|&i| i.title.contains("1件目"))
            .unwrap()
            .clone();
        item.title = "更新した一件目".to_string();
        if let Err(e) = todo.edit_todo(&item, sess).await {
            unreachable!("更新処理に失敗した。[{e}]");
        }
        let Some(item_new) = todo
            .get_todo_list(sess, false)
            .await
            .unwrap()
            .iter()
            .find(|&i| i.title.contains("更新した一件目"))
            .cloned()
        else {
            unreachable!("更新したレコードが見つからないよ?");
        };
        assert_eq!(item.id, item_new.id, "更新したレコードのidが化けてる");

        // ニセセッションで試す
        match todo.edit_todo(&item, Uuid::now_v7()).await {
            Ok(_) => unreachable!("偽のセッションで更新成功してはならない。"),
            Err(TodoError::NotFoundTodo) => { /* 正常 */ }
            Err(e) => unreachable!("偽セッションのときのエラー:{e}"),
        }
    }

    async fn login_for_test(todo: &Todo) -> Uuid {
        let user_name = "testdayo";
        let user_pass = "passrordnona";
        todo.add_user(user_name, user_pass).await.unwrap();
        todo.login(user_name, user_pass).await.unwrap()
    }

    async fn create_todo_for_test(todo: &Todo, sess: Uuid) {
        use chrono::Days;
        let items = [
            ItemTodo {
                id: 100,
                user_name: "kore_naihazu".to_string(),
                title: "テストアイテム1件目".to_string(),
                work: Some("これは、中身を入れる。".to_string()),
                update_date: None,
                start_date: Some(Local::now().date_naive() - Days::new(1)),
                end_date: Some(Local::now().date_naive() + Days::new(5)),
                done: false,
            },
            ItemTodo {
                id: 100,
                user_name: "kore_naihazu".to_string(),
                title: "テストアイテム2件目(work=null)".to_string(),
                work: Some("".to_string()),
                update_date: None,
                start_date: Some(Local::now().date_naive() - Days::new(1)),
                end_date: Some(Local::now().date_naive() + Days::new(5)),
                done: false,
            },
            ItemTodo {
                id: 100,
                user_name: "kore_naihazu".to_string(),
                title: "テストアイテム3件目(work=space)".to_string(),
                work: Some(" \t　".to_string()),
                update_date: None,
                start_date: Some(Local::now().date_naive() - Days::new(1)),
                end_date: Some(Local::now().date_naive() + Days::new(5)),
                done: false,
            },
        ];
        for item in items {
            todo.add_todo(sess, &item).await.unwrap();
        }
    }
}
