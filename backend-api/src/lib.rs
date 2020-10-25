use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Request {
    Echo { message: String },
    PickRepo,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Request {
    Echo { message: String },
    PickRepo { callback: String, error: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    PickRepo { path: String },
}
