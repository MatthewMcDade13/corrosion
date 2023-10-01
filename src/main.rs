mod ast;

mod compiler;
mod env;
mod interp;
mod lex;
mod parse;
mod sys;
mod value;
mod vm;

use std::io::Write;

use ast::Expr;
use clap::{arg, command, Command};

use interp::Interpreter;
use lex::{LexResult, Lexer};
use log::debug;
use parse::Parser;
use vm::VM;

use crate::{
    ast::AstStringify,
    value::{Token, TokenType, Value},
};

fn main() -> anyhow::Result<()> {
    let args = Command::new("corrosion")
        .about("Corrosion Programming Language Interpreter and Compiler")
        .version("0.0.1")
        // .subcommand_required(true)
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
    let mut vm = VM::new();
    let mut buffer = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut buffer)?;
        vm.interpret_source(&buffer.trim())?;
        buffer.clear();
    }
}

/// (* (- 123) (group 45.67))
fn print_expr() -> anyhow::Result<()> {
    let e = Box::new(Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token {
                ty: TokenType::Minus,
                literal: Value::Nil,
                line: 1,
                lexeme: "-".into(),
            },
            right: Box::new(Expr::Literal(Value::Number(123.0))),
        }),
        operator: Token {
            ty: TokenType::Star,
            literal: Value::Nil,
            line: 1,
            lexeme: "*".into(),
        },
        right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Value::Number(
            45.67,
        ))))),
    });
    println!("{}", AstStringify.stringify(e.as_ref())?);
    Ok(())
}
