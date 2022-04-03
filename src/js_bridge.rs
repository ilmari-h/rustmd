//use futures::StreamExt;
//use wasm_streams::ReadableStream;
//use wasm_bindgen::prelude::*;
//use js_sys::Uint8Array;
//// use web_sys::{console, window, Response,Document};
//// use wasm_bindgen_futures::JsFuture;
//
//fn to_vec_u8(chunk: JsValue) -> Vec<u8> {
//    let arr: Uint8Array = chunk.into();
//    return arr.to_vec();//.iter().map(|num| num - 1).collect();
//}
//
//fn to_js_array(chunk: Vec<u8>) -> Uint8Array {
//    unsafe {
//        return Uint8Array::view(&chunk[..]);
//    };
//}
//
//pub async fn read_js_stream(js_stream: wasm_streams::readable::sys::ReadableStream) -> Vec<u8> {
//
//    // Convert the JS ReadableStream to a Rust stream
//    let stream = ReadableStream::from_raw(js_stream).into_stream().map(|v| to_vec_u8(v.unwrap_throw()));
//
//    // Consume the stream and convert to a vector.
//    let chunks: Vec<Vec<u8>> = stream.collect().await;
//    return chunks.into_iter().flatten().collect();
//}
//
