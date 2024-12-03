//! アプリケーション全体のステータスを保持する。

use crate::{config::NekoTodoConfig, todo::Todo};
use std::sync::{Arc, Mutex};

pub struct AppStatus {
    config: Arc<Mutex<NekoTodoConfig>>,
    todo: Todo,
}

impl AppStatus {
    pub fn new() -> Self {
        let config = match NekoTodoConfig::new() {
            Ok(conf) => Arc::new(Mutex::from(conf)),
            Err(e) => {
                eprintln!("致命的エラー。configの取得に失敗。\n {}", e);
                std::process::exit(1)
            }
        };
        let todo = Todo::new();
        Self { config, todo }
    }

    pub fn config(&self) -> &Mutex<NekoTodoConfig> {
        &self.config
    }

    pub fn todo(&self) -> &Todo {
        &self.todo
    }
}
