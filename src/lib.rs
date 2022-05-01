#![allow(dead_code)]
#![allow(non_upper_case_globals)]

use wasm_bindgen::prelude::*;

//mod js_bridge;
mod tokens;
mod tree;
mod compilation_targets;
mod parser;


#[wasm_bindgen]
pub fn compile_md_from_js(input: &str) -> String {
    let tokenized = parser::parse_md_str(input);
    return compilation_targets::to_html::compile_all(tokenized).to_string();
}

pub fn main() {

}
