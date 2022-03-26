#![allow(dead_code)]
#![allow(non_upper_case_globals)]


//mod js_bridge;
mod tokens;
mod compilation_targets;
mod parser;
use compilation_targets::to_html;

pub fn test() -> String {
    print!("hello\n");
    //let test: String = parser::parens("(testing)").unwrap().0.to_string();
    //let test2: String = parser::parens("(testing)").unwrap().1.to_string();
    //println!("1{}\n",test);
    //println!("2{}\n",test2);
    return "".to_string();
}
