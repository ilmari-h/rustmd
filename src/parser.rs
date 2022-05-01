use nom::{
    IResult,
    error::Error,
    Err,
    sequence::{delimited,preceded,tuple, terminated},
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
    character::complete::newline,
    character::complete::line_ending,
    character::{is_newline, complete::anychar},
    bytes::complete::is_not,
    bytes::complete::tag,
    bytes::complete::take_till,
    bytes::complete::take_till1,
    combinator::{not, rest,eof},
    multi::{many1_count, many1, many_till, many0, fold_many0}, // Get all lines and apply syntax parsing
    error::ErrorKind, Parser
};
use std::{collections::VecDeque, vec};

//#[macro_use]
use crate::{tokens::*, tree::{Tree, TreeIndex}};

// TODO: other inline symbols?
const INLINE_SYMBOL: char = '*';

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
    let lines = fold_many0(
        take_line,
        Vec::new,
        |mut res: Vec<_>, item| {res.push(parse_line(item)); res}
    )(input);

    return lines.unwrap().1;
}
/*
pub fn parse_line(line_str: &str) -> MdLine {
}
    */

pub fn parse_line(line_str: &str) -> MdLine {

    if let Ok((rem, token)) = parse_line_consuming_token(line_str) {

        // Fill children in line consuming token's syntax tree.
        let mut stack: VecDeque<(usize, &str, Token)> =
            parse_children(token.child_parsers(), rem).iter().map(|c| (0,c.0,c.1.clone())).collect();

        let mut line: Tree<Token> = Tree::new(
            token
        );

        while let Some((p_idx,unconsumed, child)) = stack.pop_front() {
            let parsers = child.child_parsers();
            let added = line.add_node_by_index(TreeIndex::Arena(p_idx), child);
            let idx = added.unwrap().raw_idx;
            let stack_extended: VecDeque<(usize, &str, Token)> =
                parse_children(parsers, unconsumed).iter().map(|c| (idx,c.0,c.1.clone())).collect();
            stack.extend(stack_extended);
            // added.unwrap().parent_raw_idx;
        }
        println!("{}",line);
        return line;
    }

    // Empty Line
    return Tree::new(Token::PlainText(PlainText{text:"".to_string()}));
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

    let leading_token = many0( |s| try_all_parsers(token_parsers.clone(), s))(src);
    match leading_token {
        Ok((rem,consumed)) => {
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
        },
        Err(_) => {
            println!("ERR WITH INPUT {:?}", src);
            return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}))
        }
    }
}

fn parse_children(
    allowed_children: Vec<fn(&str) -> IResult<&str, (&str,Token)>>, src: &str)
-> Vec<(&str,Token)> { // Vec<(&str, Token)>

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

fn parse_line_consuming_token(source: &str) -> IResult<&str, Token> {
    let line_consuming_tokens = [
        Header::parse_lines,
        Paragraph::parse_lines
    ];
    for parse in line_consuming_tokens {
        let res = parse(source);
        if res.is_ok() { return res }
    }
    // Shouldn't happen
    let err = Error{input: "", code: ErrorKind::Satisfy};
    return Err(Err::Error(err));
}

/**
* Parse consuming the entire line. Returns remainder for children.
*/
pub trait LineConsumingParse {
    fn parse_lines(source: &str) -> IResult<&str,Token>;
}

/**
* Parse a token on a line. Remainder is the remaining line, tuple is string for children and Token
* itself.
*/
pub trait Parse {
    fn parse(source: &str) -> IResult<&str,(&str,Token)>;
}

impl LineConsumingParse for Header {
    fn parse_lines(source: &str) -> IResult<&str,Token> {

        let count_r: IResult<&str, usize> = terminated(many1_count(tag("#")), tag(" "))(source);
        match count_r {
            Ok((rem, count)) => {

                return Ok((
                    rem,
                    Token::Header(Header{
                        level: count as u32
                    })
                ))
            },
            Err(e) => return Err(e)
        }
    }
}

impl LineConsumingParse for Paragraph {
    fn parse_lines(source: &str) -> IResult<&str,Token> {
        return Ok((
            source,
            Token::Paragraph(Paragraph{})
        ));
    }
}

impl LineConsumingParse for List {
    fn parse_lines(source: &str) -> IResult<&str,Token> {
        return Ok((
            source,
            Token::List(List{})
        ));
    }
}

impl Parse for Italic {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        let res: IResult<&str, &str> = delimited( char('*'), is_not("*"), char('*'))(source);
        match res {
            Ok((rem, consumed)) => Ok((
                rem,
                (consumed,Token::Italic(Italic{}))
            )),
            Err(e) => Err(e)
        }
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

        let res:IResult<&str, (&str,&str)> = tuple((caption,url))(source);
        match res {
            Ok((rem, (caption,url))) => {
                return Ok((
                    rem,
                    (caption,Token::Link(Link{url: url.to_string()}))
                    ))
                }
            ,
            Err(e) => Err(e)
        }
    }
}

impl Parse for InlineCode {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        let res: IResult<&str, &str> = terminated(
            preceded(tag("`"), take_till(|c| c == '`')),
            tag("`"))(source);
        match res {
            Ok((rem, consumed)) => Ok((
                rem,
                (consumed,Token::InlineCode(InlineCode{}))
            )),
            Err(e) => Err(e)
        }
    }
}

impl Parse for Bold {
    fn parse(source: &str) -> IResult<&str,(&str,Token)> {
        let res: IResult<&str, &str> = terminated(
            preceded(tag("**"), take_till(|c| c == INLINE_SYMBOL)),
            tag("**"))(source);
        match res {
            Ok((rem, consumed)) => Ok((rem,
                (consumed,Token::Bold(Bold{}))
            )),
            Err(e) => Err(e)
        }
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
            Token::Link(_) => vec![Italic::parse, Bold::parse],
            Token::Bold(_) => vec![Italic::parse, Link::parse],
            Token::Italic(_) => vec![Bold::parse, Link::parse],
            Token::InlineCode(_) => vec![Italic::parse, Bold::parse],
            _ => vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn t_take_line() {
    //     assert_eq!(("Line 2\nLine 3", "Line 1"), take_line("Line 1\nLine 2\nLine 3").unwrap());
    //     assert_eq!(true, take_line("Last Line").is_ok());
    //     assert_eq!(true, take_line("\nLast Line").is_ok());
    //     assert_eq!(true, take_line("\n\n\nLast Line").is_ok());
    //     assert_eq!(true, take_line("").is_err());
    // }

    #[test]
    fn t_parse_md_str() {
        assert_eq!(true,parse_md_str("\
            ### First header\n\
            **Bold text** Plain text in between *Italics after*\n\
            Here's some plain text *and italics* and plain text *and italics again*\n
            [This is a link](www.gnu.org)\n
            [**This is a bold link**](www.gnu.org)\n
            \nFailed italics text*\n**Unterminated bold\n# Last header"
        ).len() > 0);
        //assert_eq!(true,parse_md_str("## Hi").len() > 0);
    }

    #[test]
    fn t_parse_italic() {
        assert_eq!(true,Italic::parse("*Text in italics*").is_ok());
        assert_eq!(true,Italic::parse("* Not complete").is_err());
        assert_eq!(true,Italic::parse("*Not complete").is_err());
        assert_eq!(true,Italic::parse("**").is_err());
    }

}
