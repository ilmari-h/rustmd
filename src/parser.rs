use nom::{
    IResult,
    error::Error,
    Err,
    sequence::{delimited,preceded,tuple, terminated},
    character::complete::char,
    character::complete::{newline, digit1},
    character::{is_newline, complete::anychar},
    bytes::complete::is_not,
    bytes::complete::tag,
    bytes::complete::take_till,
    multi::{many1_count, many1, many_till, many0, fold_many0 },
    error::ErrorKind
};
use std::{collections::VecDeque, vec};

//#[macro_use]
use crate::{tokens::*, tree::{Tree, TreeIndex}};

/**
 Takes a line from a string of one or more lines. Returns Err if input is empty.
*/
fn take_line(s: &str) -> IResult<&str, &str> {
    match terminated(take_till(|c| is_newline(c as u8)),newline)(s) { // Cast to u8 safe? What about other line ending characters eg. \r?
        Ok(val) => Ok(val),
        Err(e) => {
            if s.is_empty() { return Err(e) }
            return Ok(("", s));
        }
    }
}

pub fn parse_md_str(input: &str) -> MdSyntaxTree {
    println!("Parsing lines...\n");
    let lines = many1(consume_lines)(input);
    return lines.unwrap().1;
}
/*

pub fn parse_line(line_str: IResult<&str,(&str,Token)>) -> MdLine {
}
*/

pub fn consume_lines(input: &str) -> IResult<&str,MdLine> {

    let (rem,(child_lines, token)) = parse_line_consuming_token(input)?;

    // Fill children in syntax tree by folding over each line consumed by the parent Token.
    let tree = child_lines.iter().fold(Tree::new(token.clone()), |mut acc,c_str| {
        let mut stack: VecDeque<(usize, &str, Token)> =
        parse_children(token.child_parsers(), c_str).iter().map(|c| (0,c.0,c.1.clone())).collect();


        while let Some((p_idx,unconsumed, child)) = stack.pop_front() {
            let parsers = child.child_parsers();
            let added = acc.add_node_by_index(TreeIndex::Arena(p_idx), child);
            let idx = added.unwrap().raw_idx;
            let stack_extended: VecDeque<(usize, &str, Token)> =
            parse_children(parsers, unconsumed).iter().map(|c| (idx,c.0,c.1.clone())).collect();
            stack.extend(stack_extended);
        }
        acc
    });
    print!("{}",tree);
    return Ok((rem,tree));

}


fn try_all_parsers(
    allowed_parsers: Vec<fn(&str) -> IResult<&str, (&str,Token)>>, source: &str)
-> IResult<&str,(&str,Token)>{
    if source.is_empty() {
        return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}));
    }
    for parse in allowed_parsers {
        let res = parse(source);
        if res.is_ok() { return res }
    }
    return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}));
}

/**
 Attempt to take a Token. If no Token is found, get leading plain text until
 Token is found or consume all input and return just PlainText.
*/
fn take_tokens_with_leading_plaintext(
    token_parsers: Vec<fn(&str) -> IResult<&str, (&str,Token)>>, src: &str)
-> IResult<&str,Vec<(&str,Token)>>{

    if src.is_empty(){
            return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}))
    }

    let (rem,consumed) = many0( |s| try_all_parsers(token_parsers.clone(), s))(src)?;
    // If no token found at head of input, consume into PlainText until found token or EOF.
    if consumed.is_empty() {
        let leading_plain_text_chars = many_till(anychar,|s| try_all_parsers(token_parsers.clone(), s))(src);

        return match leading_plain_text_chars {
            Ok((remt, (chars,tk))) => {

                let plain_text: String = chars.into_iter().collect();
                return Ok((
                    remt,
                    vec![
                        ("",Token::PlainText(PlainText{text: plain_text})), // Empty string since plain text has no children
                        tk]
                ));
            },
            // Reached end without finding tokens
            Err(_) => Ok((
                "",
                vec![
                    ("",Token::PlainText(PlainText{text: src.to_string()}))] // Empty string since plain text has no children
            ))
        }
    }
    // Found Token in the beginning, no plain text.
    return Ok((rem,consumed));
}

