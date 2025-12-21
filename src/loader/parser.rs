use crate::loader::log::{LogEntryDisplay, Logs};
use crate::loader::{Span, Spanned};

use super::ast::*;
use super::lexer::{Lexer, Token};
use std::iter::Peekable;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek: Option<Spanned<Token<'a>>>,
    logs: Logs<'a>,
    src: &'a str,
    eof: Span,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser {
            eof: lexer.eof_span(),
            src: lexer.input(),
            logs: Logs::new(lexer.input()),
            peek: None,
            lexer,
        }
    }

    fn next_token(&mut self) -> Option<Spanned<Token<'a>>> {
        if self.peek.is_some(){
            return self.peek.take()
        }
        loop {
            match self.lexer.next()? {
                Spanned(Ok(Token::Comment(_)), _) => {}
                Spanned(Ok(ok), r) => return Some(Spanned(ok, r)),
                Spanned(Err(err), span) => self.logs.emit_error(format!("lexer: {err:?}"), span),
            }
        }
    }

    fn peek_token(&mut self) -> Option<Spanned<Token<'a>>> {
        if self.peek.is_none(){
            self.peek = self.next_token();
        }
        self.peek
    }

    fn expect_token(&mut self, expected: Token<'a>) -> (bool, Span) {
        if let Some(Spanned(token, span)) = self.next_token() {
            if token != expected {
                self.logs.emit_error(
                    format!("unexpected token {:#}, expected {:}", token, expected),
                    span,
                );
                (false, span)
            } else {
                (true, span)
            }
        } else {
            self.logs
                .emit_error(format!("unexpected eof expected {:#}", expected), self.eof);
            (false, self.eof)
        }
    }

    pub fn parse_symbol(&mut self) -> Spanned<Symbol<'a>> {
        match self.next_token() {
            Some(Spanned(Token::Tilde, r)) => Spanned(Symbol::Epsilon, r),
            Some(Spanned(Token::Ident("epsilon"), r)) => Spanned(Symbol::Epsilon, r),
            Some(Spanned(Token::Ident(super::EPSILON_LOWER), r)) => Spanned(Symbol::Epsilon, r),
            Some(Spanned(Token::Ident(ident), r)) => Spanned(Symbol::Ident(ident), r),
            Some(Spanned(got, span)) => {
                self.logs.emit_error(
                    format!(
                        "unexpected token {:#}, expected {:}|{:}",
                        got,
                        Token::Tilde,
                        Token::Ident("")
                    ),
                    span,
                );
                Spanned(Symbol::Ident("<INVALID>"), span)
            }
            None => {
                self.logs.emit_error(
                    format!(
                        "unexpected eof expected {:}|{:}",
                        Token::Tilde,
                        Token::Ident("")
                    ),
                    self.eof,
                );
                Spanned(Symbol::Ident("<INVALID>"), self.eof)
            }
        }
    }

    pub fn parse_tupple(&mut self) -> Spanned<Tuple<'a>> {
        let mut items = Vec::new();
        let (matched, start) = self.expect_token(Token::LPar);
        if !matched {
            return Spanned(Tuple(Vec::new()), start);
        }

        while !matches!(self.peek_token(), Some(Spanned(Token::RPar, _))) {
            items.push(self.parse_item());
            if matches!(self.peek_token(), Some(Spanned(Token::Comma, _))) {
                self.next_token();
            }
            if self.peek_token().is_none() {
                self.logs.emit_error(
                    format!("unexpected eof expected {:}", Token::RPar),
                    self.eof,
                );
                break;
            }
        }

        let (_, end) = self.expect_token(Token::RPar);

        Spanned(Tuple(items), start.join(end))
    }

    pub fn parse_item(&mut self) -> Spanned<Item<'a>> {
        match self.peek_token() {
            Some(Spanned(Token::Ident(_) | Token::Tilde, _)) => {
                self.parse_symbol().map(Item::Symbol)
            }
            Some(Spanned(Token::LPar, _)) => self.parse_tupple().map(Item::Tuple),
            Some(Spanned(Token::LBrace | Token::LBracket, _)) => self.parse_list().map(Item::List),
            Some(Spanned(got, span)) => {
                self.next_token();
                self.logs.emit_error(
                    format!(
                        "unexpected token {:#}, expected {:}|{:}|{:}|{:}|{:}",
                        got,
                        Token::Tilde,
                        Token::Ident(""),
                        Token::LPar,
                        Token::LBrace,
                        Token::LBracket,
                    ),
                    span,
                );
                Spanned(Item::Symbol(Symbol::Ident("<INVALID>")), span)
            }
            None => {
                self.logs.emit_error(
                    format!(
                        "unexpected eof expected {:}|{:}|{:}|{:}|{:}",
                        Token::Tilde,
                        Token::Ident(""),
                        Token::LPar,
                        Token::LBrace,
                        Token::LBracket,
                    ),
                    self.eof,
                );
                Spanned(Item::Symbol(Symbol::Ident("<INVALID>")), self.eof)
            }
        }
    }

    pub fn parse_list(&mut self) -> Spanned<List<'a>> {
        let mut list = Vec::new();

        let (start, match_end) = match self.next_token() {
            Some(Spanned(Token::LBrace, r)) => (r, Token::RBrace),
            Some(Spanned(Token::LBracket, r)) => (r, Token::RBracket),
            Some(Spanned(got, span)) => {
                self.logs.emit_error(
                    format!(
                        "unexpected token {:#}, expected {:}|{:}",
                        got,
                        Token::RBrace,
                        Token::RBracket
                    ),
                    span,
                );
                return Spanned(List(Vec::new()), span);
            }
            None => {
                self.logs.emit_error(
                    format!(
                        "unexpected eof expected {:}|{:}",
                        Token::RBrace,
                        Token::RBracket
                    ),
                    self.eof,
                );
                return Spanned(List(Vec::new()), self.eof);
            }
        };

        while self.peek_token().map(|t| t.0) != Some(match_end) {
            list.push(self.parse_item());
            if matches!(self.peek_token(), Some(Spanned(Token::Comma, _))) {
                self.next_token();
            }
            if self.peek_token().is_none() {
                self.logs
                    .emit_error(format!("unexpected eof expected {:}", match_end), self.eof);
                break;
            }
        }
        let (_, end) = self.expect_token(match_end);
        Spanned(List(list), start.join(end))
    }

    pub fn parse_regex(&mut self) -> Spanned<Regex<'a>> {
        todo!()
    }

    pub fn parse_elements(mut self) -> (Vec<Spanned<TopLevel<'a>>>, Logs<'a>) {
        let mut result = Vec::new();

        loop {
            let Some(next) = self.next_token() else { break };
            match (next, self.peek_token()) {
                (Spanned(Token::Ident(ident), start), Some(Spanned(Token::LPar, _))) => {
                    let tuple = self.parse_tupple();
                    let span = start.join(tuple.1);
                    let dest = Spanned(Dest::Function(Spanned(ident, start), tuple), span);
                    self.expect_token(Token::Eq);
                    let item = self.parse_item();
                    let span = start.join(item.1);
                    result.push(Spanned(TopLevel::Assignment(dest, item), span));
                }
                (
                    Spanned(Token::Ident(_), _),
                    Some(Spanned(Token::LSmallArrow | Token::Ident(_), _)),
                ) => {
                    todo!()
                }
                (Spanned(Token::Ident(ident), start), _) => {
                    let dest = Spanned(Dest::Ident(ident), start);
                    self.expect_token(Token::Eq);
                    let item = self.parse_item();
                    let span = start.join(item.1);
                    result.push(Spanned(TopLevel::Assignment(dest, item), span));
                }
                _ => self.logs.emit_error(
                    format!(
                        "unexpected token {:#}, expected {:}",
                        next.0,
                        Token::Ident("")
                    ),
                    next.1,
                ),
            }
        }

        (result, self.logs)
    }

    pub fn logs(&self) -> impl Iterator<Item = LogEntryDisplay<'_>> {
        self.logs.displayable()
    }
}
