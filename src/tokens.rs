use std::fmt;

use crate::tree::*;

/**
 * Tokens contain information about markdown elements in a general manner,
 * without dictating any details about how they may eventually be compiled.
 * In other words: Tokens are a compilation target agnostic way to represent
 * the syntax tree of a markdown file.
 */

// ----------------------------------------------------------------------------
// TYPES
// ----------------------------------------------------------------------------

#[derive(PartialEq)]
#[derive(Clone)]
pub enum Token {
    Header(Header),
    Paragraph(Paragraph),
    List(List),
    Code(Code),
    PlainText(PlainText),
    Italic(Italic),
    InlineCode(InlineCode),
    Link(Link),
    Bold(Bold),
    ListItem(ListItem),
}

pub trait Leveled {
    fn level(&self) -> u32;
}

pub trait Src {
    fn src(&self) -> String;
}

pub trait TextComponent {
    fn text(&self) -> String;
}

// ----------------------------------------------------------------------------
// TOKENS
// ----------------------------------------------------------------------------

// Higher level / Line consuming

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct List {
    pub level: usize
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct OrderedList {
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Header {
    pub level: u32
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Paragraph {
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Code {
}

// Lower level / Inline

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct PlainText {
    pub text: String
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Italic {
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct InlineCode {
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Bold {
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Link {
    pub url: String
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct ListItem {
}

// ----------------------------------------------------------------------------
// TRAIT IMPLEMENTATIONS FOR TYPES
// ----------------------------------------------------------------------------


impl Leveled for Header {
    fn level(&self) -> u32 {
        self.level
    }
}

impl Leveled for List {
    fn level(&self) -> u32 {
        self.level as u32
    }
}

impl TextComponent for PlainText {
    fn text(&self) -> String {
        self.text.clone()
    }
}

// TextComponents are generally inline and do not have children

// ----------------------------------------------------------------------------
// DEBUG IMPLEMENTATIONS
// ----------------------------------------------------------------------------


impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Header(h) => return h.fmt(f),
            Token::PlainText(t) => return t.fmt(f),
            Token::Italic(t) => return t.fmt(f),
            Token::Link(t) => return t.fmt(f),
            Token::List(t) => return t.fmt(f),
            Token::Bold(t) => return t.fmt(f),
            Token::Paragraph(t) => return t.fmt(f),
            Token::InlineCode(t) => return t.fmt(f),
            Token::ListItem(t) => return t.fmt(f),
            Token::Code(t) => return t.fmt(f)
        }
    }
}

// ----------------------------------------------------------------------------
// MD STRUCTURE TYPE ALIASES
// ----------------------------------------------------------------------------

pub type MdLine = Tree<Token>;
pub type MdSyntaxTree = Vec<MdLine>;

