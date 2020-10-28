#![recursion_limit = "1024"]

use backend_api as api;
use backend_api::Response;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::services::ConsoleService;
use yewtil::future::LinkFuture;

enum Msg {
    AddOne,
    Echo { message: String },
    PickRepo,
    RepoPicked { repo: String },
    FilterChanged(String),
    GetLockedFiles,
    LockedFilesReceived(Vec<String>),
    FilteredFilesReceived(Vec<String>),
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
    locked_files: Vec<String>,
    filtered_files: Vec<String>,
    list_type: ListType,
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

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
            repo: String::new(),
            filter: String::new(),
            locked_files: Vec::new(),
            filtered_files: Vec::new(),
            list_type: ListType::LockedFiles,
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

                false
            }
            Msg::LockedFilesReceived(v) => {
                self.locked_files = v;
                for s in &self.locked_files {
                    ConsoleService::log(&s);
                }
                false
            }
            Msg::FilteredFilesReceived(v) => {
                self.filtered_files = v;
                if !self.filtered_files.is_empty() {
                    ConsoleService::log(&self.filtered_files[0]);
                }
                true
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
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
                <button onclick=self.link.callback(|_| Msg::PickRepo)>{ "Pick Repo" }</button>
                <button onclick=self.link.callback(|_| Msg::Echo{message: String::from("fuck")})>{ "Echo" }</button>
                <p>{ &self.repo }</p>
                <input type="text" value={&self.filter} placeholder="Type Here" oninput=self.link.callback(|e: InputData| Msg::FilterChanged(e.value))/>
                <p>{ &self.filter }</p>
                <button onclick=self.link.callback(|_| Msg::GetLockedFiles)>{ "Get locked files" }</button>
                <ul>
                    { for self.filtered_files.iter().map(|f|{ html! {
                        <li>{f}</li>
                    } }) }
                </ul>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
