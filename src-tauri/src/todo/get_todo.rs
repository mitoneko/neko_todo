//! todoデータの取得

use super::*;
use crate::{config::ItemSortOrder, database::*};
use chrono::Local;
use log::error;
use uuid::Uuid;

impl Todo {
    /// todoの一覧を取得する。(仮実装。インターフェース未確定)
    pub async fn get_todo_list(
        &self,
        sess: Uuid,
        only_imcomplete: bool,
        sort_order: ItemSortOrder,
    ) -> Result<Vec<ItemTodo>, TodoError> {
        let ref_date = Local::now().date_naive();
        self.database
            .get_todo_item(sess, ref_date, only_imcomplete, sort_order)
            .await
            .map_err(|e| match e {
                DbError::FailDbAccess(e) => TodoError::FailDbAccess(e),
                e => unreachable!("[get_todo_list]get_todo_item[{e}]"),
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
}
