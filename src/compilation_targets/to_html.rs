use crate::tokens::*;


type TargetHTML = String;

trait Compile<T> {
    fn compile(&self) -> T;
}


pub fn compile_all(input: MdSyntaxTree) -> String {
    return input.compile();
}

impl Compile<TargetHTML> for MdLine {
    fn compile(&self) -> TargetHTML {

        // Fold all tokens on line.
        self.iter()
            .fold("".to_string(),
                |acc:String,line: &Token|
                    format!("{}{}\n",acc, line.compile())
            )
    }
}

impl Compile<TargetHTML> for MdSyntaxTree {
    fn compile(&self) -> TargetHTML {

        // Fold all lines in syntax tree.
        self.iter()
            .fold("".to_string(),
                |acc:String,line: &MdLine|
                    format!("{}{}",acc, line.compile())
            )
    }
}

impl Compile<TargetHTML> for Token {
    fn compile(&self) -> TargetHTML {
        match self {
            Token::Header(h) => return h.compile(),
            Token::PlainText(t) => return t.compile()
        }
    }
}

impl Compile<TargetHTML> for PlainText {

    fn compile(&self) -> TargetHTML {
        self.text().to_string()
    }
}

impl Compile<TargetHTML> for Header {

    fn compile(&self) -> TargetHTML {
        // TODO multiple
        let children_html = if self.children.len() > 0 {self.children()[0].compile()} else {"".to_string()};
        return format!("<h{l}>{c}</h{l}>",l=self.level(),c=children_html);
    }
}
