use std::env;
use std::fs;
mod tokens;
mod parser;
mod compilation_targets;
use compilation_targets::to_html::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[2];
    println!("Searching for {}",filename);
    let contents = fs::read_to_string(filename).expect("Error reading file.");
    let stuff = parser::parse_md_str(&contents);
    println!("{:?}", stuff);
    let html = compile_all(stuff);
    println!("{:?}", html);
}
