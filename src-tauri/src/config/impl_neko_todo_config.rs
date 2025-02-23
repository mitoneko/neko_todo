//! NetoTodoConfig実装

use super::*;
use directories::ProjectDirs;
use std::{
    fs::OpenOptions,
    io::{BufWriter, ErrorKind, Result, Write},
    path::PathBuf,
};
use uuid::Uuid;

impl NekoTodoConfig {
    pub fn new() -> dotenvy::Result<Self> {
        let file = Self::get_config_file_path().map_err(dotenvy::Error::Io)?;
        dotenvy::from_path(file)?;
        let session_id = std::env::var(SESSION)
            .ok()
            .map(|s| Uuid::parse_str(&s).expect("環境ファイル異常:SESSION_ID不正"));

        Ok(Self {
            db_host: std::env::var(DB_HOST).unwrap_or_default(),
            db_user: std::env::var(DB_USER).unwrap_or_default(),
            db_pass: std::env::var(DB_PASS).unwrap_or_default(),
            session_id,
            dirty: false,
            is_incomplete: true,
            item_sort_order: ItemSortOrder::EndAsc,
        })
    }

    pub fn get_db_host(&self) -> &str {
        &self.db_host
    }

    pub fn get_db_user(&self) -> &str {
        &self.db_user
    }

    pub fn get_db_pass(&self) -> &str {
        &self.db_pass
    }

    pub fn get_session_id(&self) -> Option<Uuid> {
        self.session_id
    }

    pub fn get_is_incomplete(&self) -> bool {
        self.is_incomplete
    }

    pub fn get_item_sort_order(&self) -> ItemSortOrder {
        self.item_sort_order
    }

    pub fn set_db_host(&mut self, val: &str) {
        self.db_host = val.to_string();
        self.dirty = true;
    }

    pub fn set_db_user(&mut self, val: &str) {
        self.db_user = val.to_string();
        self.dirty = true;
    }

    pub fn set_db_pass(&mut self, val: &str) {
        self.db_pass = val.to_string();
        self.dirty = true;
    }

    pub fn set_session_id(&mut self, uuid: &Uuid) {
        self.session_id = Some(*uuid);
        self.dirty = true;
    }

    pub fn set_is_incomplete(&mut self, is_incomplete: bool) {
        self.is_incomplete = is_incomplete;
    }

    pub fn set_item_sort_order(&mut self, item_sort_order: ItemSortOrder) {
        self.item_sort_order = item_sort_order;
    }

    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        let path = Self::get_config_file_path()?;
        let file = OpenOptions::new().write(true).truncate(true).open(&path)?;
        let mut buffer = BufWriter::new(file);
        writeln!(buffer, "{}={}", DB_HOST, self.get_db_host())?;
        writeln!(buffer, "{}={}", DB_USER, self.get_db_user())?;
        writeln!(buffer, "{}={}", DB_PASS, self.get_db_pass())?;
        if let Some(s) = self.session_id {
            writeln!(buffer, "{}={}", SESSION, s)?;
        }
        self.dirty = false;
        Ok(())
    }

    /// コンフィグファイルのファイル名を生成する
    /// 必要に応じて、コンフィグファイル用のディレクトリ("neko_todo")を生成し
    /// さらに、存在しなければ、空のコンフィグファイル("neko_todo.conf")を生成する。
    pub(super) fn get_config_file_path() -> Result<PathBuf> {
        use std::io;
        // 環境依存コンフィグ用ディレクトリの取得
        // 必要であれば、自分用のディレクトリを生成する。
        // ここでエラーになるのは、OSシステムに問題がある。
        let mut path: PathBuf = ProjectDirs::from("jp", "laki", "nekotodo")
            .ok_or(io::Error::new(ErrorKind::Other, "Not Found Home"))?
            .config_dir()
            .into();
        if let Err(e) = std::fs::create_dir(&path) {
            if e.kind() != ErrorKind::AlreadyExists {
                return Err(e);
            }
        }

        // コンフィグファイルがなければ、空のファイルを生成する。
        path.push(CONF_FILE_NAME);
        if let Err(e) = std::fs::File::create_new(&path) {
            if e.kind() != ErrorKind::AlreadyExists {
                return Err(e);
            }
        }
        Ok(path)
    }
}

impl Drop for NekoTodoConfig {
    fn drop(&mut self) {
        if self.dirty {
            self.save().unwrap();
        }
    }
}
