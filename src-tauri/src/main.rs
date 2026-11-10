#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::{fs, path::PathBuf};

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

    let portable_data_dir = root.join("data");
    let data_dir = if fs::create_dir_all(&portable_data_dir).is_ok() {
        portable_data_dir
    } else {
        let Some(local_data) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
            return;
        };
        let data_dir = local_data.join("Lightmatter").join("data");
        if fs::create_dir_all(&data_dir).is_err() {
            return;
        }
        data_dir
    };

    let candidates = [
        root.join("lightmatter"),
        root.join("lightmatter.exe"),
        root.join("resources").join("lightmatter.exe"),
        root.join("target").join("release").join("lightmatter"),
        root.join("target").join("release").join("lightmatter.exe"),
    ];
    let resources = root.join("resources");

    let binary = candidates
        .into_iter()
        .find(|path| path.exists())
        .or_else(|| find_sidecar(&resources))
        .or_else(|| find_sidecar(&root))
        .or_else(|| find_sidecar(&root.join("binaries")));
    let Some(binary) = binary else {
        return;
    };

    let backend_root = if resources.is_dir() { resources } else { root };
    let _ = Command::new(binary)
        .env("RESUMA_ADDR", server_addr)
        .env("RESUMA_DATA_DIR", &data_dir)
        .current_dir(&backend_root)
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(updater) = app_handle.updater() {
                    if let Ok(Some(update)) = updater.check().await {
                        let _ = update.download_and_install(|_, _| {}, || {}).await;
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lightmatter desktop app");
}
