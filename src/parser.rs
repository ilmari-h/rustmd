use nom::{
    IResult,
    error::Error,
    Err,
    sequence::{delimited,preceded,tuple, terminated},
    character::complete::char,
    character::{complete::{newline, digit1}, is_space},
    character::{is_newline, complete::anychar},
    bytes::complete::is_not,
    bytes::complete::tag,
    bytes::complete::take_till,
    bytes::complete::take_until,
    bytes::complete::{take_while, take_while_m_n},
    multi::{many1_count, many1, many_till, many0, fold_many0 },
    error::ErrorKind, Parser, Slice
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

    let (rem,(c_str, token)) = parse_line_consuming_token(input)?;

    // Fill children in syntax tree by folding over each line consumed by the parent Token.
    let mut tree = Tree::new(token.clone());
    let mut stack: VecDeque<(usize, &str, Token)> =
    parse_children( token.child_parsers(), c_str, 1)
        .iter()
        .map(|c| (0,c.0,c.1.clone()))
        .collect();


    while let Some((p_idx,unconsumed, child)) = stack.pop_front() {
        let parsers = child.child_parsers();
        let added = tree.add_node_by_index(TreeIndex::Arena(p_idx), child);
        let idx = added.as_ref().unwrap().raw_idx;
        let depth = added.unwrap().depth;
        let stack_extended: VecDeque<(usize, &str, Token)> =
        parse_children(parsers, unconsumed, depth + 1)
            .iter()
            .map(|c| (idx,c.0,c.1.clone()))
            .collect();
        stack.extend(stack_extended);
    }
    print!("{}",tree);
    return Ok((rem,tree));

}


fn try_all_parsers(
    allowed_parsers: Vec<fn(&str,usize) -> IResult<&str, (&str,Token)>>,
    source: &str,
    depth: usize)
-> IResult<&str,(&str,Token)>{
    if source.is_empty() {
        return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}));
    }
    for parse in allowed_parsers {
        let res = parse(source,depth);
        if res.is_ok() { return res }
    }
    return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}));
}

/**
 Attempt to take a Token. If no Token is found, get leading plain text until
 Token is found or consume all input and return just PlainText.
*/
fn take_tokens_with_leading_plaintext(
    token_parsers: Vec<fn(&str,usize) -> IResult<&str, (&str,Token)>>,
    src: &str,
    depth:usize)
-> IResult<&str,Vec<(&str,Token)>>{

    if src.is_empty(){
            return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}))
    }

    let (rem,consumed) = many0( |s| try_all_parsers(token_parsers.clone(), s, depth))(src)?;
    // If no token found at head of input, consume into PlainText until found token or EOF.
    if consumed.is_empty() {
        let leading_plain_text_chars = many_till(anychar,|s| try_all_parsers(token_parsers.clone(), s, depth))(src);

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
    allowed_children: Vec<fn(&str, usize) -> IResult<&str, (&str,Token)>>,
    src: &str,
    depth: usize)
