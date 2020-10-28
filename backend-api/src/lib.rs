use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Request {
    Echo { message: String },
    PickRepo,
    GetLockedFiles,
    GetFilteredFiles { filter: String },
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Request {
    Echo { message: String },
    PickRepo { callback: String, error: String },
    GetLockedFiles { callback: String, error: String },
    GetFilteredFiles { filter: String, callback: String, error: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    PickRepo { path: String },
    GetLockedFiles { locked_files: Vec<String> },
    GetFilteredFiles { filtered_files: Vec<String> }
}
