pub mod dfa;
pub mod lexer;
pub mod parser;
pub mod ast;

pub struct SymbolMap<T>([T; 256]);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct State(u16);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Symbol(u16);
