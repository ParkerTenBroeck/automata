use std::ops::Range;

use super::Spanned;

#[derive(Clone, Debug)]
pub enum ListKind {
    Brace,
    Bracket,

    BraceComma,
    BracketComma,
}

#[derive(Clone, Debug)]
pub struct Tuple<'a>(pub Vec<Spanned<Item<'a>>>);

#[derive(Clone, Copy, Debug)]
pub enum Symbol<'a> {
    Epsilon(&'a str),
    Ident(&'a str),
}

#[derive(Clone, Debug)]
pub enum Item<'a> {
    Symbol(Symbol<'a>),
    Tuple(Tuple<'a>),
    List(List<'a>),
}

#[derive(Clone, Debug)]
pub enum Regex<'a> {
    Terminal(&'a str),
    Match {
        complement: bool,
        patterns: Vec<Range<char>>,
    },
    Concat(Vec<Regex<'a>>),
    Star(Box<Regex<'a>>),
    Plus(Box<Regex<'a>>),
    Union(Vec<Regex<'a>>),
    Intersection(Vec<Regex<'a>>),
    Complement(Box<Regex<'a>>),
}

#[derive(Clone, Debug)]
pub struct List<'a>(pub Vec<Spanned<Item<'a>>>, pub ListKind);

#[derive(Clone, Debug)]
pub struct ProductionGroup<'a>(pub Vec<Spanned<Symbol<'a>>>);

#[derive(Clone, Debug)]
pub enum TopLevel<'a> {
    Item(Spanned<&'a str>, Spanned<Item<'a>>),
    TransitionFunc(
        Spanned<(Spanned<&'a str>, Spanned<Tuple<'a>>)>,
        Spanned<Item<'a>>,
    ),
    ProductionRule(
        Spanned<ProductionGroup<'a>>,
        Spanned<Vec<Spanned<ProductionGroup<'a>>>>,
    ),
    Table(),
}

use crate::loader::{Context, log::LogSink};

impl<'a> Spanned<Item<'a>> {
    pub fn expect_symbol(&self, ctx: &mut Context<'a>) -> Option<Symbol<'a>> {
        match &self.0 {
            Item::Symbol(sym) => return Some(*sym),
            Item::Tuple(_) => _ = ctx.emit_error("expected ident found tuple", self.1),
            Item::List(_) => _ = ctx.emit_error("expected ident found list", self.1),
        }
        None
    }

    pub fn expect_ident(&self, ctx: &mut Context<'a>) -> Option<&'a str> {
        match &self.0 {
            Item::Symbol(Symbol::Ident(ident)) => return Some(ident),
            Item::Symbol(Symbol::Epsilon(_)) => _ = ctx.emit_error("expected ident found epsilon", self.1),
            Item::Tuple(_) => _ = ctx.emit_error("expected ident found tuple", self.1),
            Item::List(_) => _ = ctx.emit_error("expected ident found list", self.1),
        }
        None
    }

    pub fn expect_set(&self, ctx: &mut Context<'a>) -> Option<&[Spanned<Item<'a>>]> {
        match &self.0 {
            Item::Symbol(Symbol::Ident(_)) => _ = ctx.emit_error("expected set found ident", self.1),
            Item::Symbol(Symbol::Epsilon(_)) => _ = ctx.emit_error("expected set found epsilon", self.1),
            Item::Tuple(_) => _ = ctx.emit_error("expected set found tuple", self.1),
            Item::List(list) => return Some(&list.0),
        }
        None
    }

    pub fn expect_list(&self, ctx: &mut Context<'a>) -> Option<&[Spanned<Item<'a>>]> {
        match &self.0 {
            Item::Symbol(Symbol::Ident(_)) => _ = ctx.emit_error("expected list found ident", self.1),
            Item::Symbol(Symbol::Epsilon(_)) => _ = ctx.emit_error("expected list found epsilon", self.1),
            Item::Tuple(_) => _ = ctx.emit_error("expected list found tuple", self.1),
            Item::List(list) => return Some(&list.0),
        }
        None
    }

    pub fn list_weak(&self) -> &[Spanned<Item<'a>>] {
        match &self.0 {
            Item::List(list) => &list.0,
            _ => std::slice::from_ref(self),
        }
    }

    pub fn set_weak(&self) -> &[Spanned<Item<'a>>] {
        match &self.0 {
            Item::List(list) => &list.0,
            _ => std::slice::from_ref(self),
        }
    }

    pub fn expect_tuple(&self, ctx: &mut Context<'a>) -> Option<Spanned<&Tuple<'a>>> {
        match &self.0 {
            Item::Symbol(Symbol::Ident(_)) => _ = ctx.emit_error("expected tuple found ident", self.1),
            Item::Symbol(Symbol::Epsilon(_)) => _ = ctx.emit_error("expected tuple found epsilon", self.1),
            Item::Tuple(tuple) => return Some(Spanned(tuple, self.1)),
            Item::List(_) => _ = ctx.emit_error("expected tuple found list", self.1),
        }
        None
    }
}
