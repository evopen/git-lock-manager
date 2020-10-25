#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use backend_api as api;
use nfd2::Response;

fn main() {
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
                            move || {
                                let p: String = match nfd2::open_pick_folder(None).unwrap() {
                                    Response::Okay(p) => p.to_str().unwrap().into(),
                                    Response::OkayMultiple(p) => p[0].to_str().unwrap().into(),
                                    Response::Cancel => String::new(),
                                };
                                Ok(api::Response::PickRepo { path: p })
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
