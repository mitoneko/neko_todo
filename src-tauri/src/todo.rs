use chrono::{Days, Local, NaiveDate};
use serde::Serialize;

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
pub struct Todo {}

impl Todo {
    pub fn new() -> Self {
        Self {}
    }

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
}
