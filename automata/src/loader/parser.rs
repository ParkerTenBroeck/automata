use crate::epsilon;
use crate::loader::log::LogSink;
use crate::loader::{Context, Span};

use super::lexer::Token as T;
use crate::loader::Spanned as S;

use super::ast::*;
use super::lexer::Lexer;

pub struct Parser<'a, 'b> {
    lexer: Lexer<'a>,
    peek: Option<S<T<'a>>>,
    ctx: &'b mut Context<'a>,
}

impl<'a, 'b> Iterator for Parser<'a, 'b> {
    type Item = S<TopLevel<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_element()
    }
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(ctx: &'b mut Context<'a>) -> Self {
        Parser {
            lexer: Lexer::new(ctx.src()),
            ctx,
            peek: None,
        }
    }

    fn advance_line(&mut self) {
        if self.expect_token(T::LineEnd).0 {
            self.peek = None;
        }
    }

    fn next_token_optional(&mut self) -> Option<S<T<'a>>> {
        match self.peek {
            Some(S(T::LineEnd, _)) => return self.peek,
            Some(_) => return self.peek.take(),
            _ => {}
        }
        loop {
            match self.lexer.next() {
                Some(S(Ok(T::Comment(_)), _)) => {}
                Some(S(Ok(T::LineEnd), span)) => {
                    self.peek = Some(S(T::LineEnd, span));
                    return self.peek;
                }
                Some(S(Ok(ok), r)) => return Some(S(ok, r)),
                Some(S(Err(err), span)) => _ = self.ctx.emit_error(format!("lexer: {err:?}"), span),
                None => return None,
            }
        }
    }

    fn peek_token_optional(&mut self) -> Option<S<T<'a>>> {
        if self.peek.is_none() {
            self.peek = self.next_token_optional();
        }
        self.peek
    }

    fn next_token(&mut self) -> S<T<'a>> {
        self.next_token_optional()
            .unwrap_or(S(T::LineEnd, self.ctx.eof()))
    }

    fn peek_token(&mut self) -> S<T<'a>> {
        self.peek_token_optional()
            .unwrap_or(S(T::LineEnd, self.ctx.eof()))
    }

    fn expect_token(&mut self, expected: T<'a>) -> (bool, Span) {
        match self.peek_token() {
            S(token, span) if token == expected => {
                self.next_token();
                (true, span)
            }
            S(token, span) => {
                self.ctx.emit_error(
                    format!("unexpected {:#} expected {:}", token, expected),
                    span,
                );
                (false, span)
            }
        }
    }

    fn parse_as_symbol(&mut self, tok: S<T<'a>>) -> S<Symbol<'a>> {
        match tok {
            S(T::Tilde, r) => S(Symbol::Epsilon("~"), r),
            S(T::Ident(repr @ epsilon!(pat)), r) => S(Symbol::Epsilon(repr), r),
            S(T::Ident(ident), r) => S(Symbol::Ident(ident), r),
            S(got, span) => {
                self.ctx.emit_error(
                    format!(
                        "unexpected {:#} expected symbol ( {:} | {:} )",
                        got,
                        T::Tilde,
                        T::Ident("")
                    ),
                    span,
                );
                S(Symbol::Ident("<INVALID>"), span)
            }
        }
    }

    fn parse_symbol(&mut self) -> S<Symbol<'a>> {
        let next = self.next_token();
        self.parse_as_symbol(next)
    }

    fn parse_tupple(&mut self) -> S<Tuple<'a>> {
        let mut items = Vec::new();
        let (matched, start) = self.expect_token(T::LPar);
        if !matched {
            return S(Tuple(Vec::new()), start);
        }

        while !matches!(self.peek_token().0, T::RPar) {
            items.push(self.parse_item());
            if matches!(self.peek_token().0, T::Comma) {
                self.next_token();
            }
            if let S(T::LineEnd, span) = self.peek_token() {
                self.ctx
                    .emit_error(format!("unexpected eol expected {:}", T::RPar), span);
                return S(Tuple(items), start.join(span));
            }
        }

        let (_, end) = self.expect_token(T::RPar);

        S(Tuple(items), start.join(end))
    }

    fn parse_item(&mut self) -> S<Item<'a>> {
        match self.peek_token().0 {
            T::Ident(_) | T::Tilde => self.parse_symbol().map(Item::Symbol),
            T::LPar => self.parse_tupple().map(Item::Tuple),
            T::LBrace | T::LBracket => self.parse_list().map(Item::List),
            _ => {
                let S(got, span) = self.next_token();
                self.ctx.emit_error(
                    format!(
                        "unexpected {:#} expected item ( {:} | {:} | {:} | {:} | {:} )",
                        got,
                        T::Tilde,
                        T::Ident(""),
                        T::LPar,
                        T::LBrace,
                        T::LBracket,
                    ),
                    span,
                );
                S(Item::Symbol(Symbol::Ident("<INVALID>")), span)
            }
        }
    }

    fn parse_list(&mut self) -> S<List<'a>> {
        let mut list = Vec::new();

        let (start, match_end) = match self.next_token() {
            S(T::LBrace, span) => (span, T::RBrace),
            S(T::LBracket, span) => (span, T::RBracket),
            S(got, span) => {
                self.ctx.emit_error(
                    format!(
                        "unexpected {:#} expected list start ( {:} | {:} )",
                        got,
                        T::RBrace,
                        T::RBracket
                    ),
                    span,
                );
                return S(List(Vec::new(), ListKind::BracketComma), span);
            }
        };

        let mut comma = false;
        while self.peek_token().0 != match_end {
            list.push(self.parse_item());

            if list.len() != 1
                && self.peek_token().0 != match_end
                && !matches!(self.peek_token().0, T::LineEnd)
                && matches!(self.peek_token().0, T::Comma) != comma
            {
                let span = self.peek_token().1;
                self.ctx.emit_warning(
                    "inconsistent comma delimiting. use commas to delimit all or no items",
                    span,
                );
            }
            if matches!(self.peek_token().0, T::Comma) {
                comma = true;
                self.next_token();
            }
            if let S(T::LineEnd, span) = self.peek_token() {
                self.ctx.emit_error(
                    format!("unexpected eol expected list close ( {:} )", match_end),
                    span,
                );
                return S(List(list, ListKind::BraceComma), start.join(span));
            }
        }
        let (_, end) = self.expect_token(match_end);
        let kind = match (comma, match_end) {
            (true, T::RBrace) => ListKind::BraceComma,
            (false, T::RBrace) => ListKind::Brace,
            (true, T::RBracket) => ListKind::BracketComma,
            (false, T::RBracket) => ListKind::Bracket,
            _ => unreachable!(),
        };
        S(List(list, kind), start.join(end))
    }

    fn parse_regex(&mut self) -> S<Regex<'a>> {
        todo!()
    }

    fn parse_production_rule(&mut self, S(sym, start): S<Symbol<'a>>) -> Option<S<TopLevel<'a>>> {
        let mut lhs_group = ProductionGroup(vec![S(sym, start)]);
        let mut lhs_group_end = start;
        while !matches!(self.peek_token().0, T::LSmallArrow | T::LineEnd) {
            let sym = self.parse_symbol();
            lhs_group_end = sym.1;
            lhs_group.0.push(sym);
        }
        if !self.expect_token(T::LSmallArrow).0 {
            return Some(S(
                TopLevel::ProductionRule(
                    S(lhs_group, start.join(lhs_group_end)),
                    S(vec![], lhs_group_end),
                ),
                start.join(lhs_group_end),
            ));
        }

        let mut groups = Vec::new();

        loop {
            let mut group = ProductionGroup(vec![]);
            while !matches!(self.peek_token().0, T::LineEnd | T::Or) {
                group.0.push(self.parse_symbol());
            }

            if group.0.is_empty() {
                let span = self.peek_token().1;
                self.ctx
                    .emit_error("cannot have empty production group", span);
            }

            let group_start = group.0.first().map(|g| g.1).unwrap_or(start);
            let group_end = group.0.last().map(|g| g.1).unwrap_or(start);
            groups.push(S(group, group_start.join(group_end)));

            if matches!(self.peek_token().0, T::Or) {
                self.next_token();
            } else {
                break;
            }
        }

        if groups.is_empty() {
            self.ctx.emit_error(
                "cannot have empty production rule",
                start.join(lhs_group_end),
            );
        }

        let rules_start = groups.first().map(|f| f.1).unwrap_or(start);
        let rules_end = groups.last().map(|f| f.1).unwrap_or(start);

        Some(S(
            TopLevel::ProductionRule(
                S(lhs_group, start.join(lhs_group_end)),
                S(groups, rules_start.join(rules_end)),
            ),
            start.join(rules_end),
        ))
    }

    fn parse_transition_function(
        &mut self,
        ident: &'a str,
        start: Span,
    ) -> Option<S<TopLevel<'a>>> {
        let tuple = self.parse_tupple();
        let span = start.join(tuple.1);
        let dest = S((S(ident, start), tuple), span);
        if !self.expect_token(T::Eq).0 {
            return None;
        }
        let item = self.parse_item();
        let span = start.join(item.1);
        Some(S(TopLevel::TransitionFunc(dest, item), span))
    }

    pub fn next_element(&mut self) -> Option<S<TopLevel<'a>>> {
        let result = loop {
            let next = self.next_token_optional()?;
            match (next, self.peek_token()) {
                // empty
                (S(T::LineEnd, _), _) => self.advance_line(),
                // transition function
                (S(T::Ident(ident), start), S(T::LPar, _)) => {
                    if let Some(tf) = self.parse_transition_function(ident, start) {
                        break Some(tf);
                    }
                }
                // item
                (S(T::Ident(ident), start), S(T::Eq, _)) => {
                    let name = S(ident, start);
                    if !self.expect_token(T::Eq).0 {
                        continue;
                    }
                    let item = self.parse_item();
                    let span = start.join(item.1);
                    break Some(S(TopLevel::Item(name, item), span));
                }
                // production rule
                (
                    sym @ S(T::Ident(_) | T::Tilde, _),
                    S(T::LSmallArrow | T::Ident(_) | T::Tilde, _),
                ) => {
                    let sym = self.parse_as_symbol(sym);
                    if let Some(pr) = self.parse_production_rule(sym) {
                        break Some(pr);
                    }
                }

                (S(T::Ident(_), _), S(tok, span)) => {
                    self.ctx.emit_error(
                        format!(
                            "unexpected {:#} expected {:} | {:}",
                            tok,
                            T::Eq,
                            T::LSmallArrow
                        ),
                        span,
                    );
                    while !matches!(self.next_token().0, T::LineEnd) {}
                }
                _ => {
                    self.ctx.emit_error(
                        format!("unexpected {:#} expected {:}", next.0, T::Ident("")),
                        next.1,
                    );
                    while !matches!(self.next_token().0, T::LineEnd) {}
                }
            }
        };
        self.advance_line();
        result
    }
}
