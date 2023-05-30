use std::{
    io::{Cursor, Write},
    sync::{Arc, Mutex},
};

use gloo::{console::log, file::callbacks::FileReader};
use web_sys::{FileList, HtmlInputElement};
use yew::prelude::*;
use zip::{write::FileOptions, ZipWriter};

fn zip(
    file_list: FileList,
    zip: Arc<Mutex<ZipWriter<Cursor<Vec<u8>>>>>,
    done: Callback<bool>,
) -> Vec<FileReader> {
    log!("zipping");
    let mut file_reader = Vec::new();
    let count = Arc::new(Mutex::new(0));
    let file_len = file_list.length();

    for i in 0..file_len {
        if let Some(file) = file_list.get(i) {
            let cc = count.clone();
            let name = file.name();
            zip.lock()
                .unwrap()
                .start_file(
                    name.as_str(),
                    FileOptions::default().compression_method(zip::CompressionMethod::DEFLATE),
                )
                .unwrap();
            let zip_c = zip.clone();
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
                    d.emit(true);
                }
            });
            file_reader.push(f);
        }
    }
    file_reader
}
#[function_component]
pub fn App() -> Html {
    let z = Arc::new(Mutex::new(ZipWriter::new(Cursor::new(Vec::new()))));
    let zc = z.clone();
    let dont_drop_file_reader = Arc::new(Mutex::new(Vec::new()));

    let on_files_zipped: Callback<bool> = Callback::from(move |_| {
        log!("done");
        let _ = zc.lock().unwrap().finish();
    });
    let on_files_uploaded: Callback<FileList> = Callback::from(move |files: FileList| {
        log!("files uploaded");
        let l = zip(files, z.clone(), on_files_zipped.clone());
        dont_drop_file_reader.lock().unwrap().extend(l); // Nee
    });

    html! {
        <FileUpload {on_files_uploaded} />
    }
}

#[function_component]
pub fn Zip() -> Html {
    html! {}
}

#[derive(Properties, PartialEq)]
pub struct FileUploadProps {
    pub on_files_uploaded: Callback<FileList>,
}

#[function_component()]
pub fn FileUpload(props: &FileUploadProps) -> Html {
    let files_uploaded = props.on_files_uploaded.clone();
    let onchange = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        if let Some(pat) = input.files() {
            files_uploaded.emit(pat);
        }

        // Self::upload_files(input.files())
    });

    html! {
        <input
            id="file-upload"
            type="file"
            accept="*"
            multiple={true}
            webkitdirectory="true"
            onchange={ onchange }
        />
    }
}
fn main() {
    yew::Renderer::<App>::new().render();
}
