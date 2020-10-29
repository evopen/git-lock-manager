use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Owner {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Lock {
    id: u32,
    path: String,
    owner: Owner,
    locked_at: String,
}
