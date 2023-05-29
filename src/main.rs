use flate2::write::GzEncoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use flate2::GzBuilder;
use gloo::file::callbacks::FileReader;
use gloo::file::Blob;
use gloo::file::File;
use gloo::file::ObjectUrl;
use std::collections::HashMap;
use std::io::prelude::*;
use web_sys::window;
use web_sys::HtmlElement;
use web_sys::{Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, Component, Context, Html};
use zip::write::FileOptions;

struct FileDetails {
    name: String,
    file_type: String,
    object_url: Option<ObjectUrl>,
}

pub enum Msg {
    Loaded(String, String, Option<ObjectUrl>),
    Files(Vec<File>),
}

pub struct App {
    readers: HashMap<String, FileReader>,
    file: Vec<FileDetails>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            file: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, object_url) => {
                self.file.push(FileDetails {
                    object_url: object_url.clone(),
                    file_type,
                    name: file_name.clone(),
                });
                let win = window().unwrap();
                let doc = win.document().unwrap();
                let dl_link = doc.create_element("a").unwrap();
                dl_link.set_attribute("href", &object_url.unwrap()).unwrap();
                dl_link
                    .set_attribute("download", format!("{}.gzip", file_name.as_str()).as_str())
                    .unwrap();
                dl_link.set_inner_html("CLICK ME");
                let body = doc.body().unwrap();
                let c = body.append_child(&dl_link).unwrap();
                // Trigger the click event on the download link

                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            let mut e = GzBuilder::new()
                                .filename(file_name.as_str())
                                .write(Vec::new(), Compression::default());
                            e.write_all(&res.unwrap()).unwrap();

                            let compressed_bytes = e.finish().unwrap();
                            let object_url =
                                ObjectUrl::from(Blob::new(compressed_bytes.as_slice()));
                            link.send_message(Msg::Loaded(file_name, file_type, Some(object_url)))
                        })
                    };
                    let w = window().unwrap().document().unwrap();
                    self.readers.insert(file_name, task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <input
                id="file-upload"
                type="file"
                accept="*"
                multiple={true}
                onchange={ctx.link().callback(move |e: Event| {
                    let input: HtmlInputElement = e.target_unchecked_into();
                    Self::upload_files(input.files())
                    })}
                   // webkitdirectory="true"
            />
        }
    }
}

impl App {
    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
