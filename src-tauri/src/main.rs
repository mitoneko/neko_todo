// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use directories::ProjectDirs;

fn main() {
    let mut log_file: std::path::PathBuf = ProjectDirs::from("jp", "laki", "nekotodo")
        .unwrap()
        .config_dir()
        .into();
    if !log_file.exists() {
        std::fs::create_dir_all(&log_file).unwrap();
    }
    log_file.push("nekotodo.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}:{} {}",
                chrono::Local::now().format("%Y/%m/%d %H:%M:%S"),
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                message
            ))
        })
        //.level(log::LevelFilter::Info)
        .level(log::LevelFilter::Debug)
        .chain(std::io::stderr())
        .chain(fern::log_file(log_file).unwrap())
        .apply()
        .unwrap();

    neko_todo_lib::run()
}
