mod ast;
mod interp;
mod lex;
mod parse;
mod sys;

use ast::expr::Expr;
use clap::{arg, command};

use lex::{LexResult, Lexer, TokenType};
use parse::Parser;

use crate::{ast::AstStringify, lex::val::ObjectVal};

fn main() -> anyhow::Result<()> {
    let args = command!()
        .arg(arg!(--file <FILEPATH>).required_unless_present("raw"))
        .arg(arg!(--raw <STRING>).required_unless_present("file"))
        .get_matches();

    let lex_result = if let Some(filepath) = args.get_one::<String>("file") {
        Lexer::scan_tokens_file(filepath)?
    } else if let Some(raw) = args.get_one::<String>("raw") {
        Lexer::scan_tokens(raw)
    } else {
        LexResult::empty()
    };

    let expr = Parser::parse(lex_result.tokens.as_ref())?;
    println!("{}", AstStringify.stringify(&expr)?);
    // print_expr();
    Ok(())
}

/// (* (- 123) (group 45.67))
fn print_expr() -> anyhow::Result<()> {
    let e = Box::new(Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: lex::Token {
                ty: TokenType::Minus,
                literal: ObjectVal::Nil,
                line: 1,
                lexeme: "-".into(),
            },
            right: Box::new(Expr::Literal(ObjectVal::Number(123.0))),
        }),
        operator: lex::Token {
            ty: TokenType::Star,
            literal: ObjectVal::Nil,
            line: 1,
            lexeme: "*".into(),
        },
        right: Box::new(Expr::Grouping(Box::new(Expr::Literal(ObjectVal::Number(
            45.67,
        ))))),
    });
    println!("{}", AstStringify.stringify(e.as_ref())?);
    Ok(())
}
