//! アプリケーション環境の構築を実施する
use clap::Parser;
use log::error;
use std::process::exit;
use tauri::async_runtime::block_on;
use thiserror::Error;

use crate::{
    app_status::AppStatus,
    config::NekoTodoConfig,
    todo::{Todo, TodoError},
};

/// アプリケーション環境の構築を行う。
pub fn setup() -> Result<AppStatus, SetupError> {
    let args = Args::parse();
    if args.setup {
        database_param_setup(&args)?;
    }

    let conf = NekoTodoConfig::new()?;

    if conf.get_db_host().is_empty()
        || conf.get_db_user().is_empty()
        || conf.get_db_pass().is_empty()
    {
        return Err(SetupError::Argument);
    }

    let todo = block_on(async {
        Todo::new(conf.get_db_host(), conf.get_db_user(), conf.get_db_pass()).await
    })?;

    Ok(AppStatus::new(conf, todo))
}

/// データベース接続パラメータの設定を設定ファイルに行い終了する。
fn database_param_setup(args: &Args) -> Result<(), SetupError> {
    let Some(ref host) = args.server else {
        return Err(SetupError::Argument);
    };
    let Some(ref user) = args.user else {
        return Err(SetupError::Argument);
    };
    let Some(ref pass) = args.pass else {
        return Err(SetupError::Argument);
    };

    // 一度試しに接続してみる。
    eprintln!("次のパラメータを使用します。");
    eprintln!("ホスト名:{}", host);
    eprintln!("ユーザー名:{}", user);
    eprintln!("パスワード:{}", pass);
    eprintln!("データベースへの接続を試行します。");
    block_on(async { Todo::new(host, user, pass).await })?;

    eprintln!("データベースへの接続に成功しました。");
    eprintln!("設定ファイルに接続情報を保存します。");
    {
        let mut conf = match NekoTodoConfig::new() {
            Ok(c) => Ok(c),
            Err(e) => Err(SetupError::SetupFile(e)),
        }?;

        conf.set_db_host(host);
        conf.set_db_user(user);
        conf.set_db_pass(pass);
    }
    eprintln!("アプリケーションを終了します。");
    exit(0);
}

/// アプリケーション引数の定義
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// データベース接続情報のセットアップを行う。
    #[arg(long)]
    setup: bool,
    /// データベースのサーバー名
    #[arg(short, long)]
    server: Option<String>,
    /// データベースのユーザー名
    #[arg(short, long)]
    user: Option<String>,
    /// データベースのパスワード
    #[arg(short, long)]
    pass: Option<String>,
}

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("設定ファイルへのアクセスに失敗")]
    SetupFile(#[from] dotenvy::Error),
    #[error("--setup時には、server,user,passの設定が必須です")]
    Argument,
    #[error("データベースへの接続に失敗")]
    ConnectDatabase(#[from] TodoError),
}
