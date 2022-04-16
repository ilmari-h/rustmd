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
            Token::PlainText(t) => return t.compile(),
            Token::InlineCode(t) => return t.compile(),
            Token::Italic(t) => return t.compile(),
            Token::Bold(t) => return t.compile(),
            Token::Paragraph(t) => return t.compile(),
            Token::Link(t) => return t.compile(),
        }
    }
}

impl Compile<TargetHTML> for PlainText {

    fn compile(&self) -> TargetHTML {
        if self.text().is_empty() { "<br>".to_string() }
        else { format!("<span>{}</span>",self.text().to_string()) }
    }
}

impl Compile<TargetHTML> for Paragraph {
    fn compile(&self) -> TargetHTML {
        let children_html = self.children()
            .iter()
            .fold("".to_string(), |sum, s| format!("{}{}",sum,s.compile()));
        format!("<div>{}</div>",children_html)
    }
}

impl Compile<TargetHTML> for Link {
    fn compile(&self) -> TargetHTML {
        let children_html = self.children()
            .iter()
            .fold("".to_string(), |sum, s| format!("{}{}",sum,s.compile()));
        let href_tag = if self.url.is_empty() {"".to_string()} else {format!("href='{}'",self.url)};
        format!("<a {h}>{c}</a>",h=href_tag,c=children_html)
    }
}

impl Compile<TargetHTML> for Header {

    fn compile(&self) -> TargetHTML {
        let children_html = self.children()
            .iter()
            .fold("".to_string(), |sum, s| format!("{}{}",sum,s.compile()));
        return format!("<h{l}>{c}</h{l}>",l=self.level(),c=children_html);
    }
}

impl Compile<TargetHTML> for Italic {

    fn compile(&self) -> TargetHTML {
        return format!("<i>{t}</i>", t=self.text());
    }
}

impl Compile<TargetHTML> for Bold {

    fn compile(&self) -> TargetHTML {
        return format!("<strong>{}</strong>", self.text());
    }
}

impl Compile<TargetHTML> for InlineCode {

    fn compile(&self) -> TargetHTML {
        return format!("<code>{}</code>", self.text());
    }
}
