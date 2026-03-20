pub mod ast;
pub mod compiler;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod runtime;
pub mod span;
pub mod standard_lib;
pub mod type_system;

pub use error::{CorvoError, CorvoResult};
pub use runtime::RuntimeState;
pub use span::{Position, Span};

use crate::compiler::Evaluator;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::path::Path;

pub fn run_file(path: &Path) -> CorvoResult<()> {
    let source = std::fs::read_to_string(path).map_err(|e| CorvoError::io(e.to_string()))?;
    run_source(&source)
}

pub fn run_source(source: &str) -> CorvoResult<()> {
    let mut state = RuntimeState::new();
    run_source_with_state(source, &mut state)
}

pub fn run_source_with_state(source: &str, state: &mut RuntimeState) -> CorvoResult<()> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let mut evaluator = Evaluator::new();
    evaluator.run(&program, state)?;

    Ok(())
}

pub fn run_repl() {
    repl::run();
}
