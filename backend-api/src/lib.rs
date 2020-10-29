use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Owner {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockEntry {
    pub id: String,
    pub path: String,
    pub owner: Owner,
    pub locked_at: String,
}


#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Request {
    Echo { message: String },
    PickRepo,
    GetLockedFiles,
    GetFilteredFiles { filter: String },
    LockFile { path: String },
    UnlockFile { id: u32 },
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Request {
    Echo {
        message: String,
    },
    PickRepo {
        callback: String,
        error: String,
    },
    GetLockedFiles {
        callback: String,
        error: String,
    },
    GetFilteredFiles {
        filter: String,
        callback: String,
        error: String,
    },
    LockFile {
        path: String,
        callback: String,
        error: String,
    },
    UnlockFile {
        id: u32,
        callback: String,
        error: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    PickRepo { path: String },
    GetLockedFiles { locked_files: Vec<String> },
    GetFilteredFiles { filtered_files: Vec<String> },
    LockFile { lock_entry: LockEntry },
    UnlockFile { id: u32 },
}


