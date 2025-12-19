use std::ops::Range;

use crate::lexer::Spanned;

#[derive(Clone, Debug)]
pub struct Tuple<'a>(pub Vec<Spanned<Symbol<'a>>>);

#[derive(Clone, Debug)]
pub enum Symbol<'a> {
    Epsilon,
    Ident(&'a str),
}

#[derive(Clone, Debug)]
pub enum Dest<'a> {
    Ident(&'a str),
    Function(Spanned<&'a str>, Spanned<Tuple<'a>>),
}

#[derive(Clone, Debug)]
pub enum Item<'a> {
    Symbol(Symbol<'a>),
    Tuple(Tuple<'a>),
    List(List<'a>),
}

#[derive(Clone, Debug)]
pub enum Regex<'a>{
    Terminal(&'a str),
    Match{
        complement: bool,
        patterns: Vec<Range<char>>
    },
    Concat(Vec<Regex<'a>>),
    Star(Box<Regex<'a>>),
    Plus(Box<Regex<'a>>),
    Union(Vec<Regex<'a>>),
    Intersection(Vec<Regex<'a>>),
    Complement(Box<Regex<'a>>),
}

#[derive(Clone, Debug)]
pub struct List<'a>(pub Vec<Spanned<Item<'a>>>);

#[derive(Clone, Debug)]
pub enum TopLevel<'a> {
    Assignment(Spanned<Dest<'a>>, Spanned<Item<'a>>),
    ProductionRule(Spanned<Symbol<'a>>, Spanned<Symbol<'a>>),
    Table(),
}