-> Vec<(&str,Token)> {

    let lines = fold_many0(
        |s| take_tokens_with_leading_plaintext(allowed_children.clone(), s, depth),
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

fn parse_line_consuming_token(source: &str) -> IResult<&str, (&str,Token)> {
    let line_consuming_tokens = [
        Code::parse_lines,
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
     Parse consuming at least one entire line.
     Returns tuple with string containing consumed lines.
    */
    fn parse_lines(source: &str) -> IResult<&str,(&str,Token)>;
}


/**
* Parse a token on a line. Remainder is the remaining line, tuple is string for children and Token
* itself.
*/
pub trait Parse {
    fn parse(source: &str, depth: usize) -> IResult<&str,(&str,Token)>;
}

impl LineConsumingParse for Header {
    fn parse_lines(source: &str) -> IResult<&str,(&str,Token)> {

        let (rem_l,consumed) = take_line(source)?;
        let (rem,count) = terminated(many1_count(tag("#")), tag(" "))(consumed)?;
        return Ok((
            rem_l, // Remaining lines
            (
                rem, // Possible children
                Token::Header(Header{
                    level: count as u32
                })
            )
        ))
    }
}

impl LineConsumingParse for Paragraph {
    fn parse_lines(source: &str) -> IResult<&str,(&str,Token)> {
        let (rem_l, consumed) = take_line(source)?;
        return Ok((
            rem_l, // Remaining lines
            (
                consumed, // Possible children
                Token::Paragraph(Paragraph{})
            )
        ));
    }
}

impl LineConsumingParse for Code {
    fn parse_lines(source: &str) -> IResult<&str,(&str,Token)> {

        let (rem_l,consumed) = preceded( tag("```"), take_until("```"))(source)?;

        return Ok((
            &rem_l[3..], // Remaining lines
            (
                consumed, // Possible children
                Token::Code(Code{})
            )
        ));
    }
}

impl List {

    // Require a depth-amount of leading whitespace to parse a new list.
    fn parse_by_depth(source: &str, depth: usize) -> IResult<&str,(&str,Token)> {

        // List can only be terminated bV
        //   1. Two newlines in a row
        //   2. A newline and a sequence other than "- "
        //   3. "- " preceded by less than depth -amount of whitespace

        // Check if the first line exists
        let res: IResult<&str, &str> = preceded(
            terminated(take_while_m_n(0,depth,|x| is_space(x as u8)), tag("- ")),
            take_until("\n\n"))(source);

        // NOTE: Manual parser!
        if let Ok((_, _)) = res {

            // Take input until terminated by
            //   1. Two newlines in a row
            //   2. A newline and a sequence other than "- "
            let mut previous_newline = false;
            let mut leading_whitespace = 0;
            let mut i = 0;
            for c in source.chars() {
                if previous_newline && ( is_newline(c as u8) || c != '-' ) {
                    if is_space(c as u8) {  // allow leading spaces before lines
                        i+=1;
                        leading_whitespace +=1;
                        continue;
                    }

                    if leading_whitespace < depth {
                        continue;
                    }
                    let consumed_s: &str = &source[0..i];
                    let rem_s: &str = &source[i..];
                    return Ok((
                        rem_s,
                        (
                            consumed_s,
                            Token::List(List{level: depth})
                        )
                    ))
                }
                i+=1;
                previous_newline = is_newline(c as u8);
            }
        }
        return Err(Err::Error(Error{input: "", code: ErrorKind::Satisfy}));
    }
}

impl LineConsumingParse for List {
    fn parse_lines(source: &str) -> IResult<&str,(&str,Token)> {
        List::parse_by_depth(source,0)
    }
}

impl Parse for List {
    fn parse(source: &str,depth:usize) -> IResult<&str,(&str,Token)> {
        List::parse_by_depth(source,depth)
    }
}

impl Parse for Italic {
    fn parse(source: &str,_:usize) -> IResult<&str,(&str,Token)> {
        let (rem,consumed) = delimited( char('*'), is_not("*"), char('*'))(source)?;
        Ok((
            rem,
            (consumed,Token::Italic(Italic{}))
        ))
    }
}

impl Parse for Link {
    fn parse(source: &str,_:usize) -> IResult<&str,(&str,Token)> {
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
    fn parse(source: &str,_:usize) -> IResult<&str,(&str,Token)> {
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
    fn parse(source: &str, _:usize) -> IResult<&str,(&str,Token)> {
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
    fn parse(source: &str,_:usize) -> IResult<&str,(&str,Token)> {
        let preciding_whitespace = terminated(take_while(|x| is_space(x as u8)), tag("- "));
        let mut list_line = preceded(
            preciding_whitespace, take_line);
        let (rem, consumed) = list_line(source)?;
        return Ok((rem,(consumed, Token::ListItem(ListItem{}))));
    }
}

pub trait HigherLevel {
    fn child_parsers(&self) -> Vec<fn(&str,usize) -> IResult<&str, (&str,Token)>>;
}

impl HigherLevel for Token {
    fn child_parsers(&self) -> Vec<fn(&str,usize) -> IResult<&str, (&str,Token)>> {
        match self {
            Token::Header(_) => vec![Italic::parse, Bold::parse, Link::parse, InlineCode::parse],
            Token::Paragraph(_) => vec![Italic::parse, Bold::parse, Link::parse, InlineCode::parse],
            Token::List(_) => vec![List::parse,ListItem::parse],
            Token::Code(_) => vec![],
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
mod tests {
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
- Third item

"
        );
        let expected_order: Vec<Token> = Vec::from([
            Token::List(List{level: 0}),
            Token::ListItem(ListItem{}),
            Token::PlainText(PlainText{text: String::from("First item")}),
            Token::ListItem(ListItem{}),
            Token::PlainText(PlainText{text: String::from("Second item")}),
            Token::ListItem(ListItem{}),
            Token::PlainText(PlainText{text: String::from("Third item")}),
            Token::Paragraph(Paragraph{}),
        ]);
        match_syntax(md_syntax, expected_order);
    }

}