fn parse_children(
    allowed_children: Vec<fn(&str) -> IResult<&str, (&str,Token)>>, src: &str)
-> Vec<(&str,Token)> {

    let lines = fold_many0(
        |s| take_tokens_with_leading_plaintext(allowed_children.clone(), s),
        Vec::new,
        |mut res: Vec<_>, mut tokens| {res.append(&mut tokens); res}
    )(src);

    match lines {
        Ok((_,tokens)) => {
            return tokens;
        },
        Err(_) => vec![]

    }

}

fn parse_line_consuming_token(source: &str) -> IResult<&str, (Vec<&str>,Token)> {
    let line_consuming_tokens = [
        List::parse_lines,
        Header::parse_lines,
        Paragraph::parse_lines,
    ];
    for parse in line_consuming_tokens {
        let res = parse(source);
        if res.is_ok() { return res }
    }

    // Should only happen with empty input.
    let err = Error{input: "", code: ErrorKind::Satisfy};
    return Err(Err::Error(err));
}

pub trait LineConsumingParse {

    /**
     Parse consuming at least an entire line.
     Returns tuple with string containing consumed lines.
    */
    fn parse_lines(source: &str) -> IResult<&str,(Vec<&str>,Token)>;
}


/**
* Parse a token on a line. Remainder is the remaining line, tuple is string for children and Token
* itself.
*/
pub trait Parse {
    fn parse(source: &str) -> IResult<&str,(&str,Token)>;
}

impl LineConsumingParse for Header {
    fn parse_lines(source: &str) -> IResult<&str,(Vec<&str>,Token)> {

        let (rem_l,consumed) = take_line(source)?;
        let (rem,count) = terminated(many1_count(tag("#")), tag(" "))(consumed)?;
        return Ok((
            rem_l, // Remaining lines
            (
                vec![rem], // Possible children
                Token::Header(Header{
                    level: count as u32
                })
            )
        ))
    }
}

impl LineConsumingParse for Paragraph {
    fn parse_lines(source: &str) -> IResult<&str,(Vec<&str>,Token)> {
        let (rem_l, consumed) = take_line(source)?;
        return Ok((
            rem_l, // Remaining lines
            (
                vec![consumed], // Possible children
                Token::Paragraph(Paragraph{})
            )
        ));
    }
}

impl LineConsumingParse for List {
    fn parse_lines(source: &str) -> IResult<&str,(Vec<&str>,Token)> {
        //let line_number = preceded(digit1, tag(". "));
        let list_line = preceded(tag("- "), take_line);
        let (rem_l, consumed) = many1(list_line)(source)?;
        return Ok((
            rem_l,
            (
                consumed,
                Token::List(List{})
            )
        ))
    }
}

impl Parse for Italic {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        let (rem,consumed) = delimited( char('*'), is_not("*"), char('*'))(source)?;
        Ok((
            rem,
            (consumed,Token::Italic(Italic{}))
        ))
    }
}

impl Parse for Link {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        let caption = terminated(
            preceded(tag("["), take_till(|c| c == ']')),
            tag("]"));
        let url = terminated(
            preceded(tag("("), take_till(|c| c == ')')),
            tag(")"));

        let (rem,(caption,url)) = tuple((caption,url))(source)?;
        return Ok((
            rem,
            (caption,Token::Link(Link{url: url.to_string()}))
        ))
    }
}

impl Parse for InlineCode {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        let (rem,consumed) = terminated(
            preceded(tag("`"), take_till(|c| c == '`')),
            tag("`"))(source)?;
        Ok((
            rem,
            (consumed,Token::InlineCode(InlineCode{}))
        ))
    }
}

impl Parse for Bold {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        let (rem,consumed) = terminated(
            preceded(tag("**"), take_till(|c| c == '*')),
            tag("**"))(source)?;
        Ok((rem,
            (consumed,Token::Bold(Bold{}))
        ))
    }
}

