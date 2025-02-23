//! configモジュールテスト

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
    unsafe {
        std::env::remove_var(DB_HOST);
        std::env::remove_var(DB_USER);
        std::env::remove_var(DB_USER);
    }
}
