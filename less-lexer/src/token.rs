use std::path::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Kind {
    Ident,
    AtKeyword,
    String,
    Number,
    Comment,
    Color,

    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Colon,        // :
    Semicolon,    // ;
    Equals,       // =
    Minus,        // -
    Plus,         // +
    Asterisk,     // *
    Slash,        // /
    Percent,      // %
    Caret,        // ^
    CaretEquals,  // ^=
    Tilde,        // ~
    TildeEquals,  // ~=
    Pipe,         // |
    PipeEquals,   // |=
    DollarEquals, // $=
    GreaterThan,  // >
    Ampersand,    // &
    Bang,         // !
    Hash,         // #

    Dot, // .

    Whitespace,
    EOF,
}

impl Token {
    pub fn new(kind: Kind, start: usize, end: usize) -> Token {
        Token { kind, start, end }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return self.kind.fmt(f);
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Ident => write!(f, "Ident"),
            Kind::AtKeyword => write!(f, "AtKeyword"),
            Kind::String => write!(f, "String"),
            Kind::Number => write!(f, "Number"),
            Kind::Minus => write!(f, "Minus"),
            Kind::Plus => write!(f, "Plus"),
            Kind::Asterisk => write!(f, "Asterisk"),
            Kind::Slash => write!(f, "Slash"),
            Kind::Equals => write!(f, "Equals"),
            Kind::TildeEquals => write!(f, "TildeEquals"),
            Kind::PipeEquals => write!(f, "PipeEquals"),
            Kind::CaretEquals => write!(f, "CaretEquals"),
            Kind::DollarEquals => write!(f, "DollarEquals"),
            Kind::GreaterThan => write!(f, "GreaterThan"),
            Kind::Tilde => write!(f, "Tilde"),
            Kind::Pipe => write!(f, "Pipe"),
            Kind::Caret => write!(f, "Caret"),
            Kind::Ampersand => write!(f, "Ampersand"),
            Kind::Comma => write!(f, "Comma"),
            Kind::Colon => write!(f, "Colon"),
            Kind::Semicolon => write!(f, "Semicolon"),
            Kind::LeftParen => write!(f, "LeftParen"),
            Kind::RightParen => write!(f, "RightParen"),
            Kind::LeftBracket => write!(f, "LeftBracket"),
            Kind::RightBracket => write!(f, "RightBracket"),
            Kind::LeftBrace => write!(f, "LeftBrace"),
            Kind::RightBrace => write!(f, "RightBrace"),
            Kind::Whitespace => write!(f, "Whitespace"),
            Kind::Comment => write!(f, "Comment"),
            Kind::EOF => write!(f, "EOF"),
            Kind::Bang => write!(f, "Bang"),
            Kind::Hash => write!(f, "Hash"),
            Kind::Dot => write!(f, "Dot"),
            Kind::Percent => write!(f, "Percent"),
            Kind::Color => write!(f, "Color"),
        }
    }
}
