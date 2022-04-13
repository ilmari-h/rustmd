use std::fmt;

/**
 * Tokens contain information about markdown elements in a general manner,
 * without dictating any details about how they may eventually be compiled.
 * In other words: Tokens are a compilation target agnostic way to represent
 * the syntax tree of a markdown file.
 */

// ----------------------------------------------------------------------------
// TYPES
// ----------------------------------------------------------------------------

pub enum Token {
    Header(Header),
    Paragraph(Paragraph),
    PlainText(PlainText),
    Italic(Italic)
}

pub trait HigherLevel {
    fn children(&self) -> &Vec<Token>;
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

pub struct Header {
    pub children: Vec<Token>,
    pub level: u32
}

pub struct Paragraph {
    pub children: Vec<Token>
}

pub struct PlainText {
    pub text: String
}

pub struct Italic {
    pub text: String
}

pub struct Link {
    children: Vec<Token>,
    url: String
}

// ----------------------------------------------------------------------------
// TRAIT IMPLEMENTATIONS FOR TYPES
// ----------------------------------------------------------------------------

// TextComponents are generally inline and do not have children
impl TextComponent for PlainText {
    fn text(&self) -> String {
        self.text.clone()
    }
}

impl TextComponent for Italic {
    fn text(&self) -> String {
        self.text.clone()
    }
}

impl Leveled for Header {
    fn level(&self) -> u32 {
        self.level
    }
}

macro_rules! impl_HigherLevel {
    (for $($t:ty),+) => {
        $(impl HigherLevel for $t {
            fn children(&self) -> &Vec<Token> {
                &self.children
            }
        })*
    }
}

impl_HigherLevel!(for Header, Paragraph);

// ----------------------------------------------------------------------------
// DEBUG IMPLEMENTATIONS
// ----------------------------------------------------------------------------

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
         .field("children", &self.children().len())
         .field("level", &self.level())
         .finish()
    }
}

impl fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Paragraph")
         .field("children", &self.children().len())
         .finish()
    }
}

impl fmt::Debug for PlainText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlainText")
            .field("content", &self.text())
            .finish()
    }
}

impl fmt::Debug for Italic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Italic")
            .field("content", &self.text())
            .finish()
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Header(h) => return h.fmt(f),
            Token::PlainText(t) => return t.fmt(f),
            Token::Italic(t) => return t.fmt(f),
            Token::Paragraph(t) => return t.fmt(f)
        }
    }
}

// ----------------------------------------------------------------------------
// MD STRUCTURE TYPE ALIASES
// ----------------------------------------------------------------------------

pub type MdLine = Vec<Token>;
pub type MdSyntaxTree = Vec<MdLine>;

