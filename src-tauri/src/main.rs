#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use backend_api as api;
use backend_api::Request;
use nfd2::Response;
use std::path::PathBuf;

fn pick_repo() -> Option<std::path::PathBuf> {
    let p = match nfd2::open_pick_folder(None).unwrap() {
        Response::Okay(p) => p,
        Response::OkayMultiple(p) => p[0].clone(),
        Response::Cancel => return None,
    };

    std::fs::read_dir(&p)
        .ok()?
        .into_iter()
        .map(|p| p.unwrap().path())
        .find(|p| p.is_dir() && p.file_name().unwrap().eq(".git"))
        .map(|_| p)
}

fn main() {
    let repo = String::new();
    tauri::AppBuilder::new()
        .invoke_handler(|_webview, arg| {
            match serde_json::from_str::<api::Request>(arg) {
                Err(e) => Err(e.to_string()),
                Ok(command) => {
                    match command {
                        // definitions for your custom commands from Cmd here
                        api::Request::Echo { message } => {
                            //  your command code
                            println!("{}", message);
                        }
                        api::Request::PickRepo { callback, error } => tauri::execute_promise(
                            _webview,
                            move || match pick_repo() {
                                None => Ok(api::Response::PickRepo {
                                    path: String::from(""),
                                }),
                                Some(p) => Ok(api::Response::PickRepo {
                                    path: String::from(p.to_str().unwrap().to_string()),
                                }),
                            },
                            callback,
                            error,
                        ),
                        api::Request::GetLockedFiles { callback, error } => tauri::execute_promise(
                            _webview,
                            move || {
                                std::thread::sleep(std::time::Duration::from_secs(3));
                                Ok(api::Response::GetLockedFiles {
                                    locked_files: vec!["asdf".to_string(), "asdfefe".to_string()],
                                })
                            },
                            callback,
                            error,
                        ),
                    }
                    Ok(())
                }
            }
        })
        .build()
        .run();
}
