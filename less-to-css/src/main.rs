use std::{fmt::Write, path::Display};

use less_ast::{
    ast::{Atom, Stylesheets},
    visitor::Visitor,
};

struct ToCss<T: Write> {
    result: T,
}

impl<T: Write> ToCss<T> {
    fn fmt_ident(&mut self, ident: Atom) {
        self.result.write_str(&ident).unwrap();
    }
}

impl<T: Write> Visitor for ToCss<T> {
    fn visit_stylesheets(&mut self, stylesheets: &mut Stylesheets) {
        self.fmt_ident("stylesheets".to_string());
    }
}

fn main() {
    // let source = r#"
    // @color : 2 *;

    // "#;
    // let mut parser = Parser::new(source);
    // let result = parser.parse();
    let mut to_css = ToCss {
        result: String::new(),
    };
    to_css.fmt_ident("hello".to_string());
    println!("{:#?}", to_css.result);
}
