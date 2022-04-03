use nom::{
    IResult,
    error::Error,
    Err,
    sequence::delimited,
    sequence::pair,
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
    character::complete::newline,
    character::complete::line_ending,
    character::is_newline,
    bytes::complete::is_not,
    bytes::complete::tag,
    bytes::complete::take_till,
    sequence::terminated,
    sequence::preceded, // Good for matching markdown tags
    multi::fold_many0 ,// Get all lines and apply syntax parsing
    multi::many1_count, // Get all lines and apply syntax parsing
    multi::many0, error::ErrorKind
};
//#[macro_use]
use crate::tokens::*;

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

    let mut line: MdLine = vec![];
    let line_after_first;

    // Test first token, in case it's a line consuming Token. In this case all input is consumed.
    // Eg. A header.
    let first_token = Token::parse(line_str, true);

    match first_token {
        Ok(token) => {
            // All input consumed.
            println!("{:?}", token.1);
            if token.0.is_empty() { return vec![token.1] }

            // Else, continue parsing line.
            line.push(token.1);
            line_after_first = token.0;
        }
        Err(_) => {

            // Empty line
            return vec![Token::PlainText(PlainText{text: "".to_string()})];
        }
    }

    // Consume the following input.
    let res = many0(|input| Token::parse(input, false))(line_after_first);
    println!("{:?}", res);

    // TODO: Use fold:
    // If item is HigherLevel, all following items are children.
    // Otherwise, add items to vector in sequence.

    // TODO: concat line and res
    return res.unwrap().1;
}

// Implement Parsable for tokens
pub trait Parsable { // Should this just be a function of Token returning Token
    fn parse(source: &str, first_token: bool) -> IResult<&str,Token>;
}
impl Parsable for Token {
    fn parse(source: &str, first_token: bool) -> IResult<&str,Token> { // this function might also need to know if on top level, to prevent eg. Headers within Headers

        // Empty input, parsing done. Return error.
        if source.is_empty() {
            let err = Error{input: "", code: ErrorKind::Satisfy};
            return Err(Err::Error(err));
        }

        let line_consuming_parse_functions = [
            Header::parse,

        ];

        // Parse for all Tokens, except PlainText, which is the default case.

        // Accept line consuming tokens only if on first token of this line
        if first_token {
            for parse in line_consuming_parse_functions {
                let res = parse(source, false);
                if res.is_ok() { return res }
            }
        }

        // TODO: other parse functions

        // No Tokens found, return PlainText
        return Ok(("", Token::PlainText(PlainText{text: source.to_string()})));
    }
}

impl Parsable for Header {
    fn parse(source: &str, _: bool) -> IResult<&str,Token> {

        let count_r: IResult<&str, usize> = terminated(many1_count(tag("#")), tag(" "))(source);
        match count_r {
            Ok((rem, count)) => {

                // Here call some generic method to consume all children until empty remainder.
                let parsed_remainder = Token::parse(rem, false);

                match parsed_remainder {
                    Ok((_, c_token)) => {
                        return Ok((
                            "",
                            Token::Header(Header{
                                children: vec![c_token],
                                level: count as u32
                            })
                        ))
                    },
                    Err(_) => {
                        return Ok((
                            "",
                            Token::Header(Header{
                                children: vec![],
                                level: count as u32
                            })
                        ))
                    }
                }
            },
            Err(e) => return Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_take_line() {
        assert_eq!(("Line 2\nLine 3", "Line 1"), take_line("Line 1\nLine 2\nLine 3").unwrap());
        assert_eq!(true, take_line("Last Line").is_ok());
        assert_eq!(true, take_line("\nLast Line").is_ok());
        assert_eq!(true, take_line("\n\n\nLast Line").is_ok());
        assert_eq!(true, take_line("").is_err());
    }

    #[test]
    fn t_parse_md_str() {
        assert_eq!(true,parse_md_str("### First header\nHere's some plain text\n# Last header").len() > 0);
    }
}
