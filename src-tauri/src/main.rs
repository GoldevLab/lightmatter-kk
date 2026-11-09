#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use tauri::{WebviewUrl, WebviewWindowBuilder};

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

    let candidates = [
        root.join("lightmatter"),
        root.join("lightmatter.exe"),
        root.join("target").join("release").join("lightmatter"),
        root.join("target").join("release").join("lightmatter.exe"),
    ];

    let binary = candidates.into_iter().find(|path| path.exists());
    let Some(binary) = binary else {
        return;
    };

    let _ = Command::new(binary)
        .env("RESUMA_ADDR", server_addr)
        .env("RESUMA_DATA_DIR", root.join("data"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    for _ in 0..50 {
        if std::net::TcpStream::connect(server_addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn main() {
    start_local_server();

    tauri::Builder::default()
        .setup(|app| {
            let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External("http://127.0.0.1:4321".parse().unwrap()))
                .title("Lightmatter")
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lightmatter desktop app");
}
