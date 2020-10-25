#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use backend_api::{Request, Response};

fn main() {
    tauri::AppBuilder::new()
        .invoke_handler(|_webview, arg| {
            match serde_json::from_str::<Request>(arg) {
                Err(e) => Err(e.to_string()),
                Ok(command) => {
                    match command {
                        // definitions for your custom commands from Cmd here
                        Request::Echo { message } => {
                            //  your command code
                            println!("{}", message);
                        }
                        Request::PickRepo { callback, error } => {}
                    }
                    Ok(())
                }
            }
        })
        .build()
        .run();
}
