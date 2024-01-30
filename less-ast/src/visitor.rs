use core::prelude;
use std::mem;

use crate::ast::*;

pub trait Visitor {
    fn visit_stylesheets(&mut self, stylesheets: &mut Stylesheets) {
        for content in &mut stylesheets.content {
            self.visit_style_content(content);
        }
    }
    fn visit_style_content(&mut self, content: &mut StyleContent) {
        match content {
            StyleContent::AtRule(at_rule) => todo!(),
            StyleContent::QualifiedRule(qualified_rule) => {
                self.visit_qualified_rule(qualified_rule)
            }
            StyleContent::DefinedStatement(defined_statement) => {
                self.visit_defined_statement(defined_statement)
            }
        }
    }

    fn visit_qualified_rule(&mut self, qualified_rule: &mut QualifiedRule) {
        for prelude in &mut qualified_rule.prelude {
            self.visit_prelude(prelude);
        }
        let block = &mut *qualified_rule.block;
        self.visit_block(block);
    }

    fn visit_block(&mut self, block: &mut CurlyBracketsBlock) {
        for content in &mut block.content {
            self.visit_curly_brackets_block_content(content);
        }
    }

    fn visit_curly_brackets_block_content(&mut self, content: &mut CurlyBracketsBlockContent) {
        match content {
            CurlyBracketsBlockContent::QualifiedRule(qualified_rule) => {
                self.visit_qualified_rule(qualified_rule);
            }
            CurlyBracketsBlockContent::DeclarationList(declaration) => {
                self.visit_declaration_list(declaration);
            }
            CurlyBracketsBlockContent::AtRule(at_rule) => {
                self.visit_at_rule(at_rule);
            }
            CurlyBracketsBlockContent::DefinedStatement(defined_statement) => {
                self.visit_defined_statement(defined_statement);
            }
        }
    }
    fn visit_declaration_list(&mut self, declaration_list: &mut DeclarationList) {
        for declaration in declaration_list {
            self.visit_declaration(declaration);
        }
    }
    fn visit_declaration(&mut self, declaration: &mut Declaration) {
        for value in &mut declaration.value {
            self.visit_variable_defined_value(value);
        }
    }
    fn visit_variable_defined_value(&mut self, declaration_props: &mut VariableDefinedValue) {
        match declaration_props {
            VariableDefinedValue::Express(express) => {
                self.visit_expression(express);
            }
            _ => {}
        }
    }
    fn visit_expression(&mut self, express: &mut Express) {
        match express {
            Express::BinaryExpression(binary_expression) => {
                self.visit_binary_expression(binary_expression);
            }
            Express::FunctionExpression(function_expression) => {
                self.visit_function_expression(function_expression);
            }
            _ => {}
        }
    }
    fn visit_binary_expression(&mut self, binary_expression: &mut BinaryExpression) {
        self.visit_expression(&mut *binary_expression.left);
        self.visit_expression(&mut *binary_expression.right);
    }
    fn visit_function_expression(&mut self, function_expression: &mut FunctionExpression) {
        todo!();
    }

    fn visit_at_rule(&mut self, at_rule: &mut AtRule) {
        todo!();
    }

    fn visit_prelude(&mut self, prelude: &mut SelectorComponentList) {
        for component in prelude {
            self.visit_selector(component);
        }
    }

    fn visit_selector(&mut self, selector: &mut Selector) {
        match selector {
            Selector::ParentSelector => {}
            Selector::SimpleSelector(simple_selector) => {
                self.visit_simple_selector(simple_selector);
            }
            Selector::PseudoSelector(pseudo_selector) => {
                self.visit_pseudo_selector(pseudo_selector);
            }
        }
    }

    fn visit_simple_selector(&mut self, simple_selector: &mut SimpleSelector) {}

    fn visit_pseudo_selector(&mut self, pseudo_selector: &mut PseudoSelector) {
        todo!();
    }

    fn visit_defined_statement(&mut self, defined_statement: &mut DefinedStatement) {}
}

impl AstVisitor {
    fn get_simple_selector_name(&mut self, simple_selector: &mut SimpleSelector) -> String {
        mem::take(&mut simple_selector.name)
    }
}

struct AstVisitor;
impl Visitor for AstVisitor {

    fn visit_selector(&mut self, mut selector: &mut Selector) {
        if let Selector::SimpleSelector(simple_selector) = selector {
            let mut old_name = self.get_simple_selector_name(simple_selector);
            old_name.insert_str(0, "hello");
            simple_selector.name = "b".to_string();
            dbg!(old_name, &simple_selector.name);
        }
    }

  
}

#[test]
fn quick_test() {
    let mut visitor = AstVisitor;
    let mut content = Vec::new();

    let select_list = vec![vec![Selector::SimpleSelector(SimpleSelector {
        span: Span::new(0, 1),
        name: "a".to_string(),
    })]];

    let qualified_rule = QualifiedRule {
        prelude: select_list,
        span: Default::default(),
        block: Box::new(CurlyBracketsBlock { content: vec![] }),
    };

    content.push(StyleContent::QualifiedRule(qualified_rule));

    let mut stylesheets = Stylesheets {
        span: Default::default(),
        content,
    };
    visitor.visit_stylesheets(&mut stylesheets);

    dbg!(&stylesheets);
}
