#![recursion_limit = "1024"]

use backend_api as api;
use backend_api::Response;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yewtil::future::LinkFuture;

enum Msg {
    AddOne,
    Echo { message: String },
    PickRepo,
    RepoPicked { repo: String },
    FilterChanged(String),
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    value: i64,
    repo: String,
    filter: String,
    lock_list: Vec<String>,
    filter_list: Vec<String>,
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
            lock_list: Vec::new(),
            filter_list: Vec::new(),
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
                self.filter = filter;
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
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
