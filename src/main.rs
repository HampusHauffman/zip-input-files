//allow dead code
use flate2::write::GzEncoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use flate2::GzBuilder;
use gloo::console::log;
use gloo::file::callbacks::FileReader;
use gloo::file::Blob;
use gloo::file::ObjectUrl;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use web_sys::window;
use web_sys::File;
use web_sys::FileList;
use web_sys::FileSystemEntry;
use web_sys::{Event, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, Component, Context, Html};
use zip::write::FileOptions;
use zip::ZipWriter;

struct FileDetails {
    name: String,
    file_type: String,
    object_url: Option<ObjectUrl>,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>, bool),
    Files(Vec<(File, String)>),
}

pub struct App {
    readers: HashMap<String, FileReader>,
    file: Vec<FileDetails>,
    zip: Arc<Mutex<ZipWriter<Cursor<Vec<u8>>>>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            file: Vec::default(),
            zip: Arc::new(Mutex::new(ZipWriter::new(Cursor::new(Vec::new())))),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, v, last) => {
                let mut zip = self.zip.lock().unwrap();
                zip.start_file(
                    format!("{}{}", file_name.as_str(), last),
                    FileOptions::default().compression_method(zip::CompressionMethod::DEFLATE),
                )
                .unwrap();
                zip.write_all(v.as_slice()).unwrap();
                log!("Zipped file ", &file_name);
                log!("last ", last);
                if last {
                    let l = zip.finish().unwrap().into_inner();

                    let object_url = ObjectUrl::from(Blob::new(l.as_slice()));
                    self.file.push(FileDetails {
                        object_url: Some(object_url.clone()),
                        file_type,
                        name: file_name.clone(),
                    });

                    let win = window().unwrap();
                    let doc = win.document().unwrap();

                    let dl_link = doc.create_element("a").unwrap();
                    dl_link.set_attribute("href", &object_url).unwrap();
                    dl_link
                        .set_attribute("download", format!("{}.zip", file_name.as_str()).as_str())
                        .unwrap();
                    dl_link.set_inner_html(format!("{}.zip", file_name.as_str()).as_str());
                    let body = doc.body().unwrap();
                    let _ = body.append_child(&dl_link).unwrap();
                }
                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                let count = Arc::new(Mutex::new(files.len()));
                for file in files.into_iter() {
                    let count = count.clone();
                    let file_name = file.0.name();
                    let file_type = file.0.type_();
                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();
                        // Handle Filesystem entrie and read as bytes
                        gloo::file::callbacks::read_as_bytes(&file.0.into(), move |res| {
                            *count.lock().unwrap() -= 1;
                            let last = *count.lock().unwrap() == 0;
                            log!("count: ", *count.lock().unwrap());
                            log!("file: ", &file_name);
                            link.send_message(Msg::Loaded(file_name, file_type, res.unwrap(), last))
                        })
                    };
                    let w = window().unwrap().document().unwrap();
                    self.readers.insert(file.1, task);
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
                webkitdirectory="true"
                onchange={ctx.link().callback(move |e: Event| {
                    let input: HtmlInputElement = e.target_unchecked_into();
                    Self::upload_files(input.files())
                    })}
            />
        }
    }
}

#[derive(Debug, Deserialize)]
struct F {
    #[serde(rename = "webkitRelativePath")]
    webkit_relative_path: String,
}
impl App {
    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            log!("Files: {:?}", &files);
            let files = js_sys::try_iter(&files).unwrap().unwrap().map(|v| {
                let l: F = serde_wasm_bindgen::from_value(v.clone().unwrap()).unwrap();
                let r = Some(l.webkit_relative_path).filter(|path| !path.is_empty());

                (web_sys::File::from(v.unwrap()), r.unwrap())
            });
            result.extend(files);
        }
        Msg::Files(result)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
