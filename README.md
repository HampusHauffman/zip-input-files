# zip-input-files 🗂
zip-input-files allows client side creation of zip files from HTML [`<input/>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input) tags thanks to wasm.
```js
    let wasm_zip = new WasmZip();
    let zipped_files = wasm_zip.zip(files);

```
where `files: FileList` are the [FileList](https://developer.mozilla.org/en-US/docs/Web/API/FileList) in the onchange of a [`<input/>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input) tag.


`zipped_files` in this case would be a [link](https://developer.mozilla.org/en-US/docs/Web/API/URL/URL) to the zip file.
```html
<a href={zipped_files} download="filename.zip">download the zip file</a>
```

