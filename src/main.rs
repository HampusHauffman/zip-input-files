mod zip_wasm;

use gloo::console::log;
use gloo::file::ObjectUrl;
use web_sys::window;
use web_sys::{Event, HtmlInputElement};
use yew::html::TargetCast;
use yew::Callback;
use yew::{html, Component, Context, Html};

pub enum Msg {
    Loaded(ObjectUrl),
}

pub struct App {
    pub object_urls: Vec<ObjectUrl>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            object_urls: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(object_url) => {
                self.object_urls.push(object_url.clone());
                let win = window().unwrap();
                let doc = win.document().unwrap();

                let dl_link = doc.create_element("a").unwrap();
                dl_link.set_attribute("href", &object_url).unwrap();
                dl_link
                    .set_attribute("download", format!("done.zip").as_str())
                    .unwrap();
                dl_link.set_inner_html(format!("don.zip").as_str());
                let body = doc.body().unwrap();
                let _ = body.append_child(&dl_link).unwrap();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ctx_clone = ctx.link().clone(); // Clone the ctx reference
        let ctx_clone2 = ctx.link().clone(); // Clone the ctx reference

        let cb = Callback::from(move |e: Event| {
            let ctx_clone_inner = ctx_clone.clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            wasm_bindgen_futures::spawn_local(async move {
                let obj = zip_wasm::wasm_zip(input.files()).await;
                ctx_clone_inner.send_message(Msg::Loaded(obj));
            });
        });

        html! {
            <>
                <input
                    id="file-upload"
                    type="file"
                    accept="*"
                    multiple={true}
                    directory="true"
                    webkitdirectory="true"
                    onchange={cb.clone()}
                />
            </>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
