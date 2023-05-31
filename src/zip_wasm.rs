use gloo::console::log;
use gloo::file::callbacks::FileReader;
use gloo::file::Blob;
use gloo::file::ObjectUrl;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use web_sys::FileList;
use yew::Callback;
use zip::write::FileOptions;

use zip::ZipWriter;
pub struct ZipWasm {
    pub comp_data: Arc<Mutex<Vec<u8>>>,
    pub readers: Arc<Mutex<HashMap<String, FileReader>>>,
    pub zip: Arc<Mutex<ZipWriter<Cursor<Vec<u8>>>>>,
}

#[derive(Debug, Deserialize)]
struct FileProperties {
    #[serde(rename = "webkitRelativePath")]
    webkit_relative_path: String,
}

pub fn wasm_zip(files: Option<FileList>, store: ZipWasm, callback: Callback<ObjectUrl>) {
    let mut result = Vec::new();

    if let Some(files) = files {
        let files = js_sys::try_iter(&files).unwrap().unwrap().map(|v| {
            let file_prop: FileProperties =
                serde_wasm_bindgen::from_value(v.clone().unwrap()).unwrap();
            let file_path = Some(file_prop.webkit_relative_path).filter(|path| !path.is_empty());
            (file_path.unwrap(), web_sys::File::from(v.unwrap()))
        });
        result.extend(files);
    }

    let count = Arc::new(Mutex::new(result.len()));

    for file in result {
        let count = count.clone();
        let name = file.0.clone();
        let zip = store.zip.clone();
        let cmp = store.comp_data.clone();
        let cb = callback.clone();

        let task = {
            gloo::file::callbacks::read_as_bytes(&file.1.into(), move |res| {
                *count.lock().unwrap() -= 1;
                let last = *count.lock().unwrap() == 0;

                let mut zip = zip.lock().unwrap();
                let _ = zip.start_file(
                    format!("{}", file.0.as_str()),
                    FileOptions::default().compression_method(zip::CompressionMethod::DEFLATE),
                );
                zip.write_all(res.unwrap().as_slice()).unwrap();
                if last {
                    let l = zip.finish().unwrap().into_inner();
                    let object_url = ObjectUrl::from(Blob::new(l.as_slice()));
                    cmp.lock().unwrap().extend(l);
                    log!("object_url: {:?}", object_url.to_string());
                    cb.emit(object_url);
                }
            })
        };
        store.readers.lock().unwrap().insert(name, task);
    }
}
