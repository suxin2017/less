use less_lexer::token::Token;
use serde::{Deserialize, Serialize};

pub type Atom = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stylesheets {
    pub span: Span,
    pub content: Vec<StyleContent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleContent {
    QualifiedRule(QualifiedRule),
    AtRule(AtRule),
    DefinedStatement(DefinedStatement),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualifiedRule {
    pub span: Span,
    pub prelude: SelectorList,
    pub block: Box<CurlyBracketsBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtRule {
    // todo span
    pub name: AtKeyword,
    pub prelude: VariableValueList,
    pub block: Option<CurlyBracketsBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParenthesesBlock(Vec<PreservedToken>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BracketsBlock(Vec<PreservedToken>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DefinedStatement {
    VariableDefined(Box<VariableDefined>),
    MapVariableDefined(MapVariableDefined),
    MixinDefined(MixinDefined),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixinDefined {
    pub name: SimpleSelector,
    pub params: Vec<Param>,
    pub block: CurlyBracketsBlock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: AtKeyword,
    pub default_params: Option<VariableValueList>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableDefined {
    pub name: AtKeyword,
    pub value: VariableValueList,
}

pub type VariableValueList = Vec<VariableDefinedValue>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableDefinedValue {
    // 1 + 2
    Express(Express),
    Ident(Ident),
    PreservedToken(PreservedToken),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapVariableDefined {
    pub name: AtKeyword,
    pub props: DeclarationList,
}

pub type DeclarationList = Vec<Declaration>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    pub name: Atom,
    pub value: DeclarationProps,
}

pub type DeclarationProps = VariableValueList;

pub type ComponentValueList = Vec<ComponentValue>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurlyBracketsBlock {
    pub content: Vec<CurlyBracketsBlockContent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CurlyBracketsBlockContent {
    QualifiedRule(QualifiedRule),
    AtRule(AtRule),
    DefinedStatement(DefinedStatement),
    DeclarationList(DeclarationList),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub enum ComponentValue {
    PreservedToken(PreservedToken),
    SelectorList(SelectorList),
    Express(Express),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixinCall {
    pub name: SelectorComponentList,
    pub params: VariableValueList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct FunctionDefinition {
    pub name: Ident,
    pub params: Vec<Ident>,
    pub guarded: Option<Express>,
    pub content: Vec<ComponentValue>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub enum Express {
    BinaryExpression(BinaryExpression),
    FunctionExpression(FunctionExpression),
    MixinCall(MixinCall),
    VariableExpression(VariableExpression),
    ParenthesesExpression(Box<Express>),
    StringEscape(StringLiteral),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub enum VariableExpression {
    Variable(AtKeyword),
    MapVariable(MapVariable),
    Color(Color),
    PreservedToken(PreservedToken),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapVariable {
    pub property: Ident,
    pub object: Box<Express>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct FunctionExpression {
    pub name: Ident,
    pub params: VariableValueList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub left: Box<Express>,
    pub operator: BinaryOperator,
    pub right: Box<Express>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Div,
    Mul,
}
pub type SelectorList = Vec<SelectorComponentList>;
pub type SelectorComponentList = Vec<Selector>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Selector {
    ParentSelector,
    SimpleSelector(SimpleSelector),
    PseudoSelector(PseudoSelector),
}

/**
 * #xx .xx
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleSelector {
    pub span: Span,
    pub name: Atom,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PseudoSelector {
    // :not
    PseudoFunction(PseudoFunction),
    // :hover
    PseudoElement(PseudoElement),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PseudoFunction {
    pub span: Span,
    pub name: Atom,
    pub params: SelectorList,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PseudoElement {
    pub span: Span,
    pub name: Atom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PreservedToken {
    Ident(Ident),
    AtKeyword(AtKeyword),
    String(StringLiteral),
    Number(NumberLiteral),
    Token(LexerToken),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LexerToken {
    pub span: Span,
    pub name: Atom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ident {
    pub span: Span,
    pub name: Atom,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtKeyword {
    pub span: Span,
    pub name: Atom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberLiteral {
    pub span: Span,
    pub value: f64,
    pub unit: Option<Atom>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringLiteral {
    pub span: Span,
    pub value: Atom,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[test]
fn quick_test() {
    // let mut content = Vec::new();

    // let variable_defined = VariableDefined {
    //     name: AtKeyword {
    //         span: Span::new(0, 1),
    //         name: "a".to_string(),
    //     },
    //     value: vec![VariableDefinedValue::PreservedToken(PreservedToken::Ident(
    //         Ident {
    //             span: Span::new(0, 1),
    //             name: "b".to_string(),
    //         },
    //     ))],
    // };
    // let map_defined = MapVariableDefined {
    //     name: AtKeyword {
    //         span: Span::new(0, 1),
    //         name: "a".to_string(),
    //     },
    //     props: vec![Declaration {
    //         name: "a".to_string(),
    //         value: vec![
    //             Express::VariableExpression(VariableExpression::Variable(Ident {
    //                 span: Span::new(0, 1),
    //                 name: "b".to_string(),
    //             })),
    //             Express::VariableExpression(VariableExpression::PreservedToken(
    //                 PreservedToken::Ident(Ident {
    //                     span: Span::new(0, 1),
    //                     name: "c".to_string(),
    //                 }),
    //             )),
    //         ],
    //     }],
    // };

    // let select_list = vec![Selector::SimpleSelector(SimpleSelector::IdSelector(
    //     IdSelector {
    //         span: Span::new(0, 1),
    //         value: "a".to_string(),
    //     },
    // ))];

    // let qualified_rule = QualifiedRule {
    //     prelude: select_list,
    //     span: Default::default(),
    //     block: Box::new(CurlyBracketsBlock {
    //         content: Box::new(CurlyBracketsBlockContent::DeclarationList(vec![
    //             Declaration {
    //                 name: "a".to_string(),
    //                 value: vec![
    //                     Express::VariableExpression(VariableExpression::Variable(Ident {
    //                         span: Span::new(0, 1),
    //                         name: "b".to_string(),
    //                     })),
    //                     Express::VariableExpression(VariableExpression::PreservedToken(
    //                         PreservedToken::Ident(Ident {
    //                             span: Span::new(0, 1),
    //                             name: "c".to_string(),
    //                         }),
    //                     )),
    //                 ],
    //             },
    //         ])),
    //     }),
    // };

    // let at_rule = AtRule {
    //     name: AtKeyword {
    //         span: Default::default(),
    //         name: "a".to_string(),
    //     },
    //     prelude: vec![Express::VariableExpression(
    //         VariableExpression::PreservedToken(PreservedToken::Ident(Ident {
    //             span: Default::default(),
    //             name: "b".to_string(),
    //         })),
    //     )],
    //     span: Default::default(),
    //     block: Some(CurlyBracketsBlock {
    //         content: Box::new(CurlyBracketsBlockContent::DeclarationList(vec![
    //             Declaration {
    //                 name: "a".to_string(),
    //                 value: vec![
    //                     Express::VariableExpression(VariableExpression::Variable(Ident {
    //                         span: Span::new(0, 1),
    //                         name: "b".to_string(),
    //                     })),
    //                     Express::VariableExpression(VariableExpression::PreservedToken(
    //                         PreservedToken::Ident(Ident {
    //                             span: Span::new(0, 1),
    //                             name: "c".to_string(),
    //                         }),
    //                     )),
    //                 ],
    //             },
    //         ])),
    //     }),
    // };

    // content.push(StyleContent::DefinedStatement(
    //     DefinedStatement::VariableDefined(Box::new(variable_defined)),
    // ));
    // content.push(StyleContent::QualifiedRule(qualified_rule));
    // content.push(StyleContent::AtRule(at_rule));

    // let stylesheets = Stylesheets {
    //     span: Default::default(),
    //     content,
    // };
    // dbg!(stylesheets);
}

impl From<Token> for Span {
    fn from(token: Token) -> Self {
        Span::new(token.start, token.end)
    }
}
