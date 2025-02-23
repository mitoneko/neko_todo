//! databaseモジュールテスト

use crate::config::ItemSortOrder;
use chrono::{Days, Local};
use sqlx::query;
use uuid::Uuid;

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
    let res = db
        .get_todo_item(sess, last_day, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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
    let res = db
        .get_todo_item(sess, last_day, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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
    let res = db
        .get_todo_item(sess, last_day, false, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    assert_eq!(res.len(), 1, "全部読み出しだけど一件あるはず。");
    let res = db
        .get_todo_item(sess, last_day, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    assert_eq!(res.len(), 1, "未完了だけだけど、一件あるはず。");

    println!("今作ったjobを完了済みにする。");
    let sql = "update todo set done=true where id=?;";
    query(sql).bind(res[0].id).execute(&pool).await.unwrap();
    let res = db
        .get_todo_item(sess, last_day, false, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    assert_eq!(res.len(), 1, "全部読み出しだけど一件あるはず。");
    let res = db
        .get_todo_item(sess, last_day, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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
    let res = db
        .get_todo_item(sess, ref_date, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    assert_eq!(res.len(), 1, "基準日と開始日が同じだからみつかる。");
    let res = db
        .get_todo_item(sess, ref_date + Days::new(1), true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    assert_eq!(res.len(), 1, "開始日の翌日が基準日だからみつかる。");
    let res = db
        .get_todo_item(sess, ref_date - Days::new(1), true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    assert_eq!(res.len(), 0, "基準日が開始日の前日だからみつからない。");
    let res = db
        .get_todo_item(sess, ref_date + Days::new(4), true, ItemSortOrder::EndAsc)
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

    let items = db
        .get_todo_item(sess, ref_date, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    let item = items.iter().find(|&i| i.title.contains("二件目")).unwrap();
    db.change_done(item.id, true).await.unwrap();

    let items = db
        .get_todo_item(sess, ref_date, true, ItemSortOrder::EndAsc)
        .await
        .unwrap();
    let item = items.iter().find(|&i| i.title.contains("二件目"));
    assert!(item.is_none(), "状態を完了にしたので見つからないはず。");

    let items = db
        .get_todo_item(sess, ref_date, false, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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
        .get_todo_item(
            sess,
            Local::now().date_naive(),
            false,
            ItemSortOrder::EndAsc,
        )
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
    let items = db
        .get_todo_item(sess, today, false, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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
    let items_new = db
        .get_todo_item(sess, today, false, ItemSortOrder::EndAsc)
        .await
        .unwrap();
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

#[sqlx::test]
async fn test_sort_end_date(pool: MySqlPool) {
    let db = Database::new_test(pool);
    let sess = login_for_test(&db).await;
    create_todo_for_test(&db, sess).await;

    let today = Local::now().date_naive();
    let recs = db
        .get_todo_item(sess, today, false, ItemSortOrder::EndAsc)
        .await
        .expect("取得時にエラーを起こした。");
    eprintln!("取得データ(昇順)");
    eprintln!("0 => {:?}", recs[0]);
    eprintln!("1 => {:?}", recs[1]);
    eprintln!("2 => {:?}", recs[2]);
    assert!(
        recs[0].end_date <= recs[1].end_date,
        "終了日が昇順になってない。"
    );
    assert!(
        recs[1].end_date <= recs[2].end_date,
        "終了日が昇順になってない(2)。"
    );

    let recs = db
        .get_todo_item(sess, today, false, ItemSortOrder::EndDesc)
        .await
        .expect("取得時にエラーを起こした(2)");
    eprintln!("取得データ(降順)");
    eprintln!("0 => {:?}", recs[0]);
    eprintln!("1 => {:?}", recs[1]);
    eprintln!("2 => {:?}", recs[2]);
    assert!(
        recs[0].end_date >= recs[1].end_date,
        "終了日が降順になってない(1)"
    );
    assert!(
        recs[1].end_date >= recs[2].end_date,
        "終了日が降順になってない(2)"
    );
}

#[sqlx::test]
async fn test_sort_start_date(pool: MySqlPool) {
    let db = Database::new_test(pool);
    let sess = login_for_test(&db).await;
    create_todo_for_test(&db, sess).await;

    let today = Local::now().date_naive();
    let recs = db
        .get_todo_item(sess, today, false, ItemSortOrder::StartAsc)
        .await
        .expect("取得時にエラーを起こした。");
    assert!(
        recs[0].start_date <= recs[1].start_date,
        "開始日が昇順になってない。"
    );
    assert!(
        recs[1].start_date <= recs[2].start_date,
        "開始日が昇順になってない(2)。"
    );

    let recs = db
        .get_todo_item(sess, today, false, ItemSortOrder::StartDesc)
        .await
        .expect("取得時にエラーを起こした(2)");
    assert!(
        recs[0].start_date >= recs[1].start_date,
        "開始日が降順になってない(1)"
    );
    assert!(
        recs[1].start_date >= recs[2].start_date,
        "開始日が降順になってない(2)"
    );
}

#[sqlx::test]
async fn test_sort_update_date(pool: MySqlPool) {
    let db = Database::new_test(pool);
    let sess = login_for_test(&db).await;
    create_todo_for_test(&db, sess).await;
    let today = Local::now().date_naive();

    // Databaseのインターフェースでupdate_dateを更新するすべはないので直接編集
    let keys = db
        .get_todo_item(sess, today, false, ItemSortOrder::EndAsc)
        .await
        .unwrap()
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let sql = "update todo set update_date = ? where id = ?";
    let days = [
        today + Days::new(2),
        today + Days::new(1),
        today + Days::new(3),
    ];
    for i in 0..3 {
        query(sql)
            .bind(days[i])
            .bind(keys[i])
            .execute(&db.pool)
            .await
            .unwrap();
    }

    let recs = db
        .get_todo_item(sess, today, false, ItemSortOrder::UpdateAsc)
        .await
        .expect("取得時にエラーを起こした。");
    assert!(
        recs[0].update_date <= recs[1].update_date,
        "更新日が昇順になってない。"
    );
    assert!(
        recs[1].update_date <= recs[2].update_date,
        "更新日が昇順になってない(2)。"
    );

    let recs = db
        .get_todo_item(sess, today, false, ItemSortOrder::UpdateDesc)
        .await
        .expect("取得時にエラーを起こした(2)");
    assert!(
        recs[0].update_date >= recs[1].update_date,
        "更新日が降順になってない(1)"
    );
    assert!(
        recs[1].update_date >= recs[2].update_date,
        "更新日が降順になってない(2)"
    );
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
        start_date: Some(Local::now().date_naive() - Days::new(4)),
        end_date: Some(Local::now().date_naive() + Days::new(2)),
        done: false,
    };
    db.add_todo_item(&item).await.unwrap();

    let item = ItemTodo {
        id: 0,
        user_name: name.to_string(),
        title: "二件目(work無し)".to_string(),
        work: None,
        update_date: None,
        start_date: Some(Local::now().date_naive() - Days::new(5)),
        end_date: Some(Local::now().date_naive() + Days::new(1)),
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
