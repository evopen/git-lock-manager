#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use anyhow::{anyhow, Result};
use backend_api as api;
use backend_api::{LockEntry, Request};
use nfd2::Response;
use serde_json::Error;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::sync::{Arc, Mutex, RwLock};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::os::windows::process::CommandExt;

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
        .find(|p| p.file_name().unwrap().eq(".git"))
        .map(|_| p)
}

fn get_lfs_files(path: &PathBuf) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg("lfs")
        .arg("ls-files")
        .arg("-n")
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .current_dir(&path)
        .output()
        .expect("failed to run git lfs ls-files");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| String::from(s))
        .collect()
}

fn get_locked_files(path: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg("lfs")
        .arg("locks")
        .current_dir(&path)
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .output()
        .expect("failed to run git lfs locks");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| String::from(s))
        .collect()
}

fn lock_file(repo: &str, file: &str) -> Option<api::LockEntry> {
    let repo = repo.to_string();
    let file = file.to_string();
    println!("locking {}", file);
    let output = std::process::Command::new("git")
        .arg("lfs")
        .arg("lock")
        .arg(&file)
        .arg("--json")
        .current_dir(&repo)
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .output()
        .expect(format!("failed to lock {}", file).as_str());
    match serde_json::from_str::<api::LockEntry>(String::from_utf8(output.stdout).unwrap().as_str())
    {
        Ok(e) => Some(e),
        Err(_) => {
            println!("failed");
            None
        }
    }
}

fn unlock_file(repo: &str, id: u32) {
    let repo = repo.to_string();
    println!("unlocking {}", id);
    let output = std::process::Command::new("git")
        .arg("lfs")
        .arg("unlock")
        .arg("-i")
        .arg(id.to_string())
        .current_dir(&repo)
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .output()
        .expect(format!("failed to unlock {}", id).as_str());
}

fn main() {
    let repo = Arc::new(RwLock::new(String::new()));
    let repo_handler = repo.clone();
    let lfs_files: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let lfs_files_handler = lfs_files.clone();
    let matcher = Arc::new(Mutex::new(SkimMatcherV2::default()));
    let matcher_handler = matcher.clone();
    tauri::AppBuilder::new()
        .invoke_handler(move |_webview, arg| {
            let repo_promise = repo_handler.clone();
            let lfs_files_promise = lfs_files_handler.clone();
            let matcher_promise = matcher_handler.clone();
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
                                    *repo_promise.write().unwrap() =
                                        String::from(p.to_str().unwrap());
                                    *lfs_files_promise.lock().unwrap() = get_lfs_files(&p);
                                    Ok(api::Response::PickRepo {
                                        path: repo_promise.read().unwrap().clone(),
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
                                    locked_files: get_locked_files(&*repo_promise.read().unwrap()),
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
                                    .filter(|f| matcher_promise.lock().unwrap().fuzzy_match(f.to_lowercase().as_str(), &filter.to_lowercase().as_str()).is_some())
                                    .take(50)
                                    .cloned()
                                    .collect();
                                Ok(api::Response::GetFilteredFiles {
                                    filtered_files: filtered_list,
                                })
                            },
                            callback,
                            error,
                        ),
                        Request::LockFile {
                            path,
                            callback,
                            error,
                        } => {
                            println!("received lock request");
                            tauri::execute_promise(
                                _webview,
                                move || match lock_file(&*repo_promise.read().unwrap(), &path) {
                                    None => Err(anyhow!("failed to lock file")),
                                    Some(lock_entry) => Ok(api::Response::LockFile { lock_entry }),
                                },
                                callback,
                                error,
                            )
                        }
                        Request::UnlockFile {
                            id,
                            callback,
                            error,
                        } => {
                            println!("received unlock request");
                            tauri::execute_promise(
                                _webview,
                                move || {
                                    unlock_file(&*repo_promise.read().unwrap(), id);
                                    Ok(api::Response::UnlockFile { id })
                                },
                                callback,
                                error,
                            )
                        }
                    }
                    Ok(())
                }
            }
        })
        .build()
        .run();
}
