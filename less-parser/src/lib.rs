use less_ast::ast::{
    AtKeyword, AtRule, Atom, ComponentValue, ComponentValueList, CurlyBracketsBlock,
    CurlyBracketsBlockContent, Declaration, DeclarationList, DefinedStatement, Ident, LexerToken,
    MapVariableDefined, NumberLiteral, PreservedToken, PseudoElement, PseudoFunction,
    PseudoSelector, QualifiedRule, Selector, SelectorComponentList, SelectorList, SimpleSelector,
    Span, StringLiteral, StyleContent, Stylesheets, VariableDefined, VariableDefinedValue,
    VariableValueComponentList, VariableValueList,
};
use less_lexer::{
    token::{Kind, Token},
    Lexer,
};
use thiserror::Error;

pub struct Parser<'source> {
    lexer: Lexer<'source>,
    source: &'source str,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Lexer::new(source),
            source,
        }
    }
}

#[derive(Debug, Error)]
enum ParserError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(Token),
    #[error("Lexer error: {0}")]
    LexerError(#[from] less_lexer::LexerError),
    #[error("Parse number error: {0}")]
    ParseNUmberError(#[from] std::num::ParseFloatError),
}

impl<'source> Parser<'source> {
    pub fn next_token(&mut self) -> Result<Token, ParserError> {
        let token = self.lexer.next_skip_whitespace()?;
        Ok(token)
    }
    pub fn peek_token(&mut self) -> Result<&Token, ParserError> {
        Ok(self.lexer.peek_skip_whitespace()?)
    }
    pub fn peek_nth_token(&mut self, n: usize) -> Result<&Token, ParserError> {
        Ok(self.lexer.peek_nth(n)?)
    }
    pub fn peek_with_whitespace(&mut self) -> Result<&Token, ParserError> {
        Ok(self.lexer.peek_skip_whitespace()?)
    }
    pub fn expect(&mut self, kind: Kind) -> Result<Token, ParserError> {
        let token = self.next_token()?;
        if token.kind == kind {
            Ok(token)
        } else {
            Err(ParserError::UnexpectedToken(token))
        }
    }
    pub fn get_atom(&self, token: &Token) -> Atom {
        self.source[token.start..token.end].to_string()
    }
    pub fn get_atom_by_span(&self, start: usize, end: usize) -> Atom {
        self.source[start..end].to_string()
    }
    pub fn get_float_number(&self, token: &Token) -> Result<f64, ParserError> {
        Ok(self.source[token.start..token.end].parse::<f64>()?)
    }

    pub fn parse(&mut self) -> Result<Stylesheets, ParserError> {
        let mut content = Vec::new();
        while let Ok(token) = self.peek_token() {
            match token.kind {
                Kind::AtKeyword => {
                    if self.is_at_defined_statement() {
                        let statement = self.parse_variable_defined()?;
                        content.push(StyleContent::DefinedStatement(statement));
                    } else {
                        let rule = self.parse_at_rule()?;
                        content.push(StyleContent::AtRule(rule));
                    }
                }
                Kind::Ident => {
                    let rule = self.parse_rule()?;
                    content.push(StyleContent::QualifiedRule(rule));
                }
                Kind::EOF => break,
                _ => return Err(ParserError::UnexpectedToken(self.next_token()?)),
            }
        }
        Ok(Stylesheets {
            span: Span::new(0, self.source.len()),
            content,
        })
    }

    fn parse_at_rule(&mut self) -> Result<AtRule, ParserError> {
        todo!()
    }

    fn is_at_defined_statement(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            if matches!(token.kind, Kind::AtKeyword) {
                if let Ok(token) = self.peek_nth_token(1) {
                    return matches!(token.kind, Kind::Colon);
                }
            }
        }
        false
    }
    fn is_at_map_defined(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            if matches!(token.kind, Kind::AtKeyword) {
                if let Ok(token) = self.peek_nth_token(1) {
                    return matches!(token.kind, Kind::LeftBrace);
                }
            }
        }
        false
    }
    fn parse_variable_defined(&mut self) -> Result<DefinedStatement, ParserError> {
        let token = self.expect(Kind::AtKeyword)?;
        self.expect(Kind::Colon)?;
        if self.is_at_map_defined() {
            let map = self.parse_map_defined()?;
            return Ok(DefinedStatement::MapVariableDefined(map));
        }
        let value = self.parse_value_defined(token)?;
        Ok(DefinedStatement::VariableDefined(Box::new(value)))
    }
    fn parse_rule(&mut self) -> Result<QualifiedRule, ParserError> {
        let start = self.peek_token()?.start;

        let prelude = self.parse_prelude()?;
        self.expect(Kind::LeftBrace)?;
        let block = self.parse_block()?;
        let end = self.expect(Kind::RightBrace)?.end;
        Ok(QualifiedRule {
            prelude,
            block: Box::new(block),
            span: Span::new(start, end),
        })
    }
    fn is_at_ampersand(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Ampersand);
        }
        false
    }
    fn is_at_left_brace(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::LeftBrace);
        }
        false
    }

    /// a, b, c
    fn parse_prelude(&mut self) -> Result<SelectorList, ParserError> {
        let mut prelude = Vec::new();
        //  {
        while !self.is_at_left_brace() {
            let component = self.parse_prelude_component()?;
            prelude.push(component);
            if self.is_at_left_brace() {
                break;
            } else {
                self.expect(Kind::Comma)?;
            }
        }
        Ok(prelude)
    }
    // a b c
    fn parse_prelude_component(&mut self) -> Result<SelectorComponentList, ParserError> {
        let mut prelude = Vec::new();
        // , or {
        while !self.is_at_semicolon() || !self.is_at_left_brace() {
            let component = self.parse_selector_component()?;
            prelude.push(component);
        }
        Ok(prelude)
    }

    fn is_at_hash(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Hash);
        }
        false
    }
    fn is_at_dot(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Dot);
        }
        false
    }
    fn is_at_colon(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Colon);
        }
        false
    }
    fn is_at_whitespace(&mut self) -> bool {
        if let Ok(token) = self.peek_with_whitespace() {
            return matches!(token.kind, Kind::Whitespace);
        }
        false
    }

    fn is_at_left_parent(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::LeftParen);
        }
        false
    }

    fn parse_element(&mut self) -> Result<Token, ParserError> {
        let token = self.peek_token()?;
        match token.kind {
            Kind::Ident | Kind::Number => {
                let token = self.next_token()?;
                return Ok(token);
            }
            _ => return Err(ParserError::UnexpectedToken(self.next_token()?)),
        }
    }

    fn is_at_right_parent(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::RightParen);
        }
        false
    }
    fn parse_pseudo_function_params(&mut self) -> Result<SelectorList, ParserError> {
        // a, b, c
        let mut prelude = Vec::new();
        //  )
        while !self.is_at_right_parent() {
            let component = self.parse_prelude_component()?;
            prelude.push(component);
            if self.is_at_right_parent() {
                break;
            } else {
                self.expect(Kind::Comma)?;
            }
        }
        Ok(prelude)
    }
    // const re = /^[#.](?:[\w-]|\\(?:[A-Fa-f0-9]{1,6} ?|[^A-Fa-f0-9]))+/;
    fn parse_selector_component(&mut self) -> Result<Selector, ParserError> {
        if self.is_at_ampersand() {
            let token = self.next_token()?;
            return Ok(Selector::ParentSelector);
        } else if self.is_at_colon() {
            let start_token = self.expect(Kind::Colon)?;
            let end_token = self.parse_element()?;

            // :not(xxx xx)
            if self.is_at_left_parent() {
                self.expect(Kind::LeftParen)?;
                let params = self.parse_pseudo_function_params()?;
                let end_token = self.expect(Kind::RightParen)?;
                return Ok(Selector::PseudoSelector(PseudoSelector::PseudoFunction(
                    PseudoFunction {
                        name: self.get_atom_by_span(start_token.start, end_token.end),
                        span: Span::new(start_token.start, end_token.end),
                        params,
                    },
                )));
            }

            return Ok(Selector::PseudoSelector(PseudoSelector::PseudoElement(
                PseudoElement {
                    name: self.get_atom_by_span(start_token.start, end_token.end),
                    span: Span::new(start_token.start, end_token.end),
                },
            )));
        } else if self.is_at_hash() {
            let start_token = self.expect(Kind::Hash)?;
            let end_token = self.parse_element()?;
            return Ok(Selector::SimpleSelector(SimpleSelector {
                name: self.get_atom_by_span(start_token.start, end_token.end),
                span: Span::new(start_token.start, end_token.end),
            }));
        } else if self.is_at_ident() || self.is_at_number() {
            let start_token = self.next_token()?;
            let end_token = self.parse_element()?;
            return Ok(Selector::SimpleSelector(SimpleSelector {
                name: self.get_atom_by_span(start_token.start, end_token.end),
                span: Span::new(start_token.start, end_token.end),
            }));
        } else {
            self.parse_combinator()
        }
    }

    fn is_at_combinator(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(
                token.kind,
                Kind::GreaterThan
                    | Kind::Plus
                    | Kind::Tilde
                    | Kind::Pipe
                    | Kind::Caret
                    | Kind::Whitespace
            );
        }
        false
    }

    fn parse_combinator(&mut self) -> Result<Selector, ParserError> {
        if self.is_at_combinator() {
            let token = self.next_token()?;
            return Ok(Selector::SimpleSelector(SimpleSelector {
                name: self.get_atom(&token),
                span: token.into(),
            }));
        }
        Err(ParserError::UnexpectedToken(self.next_token()?))
    }
    fn is_at_right_curly_bracket(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::RightBrace);
        }
        false
    }
    fn try_parse_declaration(&mut self) -> Result<DeclarationList, ParserError> {
        let mut declaration_list = Vec::new();
        while !self.is_at_right_curly_bracket() {
            self.lexer.start();
            let name = self.expect(Kind::Ident)?;
            if self.is_at_colon() {
                self.expect(Kind::Colon)?;

                let value = self.parse_value_list()?;
                let is_at_right_curly_bracket = self.is_at_right_curly_bracket();
                let end;
                if is_at_right_curly_bracket {
                    // TODO: should get the last token span
                    end = 1;
                } else {
                    end = self.expect(Kind::Semicolon)?.end;
                }
                declaration_list.push(Declaration {
                    name: self.get_atom(&name),
                    span: Span::new(name.start, end),
                    value,
                });
                if is_at_right_curly_bracket {
                    break;
                }
            } else {
                self.lexer.restore();
                break;
            }
        }

        return Err(ParserError::UnexpectedToken(self.next_token()?));
    }

    fn parse_block(&mut self) -> Result<CurlyBracketsBlock, ParserError> {
        let mut content: Vec<CurlyBracketsBlockContent> = Vec::new();
        while let Ok(token) = self.peek_token() {
            match token.kind {
                Kind::AtKeyword => {
                    if self.is_at_defined_statement() {
                        let statement = self.parse_variable_defined()?;
                        content.push(CurlyBracketsBlockContent::DefinedStatement(statement));
                    } else {
                        let rule = self.parse_at_rule()?;
                        content.push(CurlyBracketsBlockContent::AtRule(rule));
                    }
                }
                Kind::Ident => {
                    self.lexer.start();
                    if let Ok(declaration) = self.try_parse_declaration() {
                        content.push(CurlyBracketsBlockContent::DeclarationList(declaration));
                        continue;
                    } else {
                        self.lexer.restore();
                    }
                    let rule = self.parse_rule()?;
                    content.push(CurlyBracketsBlockContent::QualifiedRule(rule));
                }
                Kind::RightBracket => break,
                _ => return Err(ParserError::UnexpectedToken(self.next_token()?)),
            }
        }
        Ok(CurlyBracketsBlock { content })
    }

    fn parse_ident(&mut self) -> Result<Ident, ParserError> {
        let ident = self.expect(Kind::Ident)?;
        Ok(Ident {
            name: self.get_atom(&ident),
            span: ident.into(),
        })
    }
    fn parse_at_keyword(&mut self) -> Result<AtKeyword, ParserError> {
        let ident = self.expect(Kind::AtKeyword)?;
        Ok(AtKeyword {
            name: self.get_atom(&ident),
            span: ident.into(),
        })
    }
    fn parse_string_literal(&mut self) -> Result<StringLiteral, ParserError> {
        let string = self.expect(Kind::String)?;
        Ok(StringLiteral {
            value: self.get_atom(&string),
            span: string.into(),
        })
    }

    fn is_at_unit(&mut self) -> bool {
        if self.is_at_ident() {
            return true;
        }
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Percent);
        }
        false
    }
    fn is_at_ident(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Ident);
        }
        false
    }

    fn parse_number_literal(&mut self) -> Result<NumberLiteral, ParserError> {
        let number = self.expect(Kind::Number)?;
        if self.is_at_unit() {
            let unit = if self.is_at_ident() {
                self.expect(Kind::Ident)?
            } else {
                self.expect(Kind::Percent)?
            };
            return Ok(NumberLiteral {
                value: self.get_float_number(&number)?,
                span: number.into(),
                unit: Some(self.get_atom(&unit)),
            });
        }
        Ok(NumberLiteral {
            value: self.get_float_number(&number)?,
            span: number.into(),
            unit: None,
        })
    }

    fn parse_map_defined(&mut self) -> Result<MapVariableDefined, ParserError> {
        todo!()
    }
    fn parse_value_defined(&mut self, name: Token) -> Result<VariableDefined, ParserError> {
        let value = self.parse_value_list()?;
        self.expect(Kind::Semicolon)?;
        Ok(VariableDefined {
            name: AtKeyword {
                name: self.get_atom(&name),
                span: name.into(),
            },
            value,
        })
    }

    fn is_at_comma(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Comma);
        }
        false
    }
    fn is_at_semicolon(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Semicolon);
        }
        false
    }
    fn is_at_space(&mut self) -> bool {
        if let Ok(token) = self.peek_with_whitespace() {
            return matches!(token.kind, Kind::Whitespace);
        }
        false
    }

    fn parse_value_list(&mut self) -> Result<VariableValueList, ParserError> {
        let mut values = Vec::new();
        while !self.is_at_semicolon() {
            let value = self.parse_value_components()?;
            values.push(value);
            if self.is_at_semicolon() {
                break;
            } else {
                self.expect(Kind::Comma)?;
            }
        }
        Ok(values)
    }

    fn parse_value_components(&mut self) -> Result<VariableValueComponentList, ParserError> {
        let mut components = Vec::new();
        while !self.is_at_comma() || !self.is_at_semicolon() {
            let component = self.value_defined_value()?;
            components.push(component);
            if self.is_at_comma() || self.is_at_semicolon() {
                break;
            } else {
                self.expect(Kind::Whitespace)?;
            }
        }
        Ok(components)
    }

    fn is_at_at_keyword(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::AtKeyword);
        }
        false
    }
    fn is_at_string(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::String);
        }
        false
    }
    fn is_at_number(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Number);
        }
        false
    }

    fn value_defined_value(&mut self) -> Result<VariableDefinedValue, ParserError> {
        if self.is_at_ident() {
            let ident = self.parse_ident()?;
            return Ok(VariableDefinedValue::PreservedToken(PreservedToken::Ident(
                ident,
            )));
        } else if self.is_at_at_keyword() {
            let keyword = self.parse_at_keyword()?;
            return Ok(VariableDefinedValue::PreservedToken(
                PreservedToken::AtKeyword(keyword),
            ));
        } else if self.is_at_string() {
            let string = self.parse_string_literal()?;
            return Ok(VariableDefinedValue::PreservedToken(
                PreservedToken::String(string),
            ));
        } else if self.is_at_number() {
            let number = self.parse_number_literal()?;
            return Ok(VariableDefinedValue::PreservedToken(
                PreservedToken::Number(number),
            ));
        } else {
            let token = self.next_token()?;
            return Ok(VariableDefinedValue::PreservedToken(PreservedToken::Token(
                LexerToken {
                    name: self.get_atom(&token),
                    span: token.into(),
                },
            )));
        }
    }
}

#[test]
fn quick_test() {
    let source = r#"
@color: 1 a,2; 
a {}
"#;
    let mut parser = Parser::new(source);
    let result = parser.parse();
    println!("{:#?}", result);
}
