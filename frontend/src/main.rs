#![recursion_limit = "1024"]

use backend_api as api;
use backend_api::Response;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::services::ConsoleService;
use yewtil::future::LinkFuture;

#[derive(Clone, Debug)]
enum Msg {
    AddOne,
    Echo { message: String },
    PickRepo,
    RepoPicked { repo: String },
    FilterChanged(String),
    GetLockedFiles,
    LockedFilesReceived(Vec<String>),
    FilteredFilesReceived(Vec<String>),
    LockFile(String),
    UnlockFile(String),
    FileLocked(Option<api::LockEntry>),
    FileUnlocked(String),
    UnlockAll,
}

enum ListType {
    LockedFiles,
    SearchResult,
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    value: i64,
    repo: String,
    filter: String,
    locked_files: HashMap<String, (String, u32)>,
    filtered_files: Vec<String>,
    list_type: ListType,
    update_time: f64,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = tauri)]
    pub type Tauri;

    #[wasm_bindgen(method, catch, js_name = promisified)]
    pub fn promisified(this: &Tauri, command: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = invoke)]
    fn invoke(this: &Tauri, command: JsValue);
}

#[wasm_bindgen]
pub fn get_tauri() -> Result<Tauri, JsValue> {
    let window = web_sys::window().unwrap();
    let tauri = window.get("__TAURI__").unwrap();
    let tauri: Tauri = js_sys::Reflect::get(&tauri, &"tauri".into())
        .unwrap()
        .unchecked_into::<Tauri>();

    Ok(tauri)
}

#[wasm_bindgen]
pub async fn pick_repo() -> Result<String, JsValue> {
    let tauri = get_tauri().unwrap();
    let value: JsValue = tauri
        .promisified(JsValue::from_serde(&api::Request::PickRepo).unwrap())
        .unwrap();
    let future = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&value));
    let response: api::Response = future.await?.into_serde().unwrap();
    match response {
        Response::PickRepo { path } => Ok(path),
        _ => Err(JsValue::from_str("failed to get pick repo response")),
    }
}

#[wasm_bindgen]
pub async fn get_locked_files() -> Result<js_sys::Array, JsValue> {
    let tauri = get_tauri().unwrap();
    let value: JsValue = tauri
        .promisified(JsValue::from_serde(&api::Request::GetLockedFiles).unwrap())
        .unwrap();
    let future = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&value));
    let response: api::Response = future.await?.into_serde().unwrap();
    match response {
        Response::GetLockedFiles { locked_files } => {
            Ok(locked_files.into_iter().map(JsValue::from).collect())
        }
        _ => Err(JsValue::from_str("failed to get locked files response")),
    }
}

#[wasm_bindgen]
pub async fn get_filtered_files(filter: String) -> Result<js_sys::Array, JsValue> {
    let tauri = get_tauri().unwrap();
    let value: JsValue = tauri
        .promisified(JsValue::from_serde(&api::Request::GetFilteredFiles { filter }).unwrap())
        .unwrap();
    let future = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&value));
    let response: api::Response = future.await?.into_serde().unwrap();
    match response {
        Response::GetFilteredFiles { filtered_files } => {
            Ok(filtered_files.into_iter().map(JsValue::from).collect())
        }
        _ => Err(JsValue::from_str("failed to get filtered files response")),
    }
}

#[wasm_bindgen]
pub async fn unlock_file(id: u32) -> Result<u32, JsValue> {
    let tauri = get_tauri().unwrap();
    let value: JsValue = tauri
        .promisified(JsValue::from_serde(&api::Request::UnlockFile { id }).unwrap())
        .unwrap();
    let future = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&value));
    ConsoleService::log("unlocking in async");
    let response: api::Response = future.await?.into_serde().unwrap();
    match response {
        Response::UnlockFile { id } => Ok(id),
        _ => Err(JsValue::from_str("failed to get unlock file response")),
    }
}

#[wasm_bindgen]
pub async fn lock_file(path: String) -> Result<String, JsValue> {
    let tauri = get_tauri().unwrap();
    let value: JsValue = tauri
        .promisified(JsValue::from_serde(&api::Request::LockFile { path }).unwrap())
        .unwrap();
    let future = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&value));
    ConsoleService::log("locking in async");
    let response: api::Response = future.await?.into_serde().unwrap();
    match response {
        Response::LockFile { lock_entry } => Ok(serde_json::to_string(&lock_entry).unwrap()),
        _ => Err(JsValue::from_str("failed to get lock file response")),
    }
}

