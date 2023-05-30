use std::{
    io::{Cursor, Write},
    sync::{Arc, Mutex},
};

use gloo::{console::log, file::callbacks::FileReader};
use web_sys::{FileList, HtmlInputElement};
use yew::prelude::*;
use zip::{write::FileOptions, ZipWriter};

pub struct ZipComp {
    zip: Arc<Mutex<ZipWriter<Cursor<Vec<u8>>>>>,
    zipped: Option<bool>,
}

impl Component for ZipComp {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            zip: Arc::new(Mutex::new(ZipWriter::new(Cursor::new(Vec::new())))),
            zipped: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = Callback::from(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let file_readers = self.zip_it(input.files().unwrap(), cb);
        });

        html! {
            <input
                id="file-upload"
                type="file"
                accept="*"
                multiple={true}
                webkitdirectory="true"
                onchange={callback}
            />
        }
    }
}
impl ZipComp {
    fn a(&self, e: Event) {
        let callback = Callback::from(|file_path: String| {
            // Process the returned file_path here
            log!("Received file path: ", file_path);
        });
        let input: HtmlInputElement = e.target_unchecked_into();
        let done_callback = callback.clone();
        let file_readers = self.zip_it(input.files().unwrap(), callback);
    }
    fn zip_it(&self, file_list: FileList, done: Callback<String>) -> Vec<FileReader> {
        log!("zipping");
        let mut file_reader = Vec::new();
        let count = Arc::new(Mutex::new(0));
        let file_len = file_list.length();

        for i in 0..file_len {
            if let Some(file) = file_list.get(i) {
                let cc = count.clone();
                let name = file.name();
                self.zip
                    .lock()
                    .unwrap()
                    .start_file(
                        name.as_str(),
                        FileOptions::default().compression_method(zip::CompressionMethod::DEFLATE),
                    )
                    .unwrap();
                let zip_c = self.zip.clone();
                let d = done.clone();
                log!("read as bytes1");
                let f = gloo::file::callbacks::read_as_bytes(&file.into(), move |res| {
                    log!("read as bytes");
                    let mut z = zip_c.lock().unwrap();
                    let _ = z.start_file(
                        name.as_str(),
                        FileOptions::default().compression_method(zip::CompressionMethod::DEFLATE),
                    );
                    let _ = z.write_all(res.unwrap().as_slice());
                    *cc.lock().unwrap() += 1;
                    if *cc.lock().unwrap() == file_len {
                        d.emit("done".to_string());
                    }
                });
                file_reader.push(f);
            }
        }
        file_reader
    }
}

fn main() {
    yew::Renderer::<ZipComp>::new().render();
}
