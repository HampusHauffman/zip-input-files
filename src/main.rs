mod zip_wasm;

use futures::FutureExt;
use gloo::console::log;
use gloo::file::callbacks::FileReader;
use gloo::file::ObjectUrl;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use web_sys::window;
use web_sys::File;
use web_sys::{Event, HtmlInputElement};
use yew::html::TargetCast;
use yew::Callback;
use yew::{html, Component, Context, Html};
use zip::ZipWriter;

use crate::zip_wasm::ZipWasm;

struct FileDetails {
    name: String,
    object_url: Option<ObjectUrl>,
}

pub enum Msg {
    Loaded(ObjectUrl),
    Files(Vec<(File, String)>),
}

pub struct App {
    pub comp_data: Arc<Mutex<Vec<u8>>>,
    pub readers: Arc<Mutex<HashMap<String, FileReader>>>,
    pub zip: Arc<Mutex<ZipWriter<Cursor<Vec<u8>>>>>,
    pub object_urls: Vec<ObjectUrl>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            comp_data: Arc::new(Mutex::new(Vec::new())),
            readers: Arc::new(Mutex::new(HashMap::default())),
            zip: Arc::new(Mutex::new(ZipWriter::new(Cursor::new(Vec::new())))),
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
            Msg::Files(files) => {
                let count = Arc::new(Mutex::new(files.len()));
                for file in files.into_iter() {
                    let f = file.1.clone();
                    let count = count.clone();
                    let file_name = file.0.name();
                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();
                        // Handle Filesystem entrie and read as bytes
                        gloo::file::callbacks::read_as_bytes(&file.0.into(), move |res| {
                            *count.lock().unwrap() -= 1;
                            let last = *count.lock().unwrap() == 0;
                            log!("count: ", *count.lock().unwrap());
                            log!("file: ", &file_name);
                            //link.send_message(Msg::Loaded(file.1.clone(), res.unwrap(), last))
                        })
                    };
                    let w = window().unwrap().document().unwrap();
                    self.readers.lock().unwrap().insert(f, task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ctx_clone = ctx.link().clone(); // Clone the ctx reference
        let z = self.zip.clone();
        let x = self.comp_data.clone();
        let y = self.readers.clone();

        let cb = Callback::from(move |e: Event| {
            let ctx_clone_inner = ctx_clone.clone();
            let ccb = Callback::from(move |obj: ObjectUrl| {
                ctx_clone_inner.send_message(Msg::Loaded(obj));
            });
            let input: HtmlInputElement = e.target_unchecked_into();
            zip_wasm::wasm_zip(
                input.files(),
                ZipWasm {
                    zip: z.clone(),
                    comp_data: x.clone(),
                    readers: y.clone(),
                },
                ccb,
            );
        });

        html! {
            <input
                id="file-upload"
                type="file"
                accept="*"
                multiple={true}
                webkitdirectory="true"
                onchange={cb}
            />
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
