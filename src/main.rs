mod ast;
mod env;
mod interp;
mod lex;
mod parse;
mod sys;

use ast::Expr;
use clap::{arg, command, Command};

use interp::Interpreter;
use lex::{LexResult, Lexer, TokenType};
use parse::Parser;

use crate::{ast::AstStringify, lex::val::ObjectVal};

fn main() -> anyhow::Result<()> {
    let args = Command::new("corrosion")
        .about("Corrosion Programming Language Interpreter and Compiler")
        .version("0.0.1")
        // .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(arg!([filepath] "path to script to run").required(false))
        .get_matches();

    if let Some(filepath) = args.get_one::<String>("filepath") {
        let result = Lexer::scan_tokens_file(filepath)?;

        let stmts = Parser::parse(result.tokens.as_ref())?;
        let mut interp = Interpreter::new();
        interp.execute_block(stmts.as_slice())?;
    } else {
        run_repl()?;
    };

    // println!("{}", AstStringify.stringify(&expr)?);
    // println!("{}", result_str);
    // print_expr();
    Ok(())
}

// TODO(FIXME) :: Fix repl
fn run_repl() -> anyhow::Result<()> {
    let mut interp = Interpreter::new();
    loop {
        let mut buffer = String::new();
        print!("> ");
        std::io::stdin().read_line(&mut buffer)?;
        let result = Lexer::scan_tokens(&buffer);

        let stmts = Parser::parse(result.tokens.as_ref())?;
        if let Some(s) = stmts.first() {
            interp.execute(s)?;
        }
    }
}

/// (* (- 123) (group 45.67))
fn print_expr() -> anyhow::Result<()> {
    let e = Box::new(Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: lex::Token {
                ty: TokenType::Minus,
                literal: ObjectVal::Unit,
                line: 1,
                lexeme: "-".into(),
            },
            right: Box::new(Expr::Literal(ObjectVal::Number(123.0))),
        }),
        operator: lex::Token {
            ty: TokenType::Star,
            literal: ObjectVal::Unit,
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
