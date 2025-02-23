//! todoアイテム操作
use super::*;
use crate::config::ItemSortOrder;
use chrono::{Local, NaiveDate};
use sqlx::{query, query_as};
use uuid::Uuid;

impl Database {
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
        sort_order: ItemSortOrder,
    ) -> Result<Vec<ItemTodo>, DbError> {
        let sql1 = r#"
            select t.id, t.user_name, title, work, update_date, start_date, end_date, done 
            from todo t join sessions s on s.user_name = t.user_name 
            where s.id=? and t.start_date <= ? 
            "#;
        let sql2 = " and done = false";
        let sql3 = match sort_order {
            ItemSortOrder::EndAsc => " order by end_date, update_date",
            ItemSortOrder::EndDesc => " order by end_date desc,  update_date",
            ItemSortOrder::StartAsc => " order by start_date, update_date",
            ItemSortOrder::StartDesc => " order by start_date desc, update_date",
            ItemSortOrder::UpdateAsc => " order by update_date, end_date",
            ItemSortOrder::UpdateDesc => " order by update_date desc, end_date",
        };
        let sql = if only_incomplete {
            format!("{} {} {};", sql1, sql2, sql3)
        } else {
            format!("{} {};", sql1, sql3)
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
}
