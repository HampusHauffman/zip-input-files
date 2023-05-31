use gloo::console::log;
use gloo::file::Blob;
use gloo::file::ObjectUrl;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Write;
use std::rc::Rc;
use web_sys::FileList;
use yew::Callback;
use zip::write::FileOptions;

use zip::ZipWriter;

#[derive(Debug, Deserialize)]
struct FileProperties {
    #[serde(rename = "webkitRelativePath")]
    webkit_relative_path: String,
}

pub fn wasm_zip(files: Option<FileList>, callback: Callback<ObjectUrl>) {
    let readers = Rc::new(RefCell::new(HashMap::new()));
    let zip = Rc::new(RefCell::new(ZipWriter::new(Cursor::new(Vec::new()))));
    let mut result = Vec::new();

    if let Some(files) = files {
        let files = js_sys::try_iter(&files).unwrap().unwrap().map(|v| {
            let file_prop: FileProperties =
                serde_wasm_bindgen::from_value(v.clone().unwrap()).unwrap();
            let file_path = Some(file_prop.webkit_relative_path).filter(|path| !path.is_empty());
            let file = web_sys::File::from(v.unwrap());
            (file_path.unwrap_or(file.name()), file)
        });
        result.extend(files);
    }

    let count = Rc::new(RefCell::new(result.len()));

    for file in result {
        let count = count.clone();
        let name = file.0.clone();
        let r = readers.clone();
        let cb = callback.clone();
        let zip = zip.clone();

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
                    log!("object_url: {:?}", object_url.to_string());
                    cb.emit(object_url);
                    //Hack: prevent FireReader from dropping (JS->WASM bs)
                    r.borrow_mut().clear();
                }
            })
        };
        readers.borrow_mut().insert(name, task);
    }
}