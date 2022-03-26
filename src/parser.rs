use nom::{
    IResult,
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
    multi::many1_count // Get all lines and apply syntax parsing
};
#[macro_use]

#[path = "tokens.rs"] mod tokens;
use tokens::*;

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
    println!("{:?}", lines);

    return lines.unwrap().1;
}

pub fn parse_line(line_str: &str) -> MdLine {

    // Do this until errored
    let res = Header::parse(line_str);
    match res {
        Ok((rem, token)) => {
            println!("{:?}", token.level);
            let v = vec![Token::Header(token)];
            return v;
        },
        Err(_) => {
            println!("{:?}", line_str);
            let txt = PlainText {
                text: line_str.to_string()
            };
            return vec![Token::PlainText(txt)];
        }
    }
}

// Implement Parsable for tokens
pub trait Parsable<Token> {
    fn parse(source: &str) -> IResult<&str,Token>;
}

impl Parsable<Header> for Header {
    fn parse(source: &str) -> IResult<&str,Header> {

        let count_r: IResult<&str, usize> = many1_count(tag("#"))(source);
        match count_r {
            Ok((rem, count)) => {
                return Ok((
                    rem,
                    Header{
                        children: vec![],
                        level: count as u32
                    }))
            },
            Err(e) => return Err(e)
        };
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
