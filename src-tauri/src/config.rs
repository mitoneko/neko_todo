//! アプリケーション設定の取得関係

use directories::ProjectDirs;
use std::{
    fs::OpenOptions,
    io::{BufWriter, ErrorKind, Result, Write},
    path::PathBuf,
};
use uuid::Uuid;

const CONF_FILE_NAME: &str = "neko_todo.conf";
const DB_HOST: &str = "NEKO_DB_DB_HOST";
const DB_USER: &str = "NEKO_DB_DB_USER";
const DB_PASS: &str = "NEKO_DB_DB_PASS";
const SESSION: &str = "NEKO_DB_SESSION_ID";

#[derive(Debug)]
pub struct NekoTodoConfig {
    db_host: String,
    db_user: String,
    db_pass: String,
    session_id: Option<Uuid>,
    dirty: bool,
    is_incomplete: bool,
}

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
    fn get_config_file_path() -> Result<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 環境設定の挙動テスト
    #[test]
    #[ignore]
    fn test_env_val() {
        let val_db_host = "test_host";
        let val_db_user = "test_user";
        let val_db_pass = "test_pass";
        save_curr_conf_file();
        {
            let mut conf = NekoTodoConfig::new().unwrap();
            // 初期状態では空文字列が返るはず
            assert_eq!(conf.get_db_host(), "");
            assert_eq!(conf.get_db_user(), "");
            assert_eq!(conf.get_db_pass(), "");
            // test_hostをセットしてセットされているか確認。
            conf.set_db_host(val_db_host);
            conf.set_db_user(val_db_user);
            conf.set_db_pass(val_db_pass);
            assert_eq!(conf.get_db_host(), val_db_host);
            assert_eq!(conf.get_db_user(), val_db_user);
            assert_eq!(conf.get_db_pass(), val_db_pass);
        } // この時点で一旦環境ファイルを保存してみる。
          // 環境ファイルをもう一度ロードして、環境を確認
        delete_env_val();
        let conf = NekoTodoConfig::new().unwrap();
        assert_eq!(conf.get_db_host(), val_db_host);
        assert_eq!(conf.get_db_user(), val_db_user);
        assert_eq!(conf.get_db_pass(), val_db_pass);
        restore_curr_conf_file();
    }

    /// テスト環境のため、元のconfファイルを退避
    fn save_curr_conf_file() {
        let file = NekoTodoConfig::get_config_file_path().unwrap();
        let mut save_file = file.clone();
        save_file.set_extension("save");
        if file.exists() {
            println!(
                "現在の環境ファイル[{:?}]を[{:?}]に退避します。",
                &file, &save_file
            );
            std::fs::rename(file, save_file).unwrap();
        }
    }

    /// テスト環境のための一時ファイルを抹消し、元のファイルを復旧
    fn restore_curr_conf_file() {
        let file = NekoTodoConfig::get_config_file_path().unwrap();
        let mut save_file = file.clone();
        save_file.set_extension("save");
        if save_file.exists() {
            if file.exists() {
                println!("テスト用環境ファイル{:?}を削除します。", &file);
                std::fs::remove_file(&file).unwrap();
            }
            println!(
                "元の環境ファイル[{:?}]を[{:?}]に復元します。",
                &save_file, &file
            );
            std::fs::rename(save_file, file).unwrap();
        }
    }

    /// テスト環境のため、環境変数をすべて消去する。
    fn delete_env_val() {
        std::env::remove_var(DB_HOST);
        std::env::remove_var(DB_USER);
        std::env::remove_var(DB_USER);
    }
}
