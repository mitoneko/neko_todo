//! todoデータの編集

use super::*;
use crate::database::*;
use log::error;
use uuid::Uuid;

impl Todo {
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
}
