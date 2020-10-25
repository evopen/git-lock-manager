use backend_api::{Request, Response};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window;
use yew::prelude::*;

enum Msg {
    AddOne,
    Echo { message: String },
    PickRepo,
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
                let fuck = get_tauri().unwrap();
                true
            }
            Msg::Echo { message } => {
                let tauri = get_tauri().unwrap();
                tauri.invoke(JsValue::from_serde(&Request::Echo { message }).unwrap());
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
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
