mod eval;
mod lexer;
pub(crate) mod parser;

pub use eval::eval_script;
pub(crate) use eval::{call_script_function, construct_script_function};

pub fn parse_only(source: &str) -> crate::runtime::Completion<()> {
    parser::parse_script(source).map(|_| ())
}
