#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use backend_api as api;
use backend_api::Request;
use nfd2::Response;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex};

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

fn get_lfs_files(path: &PathBuf) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg("lfs")
        .arg("ls-files")
        .arg("-n")
        .current_dir(&path)
        .output()
        .expect("failed to run git lfs ls-files");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| String::from(s))
        .collect()
}

fn main() {
    let mut repo = Arc::new(Mutex::new(String::new()));
    let repo_handler = repo.clone();
    let mut lfs_files: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let lfs_files_handler = lfs_files.clone();
    tauri::AppBuilder::new()
        .invoke_handler(move |_webview, arg| {
            let mut repo_promise = repo_handler.clone();
            let mut lfs_files_promise = lfs_files_handler.clone();
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
                                    path: String::new(),
                                }),
                                Some(p) => {
                                    *repo_promise.lock().unwrap() =
                                        String::from(p.to_str().unwrap());
                                    *lfs_files_promise.lock().unwrap() = get_lfs_files(&p);
                                    Ok(api::Response::PickRepo {
                                        path: repo_promise.lock().unwrap().clone(),
                                    })
                                }
                            },
                            callback,
                            error,
                        ),
                        api::Request::GetLockedFiles { callback, error } => tauri::execute_promise(
                            _webview,
                            move || {
                                Ok(api::Response::GetLockedFiles {
                                    locked_files: vec!["asdf".to_string(), "asdfefe".to_string()],
                                })
                            },
                            callback,
                            error,
                        ),
                        Request::GetFilteredFiles {
                            filter,
                            callback,
                            error,
                        } => tauri::execute_promise(
                            _webview,
                            move || {
                                let filtered_list: Vec<String> = lfs_files_promise
                                    .lock()
                                    .unwrap()
                                    .iter()
                                    .filter(|f| f.contains(&filter))
                                    .cloned()
                                    .collect();
                                Ok(api::Response::GetFilteredFiles {
                                    filtered_files: filtered_list,
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
