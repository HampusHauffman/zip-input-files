use futures::channel::mpsc;
use futures::StreamExt;
use gloo::file::Blob;
use gloo::file::ObjectUrl;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Write;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::FileList;
use zip::write::FileOptions;

use zip::ZipWriter;

#[derive(Deserialize)]
struct FileProperties {
    #[serde(rename = "webkitRelativePath")]
    webkit_relative_path: String,
}

#[wasm_bindgen]
pub struct WasmZip {
    object_url: Vec<ObjectUrl>,
}

impl Default for WasmZip {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmZip {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmZip {
        WasmZip {
            object_url: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn get_zip(&self) -> Vec<JsValue> {
        self.object_url
            .iter()
            .map(|v| JsValue::from_str(v))
            .collect()
    }

    #[wasm_bindgen]
    pub async fn zip(&mut self, files: FileList) -> Result<JsValue, JsValue> {
        let readers = Rc::new(RefCell::new(HashMap::new()));
        let zip = Rc::new(RefCell::new(ZipWriter::new(Cursor::new(Vec::new()))));
        let mut result = Vec::new();
        let (done_sender, mut done_receiver) = mpsc::unbounded::<ObjectUrl>();

        let files = js_sys::try_iter(&files).unwrap().unwrap().map(|v| {
            let file_prop: FileProperties =
                serde_wasm_bindgen::from_value(v.clone().unwrap()).unwrap();
            let file_path = Some(file_prop.webkit_relative_path).filter(|path| !path.is_empty());
            let file = web_sys::File::from(v.unwrap());
            (file_path.unwrap_or(file.name()), file)
        });
        result.extend(files);

        let count = Rc::new(RefCell::new(result.len()));

        for file in result {
            let count = count.clone();
            let name = file.0.clone();
            let zip = zip.clone();
            let done_sender = done_sender.clone();

            let task = {
                gloo::file::callbacks::read_as_bytes(&file.1.into(), move |res| {
                    *count.borrow_mut() -= 1;
                    let last = *count.borrow_mut() == 0;

                    let _ = zip.borrow_mut().start_file(
                        format!("{}", file.0.as_str()),
                        FileOptions::default().compression_method(zip::CompressionMethod::DEFLATE),
                    );
                    zip.borrow_mut().write_all(res.unwrap().as_slice()).unwrap();
                    if last {
                        let l = zip.borrow_mut().finish().unwrap().into_inner();
                        let object_url = ObjectUrl::from(Blob::new(l.as_slice()));
                        let _ = done_sender.unbounded_send(object_url);
                    }
                })
            };
            readers.borrow_mut().insert(name, task);
        }
        let object_url = match done_receiver.next().await {
            Some(obj) => obj,
            None => ObjectUrl::from(Blob::new("")),
        };
        readers.borrow_mut().clear();
        let obj_url_cp = object_url.clone();
        self.object_url.push(obj_url_cp);
        Ok(JsValue::from_str(&object_url))
    }
}
