#![allow(dead_code)]
#![allow(non_upper_case_globals)]

use wasm_bindgen::prelude::*;

//mod js_bridge;
mod tokens;
mod compilation_targets;
mod parser;

#[wasm_bindgen]
pub fn compile_md_from_js(input: &str) -> String {
    let tokenized = parser::parse_md_str(input);
    return compilation_targets::to_html::compile_all(tokenized).to_string();
}

#[wasm_bindgen]
pub fn main() -> String {
    //let test: String = parser::parens("(testing)").unwrap().0.to_string();
    //let test2: String = parser::parens("(testing)").unwrap().1.to_string();
    //println!("1{}\n",test);
    //println!("2{}\n",test2);
    return "Main func".to_string();
}