// #[wasm_bindgen]
// pub async fn get_time(path: String) -> Result<u64, JsValue> {
//     Ok(SystemTime::now()
//         .duration_since(SystemTime::UNIX_EPOCH)
//         .unwrap()
//         .as_secs())
// }

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
            repo: String::new(),
            filter: String::new(),
            locked_files: HashMap::new(),
            filtered_files: Vec::new(),
            list_type: ListType::LockedFiles,
            update_time: 0.0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
            Msg::PickRepo => {
                self.link.send_future(async {
                    match pick_repo().await {
                        Ok(repo) => Msg::RepoPicked { repo },
                        Err(_) => Msg::RepoPicked {
                            repo: String::new(),
                        },
                    }
                });
                true
            }
            Msg::Echo { message } => {
                let tauri = get_tauri().unwrap();
                tauri.invoke(JsValue::from_serde(&api::Request::Echo { message }).unwrap());
                true
            }
            Msg::RepoPicked { repo } => {
                if !repo.is_empty() {
                    self.repo = repo;
                    self.list_type = ListType::LockedFiles;
                    self.filtered_files.clear();
                    self.filter.clear();
                    self.link.send_message(Msg::GetLockedFiles);
                    true
                } else {
                    false
                }
            }
            Msg::FilterChanged(filter) => {
                match filter.is_empty() {
                    true => self.list_type = ListType::LockedFiles,
                    false => {
                        ConsoleService::log(&"filterstart");
                        let filter = filter.clone();
                        self.list_type = ListType::SearchResult;
                        self.link.send_future(async {
                            match get_filtered_files(filter).await {
                                Ok(arr) => Msg::FilteredFilesReceived(
                                    arr.iter().map(|v| v.as_string().unwrap()).collect(),
                                ),
                                Err(_) => Msg::FilteredFilesReceived(Vec::new()),
                            }
                        })
                    }
                };
                self.filter = filter;
                true
            }
            Msg::GetLockedFiles => {
                // if js_sys::Date::now() / 1000.0 - self.update_time > 10.0 {
                    ConsoleService::log("updating");
                    match self.repo.is_empty() {
                        true => ConsoleService::log("did not select git repo yet"),
                        false => {
                            self.link.send_future(async {
                                match get_locked_files().await {
                                    Ok(arr) => Msg::LockedFilesReceived(
                                        arr.iter().map(|v| v.as_string().unwrap()).collect(),
                                    ),
                                    Err(_) => Msg::LockedFilesReceived(Vec::new()),
                                }
                            });
                        }
                    }
                //     self.update_time = js_sys::Date::now() / 1000.0;
                // } else {
                //     ConsoleService::log("skipping");
                // }

                false
            }
            Msg::LockedFilesReceived(v) => {
                ConsoleService::log("updated");
                self.locked_files.clear();
                for entry in v {
                    let entry: Vec<String> =
                        entry.split_whitespace().map(|s| s.to_string()).collect();

                    self.locked_files.insert(
                        entry.get(0).unwrap().clone(),
                        (
                            entry.get(1).unwrap().clone(),
                            entry.get(2).unwrap()[3..].parse().unwrap(),
                        ),
                    );
                }
                true
            }
            Msg::FilteredFilesReceived(v) => {
                self.filtered_files = v;
                if !self.filtered_files.is_empty() {
                    ConsoleService::log(&self.filtered_files[0]);
                }
                true
            }
            Msg::LockFile(v) => {
                ConsoleService::log("locking");
                if !self.locked_files.contains_key(&v.clone()) {
                    self.link.send_future(async {
                        match lock_file(v).await {
                            Ok(s) => {
                                let lock_entry =
                                    serde_json::from_str::<api::LockEntry>(s.as_str()).unwrap();
                                Msg::FileLocked(Some(lock_entry))
                            }
                            Err(_) => Msg::FileLocked(None),
                        }
                    });
                }
                false
            }
            Msg::UnlockFile(v) => {
                if self.locked_files.contains_key(&v.clone()) {
                    let id = self.locked_files.get(&v).unwrap().1;
                    self.link.send_future(async move {
                        match unlock_file(id).await {
                            Ok(s) => Msg::FileUnlocked(v),
                            Err(_) => Msg::FileUnlocked(String::new()),
                        }
                    });
                }
                false
            }
            Msg::FileLocked(s) => {
                if s.is_none() {
                    false
                } else {
                    let s = s.unwrap();
                    ConsoleService::log(format!("{} locked", s.path).as_str());
                    self.locked_files
                        .insert(s.path, (s.owner.name, s.id.parse().unwrap()));
                    self.link.send_future(async { Msg::GetLockedFiles });
                    true
                }
            }
            Msg::FileUnlocked(s) => {
                ConsoleService::log(format!("{} unlocked", s).as_str());
                self.locked_files.remove(&s);
                self.link.send_future(async { Msg::GetLockedFiles });
                true
            }
            Msg::UnlockAll => {
                for (k, v) in self.locked_files.iter() {
                    let k = k.clone();
                    self.link.send_future(async move { Msg::UnlockFile(k) });
                }
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".

        false
    }

    fn view(&self) -> Html {
        let filtered_list_item = |f: &String| {
            let is_locked = self.locked_files.contains_key(&f.clone());
            let locked_by = match self.locked_files.get(&f.clone()) {
                None => "",
                Some(v) => &v.0,
            };
            let (button_text, button_type, event) = if is_locked {
                (
                    "Unlock",
                    "pure-button button-success",
                    Msg::UnlockFile(f.clone()),
                )
            } else {
                (
                    "Lock",
                    "pure-button pure-button-primary",
                    Msg::LockFile(f.clone()),
                )
            };
            let path = f.split_whitespace().next().unwrap();

            html! {
                <tr>
                    <td>{ f }</td>
                    <td>{locked_by}</td>
                    <td class={"center"}>
                        <button class={button_type} onclick=self.link.callback(move |_|{event.clone()})>{button_text}</button>
                    </td>
                </tr>
            }
        };

        let locked_list_item = |f: &String| {
            let locked_by = match self.locked_files.get(&f.clone()) {
                None => "",
                Some(v) => &v.0,
            };
            let f = f.clone();
            html! {
                <tr>
                    <td>{ f.clone() }</td>
                    <td>{locked_by}</td>
                    <td class={"center"}>
                        <button class={"pure-button button-success"} onclick=self.link.callback(move |_|{Msg::UnlockFile(f.clone())})>{"Unlock"}</button>
                    </td>
                </tr>
            }
        };

        let table = match self.list_type {
            ListType::LockedFiles => {
                html! {
                <div>
                    <table class="pure-table">
                        <thead>
                            <tr>
                                <th>{"File Name"}</th>
                                <th>{"Locked By"}</th>
                                <th>{"Action"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for self.locked_files.iter().map(|(v, _)|{v}).map(locked_list_item) }
                        </tbody>
                    </table>
                </div>
                }
            }
            ListType::SearchResult => {
                html! {
                    <div>
                        <table class="pure-table">
                            <thead>
                                <tr>
                                    <th>{"File Name"}</th>
                                    <th>{"Locked By"}</th>
                                    <th>{"Action"}</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for self.filtered_files.iter().map(filtered_list_item) }
                            </tbody>
                        </table>
                    </div>
                }
            }
        };

        html! {
        <div>
            <div class={"pure-g"}>
                <div class="pure-u-1-5">
                    <button class={"pure-button button-warning"} onclick=self.link.callback(|_| Msg::PickRepo)>{ "Pick Repo" }</button>
                </div>
                <div class="pure-u-2-5">
                    <button class={"pure-button pure-button-disabled"} onclick=self.link.callback(|_| Msg::PickRepo)>{ &self.repo }</button>
                </div>
                <div class="pure-u-1-5">
                    <button class={"pure-button button-primary"} onclick=self.link.callback(|_| Msg::GetLockedFiles)>{ "Force Refresh Locked Files" }</button>
                </div>
            </div>
            <div class={"pure-g"}>
            <div class="pure-u-1">
                <form class="pure-form">
                 <fieldset>
                    <input type="text" value={&self.filter} class={"pure-input-2-3"} placeholder="Type Here" oninput=self.link.callback(|e: InputData| Msg::FilterChanged(e.value))/>
                    <button type="button" class={"pure-button button-secondary pure-input-1-3"} onclick=self.link.callback(|_| Msg::UnlockAll)>{ "Unlock All" }</button>
                 </fieldset>
                </form>
            </div>

            </div>
             {table}
        </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
