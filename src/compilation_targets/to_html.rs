use crate::tokens::*;

struct TargetHTML {
    html: String
}

pub trait Compile<S,T> {
    fn compile(source: S) -> T;
}


pub fn testing() -> String {
    return "tst".to_string();
}

impl Compile<MdSyntaxTree,TargetHTML> for TargetHTML {
    fn compile(source: MdSyntaxTree) -> TargetHTML {
        TargetHTML{html: "".to_string()}
    }
}

impl Compile<Header,String> for Header {

    fn compile(source: Header) -> String {
        return "".to_string();
    }
}
