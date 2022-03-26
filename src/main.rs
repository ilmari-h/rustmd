use rustmd::test;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let contents = fs::read_to_string(filename).expect("Error reading file.");
    println!("File - {}\n{}",filename, contents);
    test();
}
