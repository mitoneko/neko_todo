use super::*;
use crate::config::ItemSortOrder;
use chrono::Local;
use sqlx::MySqlPool;
use uuid::Uuid;

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
        .get_todo_list(sess, true, ItemSortOrder::EndAsc)
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
        .get_todo_list(sess, true, ItemSortOrder::EndAsc)
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
        .get_todo_list(sess, true, ItemSortOrder::EndAsc)
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

    let items = todo
        .get_todo_list(sess, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    let item = items
        .iter()
        .find(|&i| i.title.contains("1件目"))
        .expect("「1件目」を含むアイテムは必ずあるはず");
    assert!(!item.done, "まだ、未完了のはずです。");
    let id = item.id;
    todo.change_done(id, sess, true)
        .await
        .expect("状態更新に失敗。あってはならない。");
    let items = todo
        .get_todo_list(sess, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    assert_eq!(
        items.len(),
        2,
        "一件完了済みにしたので、このリストは2件しかない。"
    );
    let items = todo
        .get_todo_list(sess, false, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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

    let items = todo
        .get_todo_list(sess, false, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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
        .get_todo_list(sess, false, ItemSortOrder::EndAsc)
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
