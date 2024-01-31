use less_ast::ast::{
    AtKeyword, AtRule, Atom, BinaryExpression, BinaryOperator, ComponentValue, ComponentValueList,
    CurlyBracketsBlock, CurlyBracketsBlockContent, Declaration, DeclarationList, DefinedStatement,
    Express, FunctionExpression, Ident, LexerToken, MapVariable, MapVariableDefined, MixinCall,
    MixinDefined, NumberLiteral, Param, PreservedToken, PseudoElement, PseudoFunction,
    PseudoSelector, QualifiedRule, Selector, SelectorComponentList, SelectorList, SimpleSelector,
    Span, StringLiteral, StyleContent, Stylesheets, VariableDefined, VariableDefinedValue,
    VariableExpression, VariableValueList,
};
use less_lexer::{
    token::{Kind, Token},
    Lexer,
};
use less_test_data::read_test_file;
use log::{debug, error, info, trace};
use simplelog::{Config, TermLogger};
use std::backtrace::Backtrace;
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
        let token = self.lexer.next()?;
        Ok(token)
    }
    pub fn skip_whitespace(&mut self) {
        while let Ok(token) = self.peek_token() {
            if token.kind == Kind::Whitespace {
                self.next_token();
            } else {
                break;
            }
        }
    }
    pub fn peek_token(&mut self) -> Result<&Token, ParserError> {
        Ok(self.lexer.peek()?)
    }
    pub fn peek_nth_token(&mut self, n: usize) -> Result<&Token, ParserError> {
        Ok(self.lexer.peek_nth(n)?)
    }

    pub fn expect(&mut self, kind: Kind) -> Result<Token, ParserError> {
        let token = self.next_token()?;
        if token.kind == kind {
            Ok(token)
        } else {
            let b = Backtrace::capture();
            println!("{}", b);
            trace!("expect: {:?},but found {:?}", kind, token);
            Err(ParserError::UnexpectedToken(token))
        }
    }

    pub fn expect_skit_whitespace(&mut self, kind: Kind) -> Result<Token, ParserError> {
        while let Ok(token) = self.peek_token() {
            if token.kind == Kind::Whitespace {
                self.next_token()?;
            } else {
                return self.next_token();
            }
        }
        self.next_token()
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
                        let statement = self.try_parse_variable_defined()?;
                        content.push(StyleContent::DefinedStatement(statement));
                    } else {
                        let rule = self.parse_at_rule()?;
                        content.push(StyleContent::AtRule(rule));
                    }
                }
                Kind::EOF => break,
                Kind::Whitespace => {
                    self.next_token()?;
                }
                _ => {
                    if self.is_at_selector_component() {
                        self.lexer.start();
                        if let Ok(mixin_defined) = self.try_parse_mixin_defined() {
                            content.push(StyleContent::DefinedStatement(
                                DefinedStatement::MixinDefined(mixin_defined),
                            ));
                            continue;
                        } else {
                            self.lexer.restore();
                        }
                        let rule = self.parse_rule()?;
                        content.push(StyleContent::QualifiedRule(rule));
                        continue;
                    }
                    trace!("unexpected token");
                    return Err(ParserError::UnexpectedToken(self.next_token()?));
                }
            }
        }
        Ok(Stylesheets {
            span: Span::new(0, self.source.len()),
            content,
        })
    }

    fn parse_at_rule(&mut self) -> Result<AtRule, ParserError> {
        let name = self.parse_at_keyword()?;
        let prelude = self.parse_value_list()?;
        let block = if self.is_at_semicolon() {
            None
        } else {
            Some(self.parse_block()?)
        };

        return Ok(AtRule {
            name,
            prelude,
            block,
        });
    }

    fn is_at_defined_statement(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            if matches!(token.kind, Kind::AtKeyword) {
                if let Ok(token) = self.peek_nth_token(1) {
                    match token.kind {
                        Kind::Colon => return true,
                        Kind::Whitespace => {
                            if let Ok(token) = self.peek_nth_token(2) {
                                return matches!(token.kind, Kind::Colon);
                            }
                        }
                        _ => {
                            return false;
                        }
                    }
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
    fn try_parse_variable_defined(&mut self) -> Result<DefinedStatement, ParserError> {
        let token = self.expect(Kind::AtKeyword)?;
        self.skip_whitespace();
        self.expect(Kind::Colon)?;
        self.skip_whitespace();
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
    fn is_at_mixin_name(&mut self) -> bool {
        self.is_at_dot() || self.is_at_hash()
    }

    fn parse_mixin_name(&mut self) -> Result<SimpleSelector, ParserError> {
        let start = self.next_token()?;
        let end = self.parse_ident()?;
        Ok(SimpleSelector {
            name: self.get_atom_by_span(start.start, end.span.end),
            span: Span::new(start.start, end.span.end),
        })
    }
    fn try_parse_mixin_defined(&mut self) -> Result<MixinDefined, ParserError> {
        let name = self.parse_mixin_name()?;
        self.skip_whitespace();
        self.expect(Kind::LeftParen)?;
        let params = self.parse_mixin_param_list()?;
        self.skip_whitespace();
        self.expect(Kind::RightParen)?;
        self.skip_whitespace();
        self.expect(Kind::LeftBrace)?;
        let block = self.parse_block()?;
        self.expect(Kind::RightBrace)?;
        Ok(MixinDefined {
            name,
            params,
            block,
        })
    }

    fn is_at_param(&mut self) -> bool {
        self.is_at_at_keyword()
    }
    fn parse_mixin_param_list(&mut self) -> Result<Vec<Param>, ParserError> {
        let mut params = Vec::new();
        while !self.is_at_right_parent() {
            let name = self.parse_at_keyword()?;
            let default_params = if self.is_at_colon() {
                self.expect(Kind::Colon)?;
                Some(self.parse_value_list()?)
            } else {
                None
            };

            params.push(Param {
                name,
                default_params,
            })
        }
        Ok(params)
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
    fn is_at_right_brace(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::RightBrace);
        }
        false
    }

    /// a, b, c
    fn parse_prelude(&mut self) -> Result<SelectorList, ParserError> {
        trace!("parse_prelude");
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
    // a b c , ' '
    fn parse_prelude_component(&mut self) -> Result<SelectorComponentList, ParserError> {
        let mut prelude = Vec::new();
        // , { }
        // mixin()
        while !self.is_at_semicolon()
            && !self.is_at_left_brace()
            && !self.is_at_right_brace()
            && !self.is_at_left_parent()
            && !self.is_at_right_parent()
            && !self.is_at_comma()
        {
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
        if let Ok(token) = self.peek_token() {
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

    fn is_at_element(&mut self) -> bool {
        return self.is_at_ident() || self.is_at_number();
    }

    fn parse_element(&mut self) -> Result<Token, ParserError> {
        let token = self.peek_token()?;
        match token.kind {
            Kind::Ident | Kind::Number => {
                let token = self.next_token()?;
                return Ok(token);
            }
            _ => {
                trace!("unexpected token: {:?}", token);
                return Err(ParserError::UnexpectedToken(self.next_token()?));
            }
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

    fn is_at_selector_component(&mut self) -> bool {
        return self.is_at_ampersand()
            || self.is_at_colon()
            || self.is_at_hash()
            || self.is_at_ident()
            || self.is_at_number()
            || self.is_at_dot();
    }

    // const re = /^[#.](?:[\w-]|\\(?:[A-Fa-f0-9]{1,6} ?|[^A-Fa-f0-9]))+/;
    fn parse_selector_component(&mut self) -> Result<Selector, ParserError> {
        trace!("parse_selector_component");
        if self.is_at_ampersand() {
            self.expect(Kind::Ampersand)?;
            return Ok(Selector::ParentSelector);
        } else if self.is_at_colon() {
            trace!("parse_selector_component");
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
            trace!("parse_element");
            let start_token = self.expect(Kind::Hash)?;
            let end_token = self.parse_element()?;
            return Ok(Selector::SimpleSelector(SimpleSelector {
                name: self.get_atom_by_span(start_token.start, end_token.end),
                span: Span::new(start_token.start, end_token.end),
            }));
        } else if self.is_at_dot() {
            trace!("parse_element");
            let start_token = self.expect(Kind::Dot)?;
            let end_token = self.parse_element()?;
            return Ok(Selector::SimpleSelector(SimpleSelector {
                name: self.get_atom_by_span(start_token.start, end_token.end),
                span: Span::new(start_token.start, end_token.end),
            }));
        } else if self.is_at_ident() || self.is_at_number() {
            trace!("parse_element");
            let start_token = self.next_token()?;
            let mut end_pos = start_token.end;
            if self.is_at_element() {
                end_pos = self.parse_element()?.end
            }
            return Ok(Selector::SimpleSelector(SimpleSelector {
                name: self.get_atom_by_span(start_token.start, end_pos),
                span: Span::new(start_token.start, end_pos),
            }));
        } else {
            trace!("unexpected token");
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
        trace!("unexpected token");
        Err(ParserError::UnexpectedToken(self.next_token()?))
    }
    fn is_at_right_curly_bracket(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::RightBrace);
        }
        false
    }
    fn is_at_left_curly_bracket(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::LeftBracket);
        }
        false
    }
    fn try_parse_declaration(&mut self) -> Result<DeclarationList, ParserError> {
        trace!("try_parse_declaration");
        let mut declaration_list = Vec::new();
        // { }
        while !self.is_at_right_curly_bracket() || !self.is_at_left_curly_bracket() {
            self.lexer.start();
            let name = self.expect_skit_whitespace(Kind::Ident)?;
            self.skip_whitespace();
            if self.is_at_colon() {
                self.expect_skit_whitespace(Kind::Colon)?;
                self.skip_whitespace();
                let value = self.parse_value_list()?;

                // the next token should have three result
                // ;} or } or ; Ident
                if self.is_at_semicolon() {
                    self.expect(Kind::Semicolon)?;
                }
                declaration_list.push(Declaration {
                    name: self.get_atom(&name),
                    value,
                });
                let is_at_right_curly_bracket = self.is_at_right_curly_bracket();
                if is_at_right_curly_bracket {
                    break;
                }
            } else {
                self.lexer.restore();
                break;
            }
        }
        return Ok(declaration_list);
    }

    fn parse_block(&mut self) -> Result<CurlyBracketsBlock, ParserError> {
        let mut content: Vec<CurlyBracketsBlockContent> = Vec::new();
        while let Ok(token) = self.peek_token() {
            trace!("parse_block: {:?}", token);
            match token.kind {
                Kind::AtKeyword => {
                    if self.is_at_defined_statement() {
                        let statement = self.try_parse_variable_defined()?;
                        content.push(CurlyBracketsBlockContent::DefinedStatement(statement));
                    } else {
                        let rule = self.parse_at_rule()?;
                        content.push(CurlyBracketsBlockContent::AtRule(rule));
                    }
                }
                Kind::Ident => {
                    let declaration = self.try_parse_declaration()?;
                    content.push(CurlyBracketsBlockContent::DeclarationList(declaration));
                    self.lexer.start();
                    if let Ok(mixin_defined) = self.try_parse_mixin_defined() {
                        content.push(CurlyBracketsBlockContent::DefinedStatement(
                            DefinedStatement::MixinDefined(mixin_defined),
                        ));
                        continue;
                    } else {
                        self.lexer.restore();
                    }
                    if self.is_at_selector_component() {
                        trace!("parse_block: Ident");
                        let rule = self.parse_rule()?;
                        content.push(CurlyBracketsBlockContent::QualifiedRule(rule));
                    }
                }

                Kind::RightBrace => break,
                Kind::Whitespace => {
                    self.next_token()?;
                }
                _ => {
                    if self.is_at_selector_component() {
                        self.lexer.start();
                        if let Ok(mixin_defined) = self.try_parse_mixin_defined() {
                            content.push(CurlyBracketsBlockContent::DefinedStatement(
                                DefinedStatement::MixinDefined(mixin_defined),
                            ));
                            continue;
                        } else {
                            self.lexer.restore();
                        }
                        self.lexer.start();
                        if let Ok(mixin_call) = self.try_parse_mixin_call() {
                            content.push(CurlyBracketsBlockContent::MixinCall(mixin_call));
                            continue;
                        } else {
                            self.lexer.restore();
                        }

                        let rule = self.parse_rule()?;
                        content.push(CurlyBracketsBlockContent::QualifiedRule(rule));
                        continue;
                    }
                    trace!("unexpected token");
                    return Err(ParserError::UnexpectedToken(self.next_token()?));
                }
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

    fn parse_value_list(&mut self) -> Result<VariableValueList, ParserError> {
        let mut values = Vec::new();
        while self.is_at_value_defined_value() {
            let value = self.parse_value_list_item()?;
            values.push(value);
            if self.is_at_semicolon() {
                break;
            }
        }
        Ok(values)
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
    fn is_at_tilde(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Tilde);
        }
        false
    }

    fn is_at_value_defined_value(&mut self) -> bool {
        return self.is_at_ident()
            || self.is_at_at_keyword()
            || self.is_at_string()
            || self.is_at_number()
            || self.is_at_whitespace()
            || self.is_at_comma()
            || self.is_at_plus()
            || self.is_at_minus()
            || self.is_at_asterisk()
            || self.is_at_slash()
            || self.is_at_left_parent()
            || self.is_at_tilde()
            || self.is_at_selector_component()
            || self.is_at_bang()
            || self.is_at_equal();
    }
    fn is_at_equal(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Equals);
        }
        false
    }
    fn is_at_plus(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Plus);
        }
        false
    }
    fn is_at_minus(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Minus);
        }
        false
    }
    fn is_at_asterisk(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Asterisk);
        }
        false
    }
    fn is_at_slash(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Slash);
        }
        false
    }

    fn is_at_bang(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::Bang);
        }
        false
    }

    // ident at-keyword string number
    fn parse_value_list_item(&mut self) -> Result<VariableDefinedValue, ParserError> {
        if !self.is_at_value_defined_value() {
            trace!("unexpected token");
            return Err(ParserError::UnexpectedToken(self.next_token()?));
        }
        if self.is_at_bang() {
            self.expect(Kind::Bang)?;
            let ident = self.parse_ident()?;
            return Ok(VariableDefinedValue::Important(ident));
        } else if self.is_at_ident() {
            // a()
            if let Ok(token) = self.peek_nth_token(1) {
                if matches!(token.kind, Kind::LeftParen) {
                    let express = self.try_parse_express()?;
                    return Ok(VariableDefinedValue::Express(express));
                }
            }

            let ident = self.parse_ident()?;
            return Ok(VariableDefinedValue::PreservedToken(PreservedToken::Ident(
                ident,
            )));
        } else if self.is_at_dot() || self.is_at_hash() {
            // may be mixin
            self.lexer.start();
            let express = self.try_parse_express();
            if let Ok(express) = express {
                return Ok(VariableDefinedValue::Express(express));
            } else {
                self.lexer.restore();
            }
        } else if self.is_at_at_keyword() {
            self.lexer.start();
            let express = self.try_parse_express();
            if let Ok(express) = express {
                return Ok(VariableDefinedValue::Express(express));
            } else {
                self.lexer.restore();
            }
        } else if self.is_at_string() {
            let string = self.parse_string_literal()?;
            return Ok(VariableDefinedValue::PreservedToken(
                PreservedToken::String(string),
            ));
        } else if self.is_at_number() {
            self.lexer.start();
            let express = self.try_parse_express();
            if let Ok(express) = express {
                return Ok(VariableDefinedValue::Express(express));
            } else {
                self.lexer.restore();
                let number = self.parse_number_literal()?;
                return Ok(VariableDefinedValue::PreservedToken(
                    PreservedToken::Number(number),
                ));
            }
        } else if self.is_at_left_parent() {
            self.lexer.start();
            let express = self.try_parse_express();
            if let Ok(express) = express {
                return Ok(VariableDefinedValue::Express(express));
            } else {
                self.lexer.restore();
            }
        }
        let token = self.next_token()?;
        return Ok(VariableDefinedValue::PreservedToken(PreservedToken::Token(
            LexerToken {
                name: self.get_atom(&token),
                span: token.into(),
            },
        )));
    }

    fn try_parse_express(&mut self) -> Result<Express, ParserError> {
        let mut cur = self.try_parse_term()?;
        loop {
            self.skip_whitespace();
            match self.peek_token()?.kind {
                Kind::Plus => {
                    self.expect(Kind::Plus)?;
                    self.skip_whitespace();
                    cur = Express::BinaryExpression(BinaryExpression {
                        left: Box::new(cur),
                        operator: BinaryOperator::Plus,
                        right: Box::new(self.try_parse_term()?),
                    });
                }
                Kind::Minus => {
                    self.expect(Kind::Minus)?;
                    self.skip_whitespace();
                    cur = Express::BinaryExpression(BinaryExpression {
                        left: Box::new(cur),
                        operator: BinaryOperator::Minus,
                        right: Box::new(self.try_parse_term()?),
                    });
                }
                _ => break,
            }
        }
        return Ok(cur);
    }
    fn try_parse_term(&mut self) -> Result<Express, ParserError> {
        let mut cur = self.try_parse_factory()?;
        loop {
            self.skip_whitespace();
            match self.peek_token()?.kind {
                Kind::Asterisk => {
                    self.expect(Kind::Asterisk)?;
                    self.skip_whitespace();
                    cur = Express::BinaryExpression(BinaryExpression {
                        left: Box::new(cur),
                        operator: BinaryOperator::Plus,
                        right: Box::new(self.try_parse_factory()?),
                    });
                }
                Kind::Slash => {
                    self.expect(Kind::Slash)?;
                    self.skip_whitespace();
                    cur = Express::BinaryExpression(BinaryExpression {
                        left: Box::new(cur),
                        operator: BinaryOperator::Minus,
                        right: Box::new(self.try_parse_factory()?),
                    });
                }

                _ => break,
            }
        }
        return Ok(cur);
    }

    fn is_at_left_bracket(&mut self) -> bool {
        if let Ok(token) = self.peek_token() {
            return matches!(token.kind, Kind::LeftBracket);
        }
        false
    }

    fn is_at_mixin_call(&mut self) -> bool {
        return self.is_at_selector_component();
    }
    fn try_parse_mixin_call(&mut self) -> Result<MixinCall, ParserError> {
        let name = self.parse_prelude_component()?;
        if self.is_at_left_parent() {
            self.expect(Kind::LeftParen)?;
            self.skip_whitespace();
            let express = self.parse_value_list()?;
            self.expect(Kind::RightParen)?;
            self.skip_whitespace();
            self.expect(Kind::Semicolon)?;
            return Ok(MixinCall {
                name,
                params: Some(express),
            });
        }
        self.expect(Kind::Semicolon)?;
        return Ok(MixinCall { name, params: None });
    }
    fn try_parse_factory(&mut self) -> Result<Express, ParserError> {
        let token = self.peek_token()?;
        match token.kind {
            Kind::Number => {
                let number = self.parse_number_literal()?;
                return Ok(Express::VariableExpression(
                    VariableExpression::PreservedToken(PreservedToken::Number(number)),
                ));
            }
            Kind::AtKeyword => {
                let keyword = self.parse_at_keyword()?;
                // @x[][][]
                if self.is_at_left_bracket() {
                    self.expect(Kind::LeftBracket)?;
                    self.skip_whitespace();
                    let object = Express::VariableExpression(VariableExpression::Variable(keyword));
                    let map_variable = MapVariable {
                        object: Box::new(object),
                        property: self.parse_ident()?,
                    };
                    self.expect(Kind::RightBracket)?;
                    return Ok(Express::VariableExpression(
                        VariableExpression::MapVariable(map_variable),
                    ));
                }
                return Ok(Express::VariableExpression(VariableExpression::Variable(
                    keyword,
                )));
            }
            Kind::Ident => {
                let name = self.parse_ident()?;
                self.expect(Kind::LeftParen)?;
                self.skip_whitespace();
                let express = self.parse_value_list()?;
                self.expect(Kind::RightParen)?;
                self.skip_whitespace();
                return Ok(Express::FunctionExpression(FunctionExpression {
                    name,
                    params: express,
                }));
            }
            Kind::Dot | Kind::Hash => {
                let express = self.try_parse_mixin_call()?;
                return Ok(Express::MixinCall(express));
            }
            // ~"string"
            Kind::Tilde => {
                self.expect(Kind::Tilde)?;
                self.skip_whitespace();
                let express = self.parse_string_literal()?;
                return Ok(Express::StringEscape(express));
            }
            Kind::LeftParen => {
                self.expect(Kind::LeftParen)?;
                self.skip_whitespace();

                let express = self.try_parse_express()?;
                self.skip_whitespace();
                self.expect(Kind::RightParen)?;
                return Ok(Express::ParenthesesExpression(Box::new(express)));
            }
            _ => {
                return Err(ParserError::UnexpectedToken(self.next_token()?));
            }
        }
    }
}

#[test]
fn quick_test() {
    TermLogger::init(
        log::LevelFilter::Trace,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    );
    let source = r#"
    .test-rulePollution {
        .polluteMixin();
    }
    
    
"#;
    let mut parser = Parser::new(source);
    let result = parser.parse();
    println!("{:#?}", result);
}

#[test]
fn quick_less_test() {
    let code = read_test_file("_main/variables.less");
    let mut parser = Parser::new(&code);
    let result = parser.parse();
    println!("{:#?}", result);
}
