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
//#[macro_use]
use crate::tokens::*;

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
    let lines = fold_many0(
        take_line,
        Vec::new,
        |mut res: Vec<_>, item| {res.push(parse_line(item)); res}
    )(input);

    return lines.unwrap().1;
}

pub fn parse_line(line_str: &str) -> MdLine {

    // Test first token, in case it's a line consuming Token. In this case all input is consumed.
    // Eg. A header.
    return vec![parse_line_consuming_token(line_str).unwrap().1];
}

// Implement Parsable for tokens
pub trait Parsable {
    fn parse(source: &str) -> IResult<&str,Token>;
}

fn try_all_parsers(allowed_parsers: Vec<fn(&str) -> IResult<&str, Token>>, source: &str) -> IResult<&str,Token>{
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
fn take_tokens_with_leading_plaintext(token_parsers: Vec<fn(&str) -> IResult<&str, Token>>, src: &str) -> IResult<&str,Vec<Token>>{

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
                        return Ok((remt,vec![Token::PlainText(PlainText{text: plain_text}),tk]));
                    },
                    // Reached end without finding tokens
                    Err(_) => Ok(("",vec![Token::PlainText(PlainText{text: src.to_string()})]))
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

fn parse_children(allowed_children: Vec<fn(&str) -> IResult<&str, Token>>, src: &str) -> Vec<Token> {

    let lines = fold_many0(
        |s| take_tokens_with_leading_plaintext(allowed_children.clone(), s),
        Vec::new,
        |mut res: Vec<_>, mut tokens| {res.append(&mut tokens); res}
    )(src);

    println!("{:?}",lines);

    match lines {
        Ok((_,tokens)) => {
            return tokens;
        },
        Err(_) => vec![]

    }

}

fn parse_line_consuming_token(source: &str) -> IResult<&str, Token> {
    let line_consuming_tokens = [
        Header::parse,
        Paragraph::parse
    ];
    for parse in line_consuming_tokens {
        let res = parse(source);
        if res.is_ok() { return res }
    }
    // Shouldn't happen
    let err = Error{input: "", code: ErrorKind::Satisfy};
    return Err(Err::Error(err));
}

impl Parsable for Header {
    fn parse(source: &str) -> IResult<&str,Token> {

        let count_r: IResult<&str, usize> = terminated(many1_count(tag("#")), tag(" "))(source);
        match count_r {
            Ok((rem, count)) => {

                // Here call some generic method to consume all children until empty remainder.
                let children = parse_children(vec![Bold::parse, Italic::parse,Link::parse, InlineCode::parse], rem);
                return Ok((
                    "",
                    Token::Header(Header{
                        children,
                        level: count as u32
                    })
                ))
            },
            Err(e) => return Err(e)
        }
    }
}

impl Parsable for Paragraph {
    fn parse(source: &str) -> IResult<&str,Token> {
        let children: Vec<Token> = parse_children(vec![Bold::parse, Italic::parse, Link::parse, InlineCode::parse],source);
        return Ok((
            "",
            Token::Paragraph(Paragraph{ children })
        ));
    }
}

impl Parsable for Italic {
    fn parse(source: &str) -> IResult<&str,Token> {
        let res: IResult<&str, &str> = delimited( char('*'), is_not("*"), char('*'))(source);
        match res {
            Ok((rem, consumed)) => Ok((rem, Token::Italic(Italic{text: consumed.to_string()}))),
            Err(e) => Err(e)
        }
    }
}

impl Parsable for Link {
    fn parse(source: &str) -> IResult<&str,Token> {
        let caption = terminated(
            preceded(tag("["), take_till(|c| c == ']')),
            tag("]"));
        let url = terminated(
            preceded(tag("("), take_till(|c| c == ')')),
            tag(")"));

        let res:IResult<&str, (&str,&str)> = tuple((caption,url))(source);
        match res {
            Ok((rem, (caption,url))) => {
                let children: Vec<Token> = parse_children(vec![Bold::parse, Italic::parse],caption);
                return Ok((
                    rem,
                    Token::Link(Link{children,url: url.to_string()})
                    ))
                }
            ,
            Err(e) => Err(e)
        }
    }
}

impl Parsable for InlineCode {
    fn parse(source: &str) -> IResult<&str,Token> {
        let res: IResult<&str, &str> = terminated(
            preceded(tag("`"), take_till(|c| c == '`')),
            tag("`"))(source);
        match res {
            Ok((rem, consumed)) => Ok((rem, Token::InlineCode(InlineCode{text: consumed.to_string()}))),
            Err(e) => Err(e)
        }
    }
}

impl Parsable for Bold {
    fn parse(source: &str) -> IResult<&str,Token> {
        let res: IResult<&str, &str> = terminated(
            preceded(tag("**"), take_till(|c| c == INLINE_SYMBOL)),
            tag("**"))(source);
        match res {
            Ok((rem, consumed)) => Ok((rem, Token::Bold(Bold{text: consumed.to_string()}))),
            Err(e) => Err(e)
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