// Consumes everything as any input passed to this will be a complete line
impl Parse for ListItem {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        return Ok(("",(source, Token::ListItem(ListItem{}))));
    }
}

pub trait HigherLevel {
    fn child_parsers(&self) -> Vec<fn(&str) -> IResult<&str, (&str,Token)>>;
}

impl HigherLevel for Token {
    fn child_parsers(&self) -> Vec<fn(&str) -> IResult<&str, (&str,Token)>> {
        match self {
            Token::Header(_) => vec![Italic::parse, Bold::parse, Link::parse, InlineCode::parse],
            Token::Paragraph(_) => vec![Italic::parse, Bold::parse, Link::parse, InlineCode::parse],
            Token::List(_) => vec![ListItem::parse],
            Token::ListItem(_) => vec![Italic::parse, Bold::parse, Link::parse, InlineCode::parse],
            Token::Link(_) => vec![Italic::parse, Bold::parse],
            Token::Bold(_) => vec![Italic::parse, Link::parse],
            Token::Italic(_) => vec![Bold::parse, Link::parse],
            Token::InlineCode(_) => vec![Italic::parse, Bold::parse],
            _ => vec![]
        }
    }
}

#[cfg(test)]
mod parser_t {
    use super::*;

    fn match_syntax(md_syntax: MdSyntaxTree, expected: Vec<Token>) {

        let mut tokens_in_order = vec![];
        for row in md_syntax {
            for tk in row.iter_dfs() {
                tokens_in_order.push(tk.clone());
            }
        }
        assert_eq!(tokens_in_order, expected);
    }

    #[test]
    fn t_headers() {
        let md_syntax = parse_md_str(
"# First header
## Second header
### Third header"
        );
        let expected_order: Vec<Token> = Vec::from([
            Token::Header(Header{level: 1}),
            Token::PlainText(PlainText{text: String::from("First header")}),
            Token::Header(Header{level: 2}),
            Token::PlainText(PlainText{text: String::from("Second header")}),
            Token::Header(Header{level: 3}),
            Token::PlainText(PlainText{text: String::from("Third header")}),
        ]);
        match_syntax(md_syntax, expected_order);
    }

    #[test]
    fn t_links() {
        let md_syntax = parse_md_str(
"[Link](http://gnu.org)
*[Italic link](http://gnu.org)*
**[Bold link](http://gnu.org)**"
        );
        let expected_order: Vec<Token> = Vec::from([
            Token::Paragraph(Paragraph{}),
            Token::Link(Link{url: String::from("http://gnu.org")}),
            Token::PlainText(PlainText{text: String::from("Link")}),

            Token::Paragraph(Paragraph{}),

            Token::Italic(Italic{}),
            Token::Link(Link{url: String::from("http://gnu.org")}),
            Token::PlainText(PlainText{text: String::from("Italic link")}),

            Token::Paragraph(Paragraph{}),

            Token::Bold(Bold{}),
            Token::Link(Link{url: String::from("http://gnu.org")}),
            Token::PlainText(PlainText{text: String::from("Bold link")}),
        ]);
        match_syntax(md_syntax, expected_order);
    }

    #[test]
    fn t_lists() {
        let md_syntax = parse_md_str(
"- First item
- Second item
- Third item"
        );
        let expected_order: Vec<Token> = Vec::from([
            Token::List(List{}),
            Token::ListItem(ListItem{}),
            Token::PlainText(PlainText{text: String::from("First item")}),
            Token::ListItem(ListItem{}),
            Token::PlainText(PlainText{text: String::from("Second item")}),
            Token::ListItem(ListItem{}),
            Token::PlainText(PlainText{text: String::from("Third item")}),
        ]);
        match_syntax(md_syntax, expected_order);
    }

    #[test]
    fn t_parse_italic() {
        assert_eq!(true,Italic::parse("*Text in italics*").is_ok());
        assert_eq!(true,Italic::parse("* Not complete").is_err());
        assert_eq!(true,Italic::parse("*Not complete").is_err());
        assert_eq!(true,Italic::parse("**").is_err());
    }

}
