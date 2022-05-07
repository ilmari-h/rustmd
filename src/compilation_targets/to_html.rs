use crate::tokens::*;

type TargetHTML = String;
type HtmlTags = (String, String);

trait Compile<T> {
    fn compile(&self) -> T;
}

pub fn compile_all(input: MdSyntaxTree) -> String {
    return input.compile();
}

// TODO: generic version of this that takes a function argument, similar use as `fold`
// Allows implementing different compilation targets
impl Compile<TargetHTML> for MdLine {
    fn compile(&self) -> TargetHTML {

        let mut html_str: String = String::new();
        let mut unclosed_by_depth = Vec::from([("".to_string(),0)]);
        for node in self.nodes_dfs() {

            let current_depth = node.level;

            // Close tags if higher in tree.
            let depth_predicate = |n: &&(String, usize)| n.1 >= current_depth;
            let to_close: Vec<_> = unclosed_by_depth.iter().filter(depth_predicate).collect();
            for unclosed in to_close {
                html_str.push_str(&unclosed.0);
            }
            unclosed_by_depth = unclosed_by_depth.iter().filter(|uc| !depth_predicate(uc)).cloned().collect();

            let html_tags = node.val.compile();
            html_str.push_str(&html_tags.0);

            // Add tag to queue to be closed later if has children. Else close tag now.
            if node.children.len() > 0  {
                unclosed_by_depth.push((html_tags.1, node.level));
            } else {
                html_str.push_str(&html_tags.1);
            }
        }
        for unclosed in &unclosed_by_depth { html_str.push_str(&unclosed.0) }
        return html_str;
    }
}

impl Compile<TargetHTML> for MdSyntaxTree {
    fn compile(&self) -> TargetHTML {

        // Fold all lines in syntax tree.
        self.iter()
            .fold("".to_string(),
                |mut acc: String,line: &MdLine|
                {acc.push_str(&line.compile()); acc}
            )
    }
}

impl Compile<HtmlTags> for Token {
    fn compile(&self) -> HtmlTags {
        match self {
            Token::Header(h) => return h.compile(),
            Token::List(t) => return t.compile(),
            Token::Paragraph(t) => return t.compile(),
            Token::PlainText(t) => return t.compile(),
            Token::InlineCode(t) => return t.compile(),
            Token::Italic(t) => return t.compile(),
            Token::Bold(t) => return t.compile(),
            Token::Link(t) => return t.compile(),
        }
    }
}

impl Compile<HtmlTags> for PlainText {

    fn compile(&self) -> HtmlTags {
        if self.text().is_empty() {("<br>".to_string(),"".to_string())}
        else { (format!("<span>{}",self.text().to_string()), "</span>".to_string()) }
    }
}

impl Compile<HtmlTags> for Paragraph {
    fn compile(&self) -> HtmlTags {
        return ("<div>".to_string(), "</div>".to_string());
    }
}

impl Compile<HtmlTags> for Link {
    fn compile(&self) -> HtmlTags {
        let href_tag = if self.url.is_empty() {"".to_string()} else {format!("href='{}'",self.url)};
        return ( format!("<a {h}>",h=href_tag,), "</a>".to_string());
    }
}

impl Compile<HtmlTags> for Header {

    fn compile(&self) -> HtmlTags {
        return ( format!("<h{l}>",l=self.level()), format!("</h{l}>",l=self.level()))
    }
}

impl Compile<HtmlTags> for Italic {

    fn compile(&self) -> HtmlTags {
        return ("<i>".to_string(), "</i>".to_string())
    }
}

impl Compile<HtmlTags> for Bold {

    fn compile(&self) -> HtmlTags {
        return ("<strong>".to_string(), "</strong>".to_string())
    }
}

impl Compile<HtmlTags> for InlineCode {

    fn compile(&self) -> HtmlTags {
        return ("<code>".to_string(), "</code>".to_string())
    }
}

impl Compile<HtmlTags> for List {

    fn compile(&self) -> HtmlTags {
        return ("<ul>".to_string(), "</ul>".to_string())
    }
}
