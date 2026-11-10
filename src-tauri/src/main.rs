#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::{fs, path::PathBuf};

use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_updater::UpdaterExt;

fn start_local_server() {
    let server_addr = "127.0.0.1:4321";
    if std::net::TcpStream::connect(server_addr).is_ok() {
        return;
    }

    let current = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => return,
    };

    let root = match current.parent() {
        Some(parent) => parent.to_path_buf(),
        None => return,
    };

    let data_dir = root.join("data");
    std::fs::create_dir_all(&data_dir).ok();

    let candidates = [
        root.join("lightmatter"),
        root.join("lightmatter.exe"),
        root.join("target").join("release").join("lightmatter"),
        root.join("target").join("release").join("lightmatter.exe"),
    ];

    let binary = candidates
        .into_iter()
        .find(|path| path.exists())
        .or_else(|| find_sidecar(&root))
        .or_else(|| find_sidecar(&root.join("binaries")));
    let Some(binary) = binary else {
        return;
    };

    let _ = Command::new(binary)
        .env("RESUMA_ADDR", server_addr)
        .env("RESUMA_DATA_DIR", &data_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    for _ in 0..60 {
        if std::net::TcpStream::connect(server_addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn find_sidecar(dir: &std::path::Path) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.flatten().map(|entry| entry.path()).find(|path| {
        path.file_stem()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("lightmatter-"))
            && path.extension().and_then(|ext| ext.to_str()) == Some("exe")
    })
}

fn main() {
    start_local_server();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External("http://127.0.0.1:4321".parse().unwrap()))
                .title("Lightmatter")
                .build()?;

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(updater) = app_handle.updater() {
                    let _ = updater.check().await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lightmatter desktop app");
}
