pub mod token;

use std::{
    collections::{vec_deque, VecDeque},
    str::CharIndices,
};

use thiserror::Error;
use token::{Kind, Token};

#[derive(Error, Debug, PartialEq)]
pub enum LexerError {
    #[error("Unexpected end of")]
    UnexpectedEof,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(Kind),
    #[error("Unexpected char: {0}")]
    UnexpectedChar(char),
}

#[derive(Debug, PartialEq, Eq)]
enum NumberContext {
    Normal,
    Dot,
    Plus,
    Minus,
}
#[derive(Debug, PartialEq, Eq)]
enum StringContext {
    Single,
    Double,
}

#[derive(Debug)]
pub struct Lexer<'source> {
    source: &'source str,

    token_stash: VecDeque<Token>,

    pub chars: CharIndices<'source>,

    stack_charts: VecDeque<(CharIndices<'source>, VecDeque<Token>)>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Lexer {
            source,
            token_stash: VecDeque::with_capacity(30),
            chars: source.char_indices(),
            stack_charts: VecDeque::with_capacity(20),
        }
    }
    pub fn advance(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    pub fn cur_char(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }
    pub fn peek_char(&mut self) -> Option<(usize, char)> {
        self.chars.clone().next()
    }
    pub fn start(&mut self) {
        self.stack_charts
            .push_back((self.chars.clone(), self.token_stash.clone()));
    }
    pub fn restore(&mut self) {
        if let Some((chars, stash)) = self.stack_charts.pop_back() {
            self.chars = chars;
            self.token_stash = stash;
        }
    }

    pub fn eat_until_end_line(&mut self) -> usize {
        while let Some((pos, ch)) = self.cur_char() {
            if ch == '\n' {
                return pos;
            }
        }
        self.source.len()
    }

    fn is_at_equal(&mut self) -> bool {
        if let Some((_, ch)) = self.peek_char() {
            if ch == '=' {
                return true;
            }
        }
        false
    }

    fn get_token(&mut self) -> Result<Token, LexerError> {
        while let Some((pos, ch)) = self.cur_char() {
            match ch {
                '(' => {
                    return Ok(Token::new(Kind::LeftParen, pos, pos + 1));
                }
                ')' => {
                    return Ok(Token::new(Kind::RightParen, pos, pos + 1));
                }
                '[' => {
                    return Ok(Token::new(Kind::LeftBracket, pos, pos + 1));
                }
                ']' => {
                    return Ok(Token::new(Kind::RightBracket, pos, pos + 1));
                }
                '{' => {
                    return Ok(Token::new(Kind::LeftBrace, pos, pos + 1));
                }
                '}' => {
                    return Ok(Token::new(Kind::RightBrace, pos, pos + 1));
                }
                ',' => {
                    return Ok(Token::new(Kind::Comma, pos, pos + 1));
                }
                ';' => {
                    return Ok(Token::new(Kind::Semicolon, pos, pos + 1));
                }
                ':' => {
                    return Ok(Token::new(Kind::Colon, pos, pos + 1));
                }
                '=' => {
                    return Ok(Token::new(Kind::Equals, pos, pos + 1));
                }
                '+' => {
                    return self.parse_number_token(pos, NumberContext::Plus);
                }
                '-' => {
                    return self.parse_number_token(pos, NumberContext::Minus);
                }
                _ if ch.is_ascii_digit() => {
                    return self.parse_number_token(pos, NumberContext::Normal);
                }
                '*' => {
                    return Ok(Token::new(Kind::Asterisk, pos, pos + 1));
                }
                '/' => return self.try_comment(pos),
                '%' => {
                    return Ok(Token::new(Kind::Percent, pos, pos + 1));
                }
                '^' => {
                    if self.is_at_equal() {
                        self.advance();
                        return Ok(Token::new(Kind::CaretEquals, pos, pos + 2));
                    }
                    return Ok(Token::new(Kind::Caret, pos, pos + 1));
                }
                '|' => {
                    if self.is_at_equal() {
                        self.advance();
                        return Ok(Token::new(Kind::PipeEquals, pos, pos + 2));
                    }
                    return Ok(Token::new(Kind::Pipe, pos, pos + 1));
                }
                '~' => {
                    if self.is_at_equal() {
                        self.advance();
                        return Ok(Token::new(Kind::TildeEquals, pos, pos + 2));
                    }
                    return Ok(Token::new(Kind::Tilde, pos, pos + 1));
                }
                '$' => {
                    if self.is_at_equal() {
                        self.advance();
                        return Ok(Token::new(Kind::DollarEquals, pos, pos + 2));
                    }
                    return Ok(Token::new(Kind::DollarEquals, pos, pos + 1));
                }
                '.' => {
                    return self.parse_number_token(pos, NumberContext::Dot);
                }
                '>' => {
                    return Ok(Token::new(Kind::GreaterThan, pos, pos + 1));
                }
                '&' => {
                    return Ok(Token::new(Kind::Ampersand, pos, pos + 1));
                }
                '!' => {
                    return Ok(Token::new(Kind::Bang, pos, pos + 1));
                }
                '@' => {
                    return self.parse_at_word(pos);
                }
                '#' => {
                    return Ok(Token::new(Kind::Hash, pos, pos + 1));
                }
                '\'' => {
                    return self.parse_string(pos, StringContext::Single);
                }
                '"' => {
                    return self.parse_string(pos, StringContext::Double);
                }
                _ if ch.is_whitespace() => {
                    if ch == ' ' {
                        loop {
                            if let Some((_, ch)) = self.peek_char() {
                                if ch == ' ' {
                                    self.advance();
                                    continue;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        return Ok(Token::new(Kind::Whitespace, pos, pos + 1));
                    }
                    continue;
                }
                _ => {
                    if Self::is_validate_ident(ch, false) {
                        return self.parse_ident_token(pos);
                    }
                    return Err(LexerError::UnexpectedChar(ch));
                }
            }
        }
        Ok(Token::new(Kind::EOF, self.source.len(), self.source.len()))
    }

    fn try_comment(&mut self, start: usize) -> Result<Token, LexerError> {
        if let Some((_, char)) = self.peek_char() {
            // single line comment
            if char == '/' {
                let end_pos = self.eat_until_end_line();
                return Ok(Token::new(Kind::Comment, start, end_pos));
            }
            // multi line comment
            if char == '*' {
                while let Some((_, ch)) = self.cur_char() {
                    if ch == '*' && matches!(self.peek_char(), Some((_, '/'))) {
                        let (end, _) = self.advance().unwrap();
                        return Ok(Token::new(Kind::Comment, start, end));
                    }
                }
                return Err(LexerError::UnexpectedEof);
            }
        }
        return Ok(Token::new(Kind::Slash, start, start + 1));
    }

    fn is_validate_ident(ch: char, in_ident: bool) -> bool {
        match ch {
            '_' => return true,
            ch if ch.is_ascii_alphabetic() => return true,
            '-' => return true,
            '\u{00b7}' => return true,
            '\u{00c0}'..='\u{00d6}' => return true,
            '\u{00d8}'..='\u{00f6}' => return true,
            '\u{00f8}'..='\u{03ff}' => return true,
            '\u{037f}'..='\u{1fff}' => return true,
            '\u{200c}' | '\u{200d}' | '\u{203f}' | '\u{2040}' => return true,
            '\u{2070}'..='\u{218f}' => return true,
            '\u{2c00}'..='\u{2fef}' => return true,
            '\u{3001}'..='\u{d7ff}' => return true,
            '\u{f900}'..='\u{fdcf}' => return true,
            '\u{fdf0}'..='\u{fffd}' => return true,
            '\u{10000}'.. => return true,
            '0'..='9' if in_ident => return true,
            _ => return false,
        }
    }

    fn is_at_ident_token(&mut self) -> (bool, usize) {
        if let Some((pos, ch)) = self.peek_char() {
            if Self::is_validate_ident(ch, false) {
                return (true, pos);
            }
        }
        (false, 0)
    }

    fn is_at_string(&mut self) -> bool {
        if let Some((_, ch)) = self.peek_char() {
            if ch == '"' || ch == '\'' {
                return true;
            }
        }
        false
    }

    fn parse_string(&mut self, start: usize, context: StringContext) -> Result<Token, LexerError> {
        let mut end_pos = start + 1;
        while let Some((pos, ch)) = self.cur_char() {
            match ch {
                '\'' => {
                    if matches!(context, StringContext::Single) {
                        return Ok(Token::new(Kind::String, start, pos + 1));
                    } else {
                        continue;
                    }
                }
                '"' => {
                    if matches!(context, StringContext::Double) {
                        return Ok(Token::new(Kind::String, start, pos + 1));
                    } else {
                        continue;
                    }
                }
                '\\' => {
                    self.advance();
                    continue;
                }
                '\n' => {
                    return Err(LexerError::UnexpectedChar(ch));
                }
                _ => {
                    continue;
                }
            }
        }
        Err(LexerError::UnexpectedEof)
    }

    fn parse_number_token(
        &mut self,
        start: usize,
        number_context: NumberContext,
    ) -> Result<Token, LexerError> {
        let mut current_number_context = number_context;
        let mut end_pos = start + 1;
        while let Some((pos, ch)) = self.peek_char() {
            match ch {
                _ if ch.is_ascii_digit() => {
                    self.advance();
                }
                '.' => {
                    if matches!(current_number_context, NumberContext::Normal) {
                        current_number_context = NumberContext::Dot;
                        self.advance();
                        continue;
                    }
                    end_pos = pos;
                    break;
                }
                _ => {
                    end_pos = pos;
                    break;
                }
            }
        }
        if end_pos != start + 1 {
            return Ok(Token::new(Kind::Number, start, end_pos));
        }
        match current_number_context {
            NumberContext::Normal => return Ok(Token::new(Kind::Number, start, end_pos)),
            NumberContext::Dot => return Ok(Token::new(Kind::Dot, start, end_pos)),
            NumberContext::Plus => return Ok(Token::new(Kind::Plus, start, end_pos)),
            NumberContext::Minus => return Ok(Token::new(Kind::Minus, start, end_pos)),
        }
    }

    fn parse_ident_token(&mut self, start: usize) -> Result<Token, LexerError> {
        while let Some((end, ch)) = self.peek_char() {
            if Self::is_validate_ident(ch, true) {
                self.advance();
                continue;
            } else {
                return Ok(Token::new(Kind::Ident, start, end));
            }
        }
        Ok(Token::new(Kind::Ident, start, self.source.len()))
    }

    fn parse_at_word(&mut self, start: usize) -> Result<Token, LexerError> {
        let (is_at_ident, ident_start) = self.is_at_ident_token();
        if is_at_ident {
            let ident_token = self.parse_ident_token(ident_start)?;
            return Ok(Token::new(Kind::AtKeyword, start, ident_token.end));
        }
        Ok(Token::new(Kind::Asterisk, start, start + 1))
    }

    pub fn next(&mut self) -> Result<Token, LexerError> {
        if !self.token_stash.is_empty() {
            return Ok(self.token_stash.pop_front().unwrap());
        }
        loop {
            let token = self.get_token()?;
            if token.kind == Kind::Comment {
                continue;
            } else {
                return Ok(token);
            }
        }
    }

    pub fn peek(&mut self) -> Result<&Token, LexerError> {
        if !self.token_stash.is_empty() {
            return Ok(self.token_stash.front().unwrap());
        }
        loop {
            let token = self.get_token()?;
            if token.kind == Kind::Comment {
                continue;
            } else {
                self.token_stash.push_back(token);
                return Ok(self.token_stash.front().unwrap());
            }
        }
    }

    pub fn peek_nth(&mut self, n: usize) -> Result<&Token, LexerError> {
        if self.token_stash.get(n).is_some() {
            return Ok(self.token_stash.get(n).unwrap());
        }

        let nth = if self.token_stash.is_empty() {
            n
        } else {
            n - self.token_stash.len()
        };

        for _ in 0..=nth {
            loop {
                let token = self.get_token()?;
                if token.kind == Kind::Comment {
                    continue;
                } else {
                    self.token_stash.push_back(token);
                    break;
                }
            }
        }

        return Ok(self.token_stash.get(n).unwrap());
    }

    pub fn peek_skip_whitespace(&mut self) -> Result<&Token, LexerError> {
        loop {
            if !self.token_stash.is_empty() {
                return Ok(self.token_stash.back().unwrap());
            }
            let token = self.get_token()?;

            if token.kind == Kind::Whitespace {
                continue;
            }

            self.token_stash.push_back(token);
            return Ok(self.token_stash.back().unwrap());
        }
    }
    pub fn next_skip_whitespace(&mut self) -> Result<Token, LexerError> {
        loop {
            if !self.token_stash.is_empty() {
                return Ok(self.token_stash.pop_front().unwrap());
            }
            let token = self.get_token()?;

            if token.kind == Kind::Whitespace {
                continue;
            }

            return Ok(token);
        }
    }

    pub fn eat() {}

    pub fn expect(kind: Kind) {}

    fn debug_token(&self, token: &Token) {
        println!("{:?}", self.source[token.start..token.end].to_string());
    }
}

#[test]
fn comment() {
    let code = r#"
///a
/* comment */
//a
/* a"#;
    let mut lex = Lexer::new(code);
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Comment, 1, 5)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Comment, 6, 18)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Comment, 20, 23)));
    assert_eq!(lex.get_token(), Err(LexerError::UnexpectedEof))
}

#[test]
fn number() {
    let code = r#"
1
0.1
+1
-1
.1
.
+
-"#;
    let mut lex = Lexer::new(code);
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Number, 1, 2)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Number, 3, 6)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Number, 7, 9)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Number, 10, 12)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Number, 13, 15)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Dot, 16, 17)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Plus, 18, 19)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Minus, 20, 21)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::EOF, 21, 21)));
}

#[test]
fn ident() {
    let code = r#"
a
你好
🚗
👪
"#;
    let mut lex = Lexer::new(code);
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Ident, 1, 2)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Ident, 3, 9)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Ident, 10, 14)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Ident, 15, 19)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::EOF, 19, 19)));
}

#[test]
fn at_ident() {
    let code = r#"
@a
@你好
@🚗
@👪
@@a
"#;
    let mut lex = Lexer::new(code);
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::AtKeyword, 1, 3)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::AtKeyword, 4, 11)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::AtKeyword, 12, 17)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::AtKeyword, 18, 23)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::Asterisk, 24, 25)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::AtKeyword, 25, 27)));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::EOF, 27, 27)));
}

#[test]
fn next_and_peek() {
    let code = r#"
a b c
"#;
    let mut lex = Lexer::new(code);
    assert_eq!(lex.next(), Ok(Token::new(Kind::Ident, 1, 2)));
    assert_eq!(lex.peek(), Ok(&Token::new(Kind::Whitespace, 2, 3)));
    assert_eq!(lex.peek(), Ok(&Token::new(Kind::Whitespace, 2, 3)));
    assert_eq!(lex.next(), Ok(Token::new(Kind::Whitespace, 2, 3)));
    assert_eq!(lex.next(), Ok(Token::new(Kind::Ident, 3, 4)));
    assert_eq!(lex.next(), Ok(Token::new(Kind::Whitespace, 4, 5)));
    assert_eq!(lex.next(), Ok(Token::new(Kind::Ident, 5, 6)));
    assert_eq!(lex.next(), Ok(Token::new(Kind::EOF, 7, 7)));
}

#[test]
fn next_and_peek_skip_whitespace() {
    let code = r#"
a b c
"#;
    let mut lex = Lexer::new(code);
    assert_eq!(lex.next(), Ok(Token::new(Kind::Ident, 1, 2)));
    assert_eq!(
        lex.peek_skip_whitespace(),
        Ok(&Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.peek_skip_whitespace(),
        Ok(&Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.next_skip_whitespace(),
        Ok(Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.next_skip_whitespace(),
        Ok(Token::new(Kind::Ident, 5, 6))
    );
    assert_eq!(lex.next(), Ok(Token::new(Kind::EOF, 7, 7)));
}

#[test]
fn string() {
    let code = r#"
'123'
"abc"
"汽车"
'🚗'
"'"
'"'
'\''
"\""
"#;
    let mut lex = Lexer::new(code);
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 1, 6));
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 7, 12));
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 13, 21));
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 22, 28));
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 29, 32));
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 33, 36));
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 37, 41));
    let token = lex.get_token().unwrap();
    lex.debug_token(&token);
    assert_eq!(token, Token::new(Kind::String, 42, 46));
    assert_eq!(lex.get_token(), Ok(Token::new(Kind::EOF, 47, 47)));
}

#[test]
fn start_and_restore() {
    let code = r#"
a b c
    "#;
    let mut lex = Lexer::new(code);
    lex.start();
    assert_eq!(lex.next(), Ok(Token::new(Kind::Ident, 1, 2)));
    assert_eq!(
        lex.peek_skip_whitespace(),
        Ok(&Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.peek_skip_whitespace(),
        Ok(&Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.next_skip_whitespace(),
        Ok(Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.next_skip_whitespace(),
        Ok(Token::new(Kind::Ident, 5, 6))
    );
    assert_eq!(lex.next(), Ok(Token::new(Kind::Whitespace, 7, 8)));
    lex.restore();
    assert_eq!(lex.next(), Ok(Token::new(Kind::Ident, 1, 2)));
    assert_eq!(
        lex.peek_skip_whitespace(),
        Ok(&Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.peek_skip_whitespace(),
        Ok(&Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.next_skip_whitespace(),
        Ok(Token::new(Kind::Ident, 3, 4))
    );
    assert_eq!(
        lex.next_skip_whitespace(),
        Ok(Token::new(Kind::Ident, 5, 6))
    );
}

#[test]
fn peek_nth() {
    let code = r#"
a  b c
"#;
    let mut lex = Lexer::new(code);
    assert_eq!(lex.next(), Ok(Token::new(Kind::Ident, 1, 2)));
    assert_eq!(lex.peek_nth(0), Ok(&Token::new(Kind::Whitespace, 2, 3)));
    assert_eq!(lex.peek_nth(1), Ok(&Token::new(Kind::Ident, 4, 5)));
    assert_eq!(lex.peek_nth(2), Ok(&Token::new(Kind::Whitespace, 5, 6)));
    assert_eq!(lex.peek_nth(3), Ok(&Token::new(Kind::Ident, 6, 7)));
    assert_eq!(lex.next(), Ok(Token::new(Kind::Whitespace, 2, 3)));
}

#[test]
fn quick_test() {
    println!("\\");
}
