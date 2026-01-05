use crate::loader::log::{LogEntryDisplay, Logs};
use crate::loader::{EPSILON_LOWER, Span, Spanned};

use super::ast::*;
use super::lexer::{Lexer, Token};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek: Spanned<Option<Token<'a>>>,
    logs: Logs<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser {
            logs: Logs::new(lexer.input()),
            peek: Spanned(None, Span(0,0)),
            lexer,
        }
    }

    fn eof(&self) -> Span {
        self.lexer.eof_span()
    }

    fn advance_line(&mut self) {
        if self.expect_token(Token::LineEnd).0 {
            self.peek = Spanned(None, Span(0,0));
        }
    }

    fn next_token(&mut self) -> Spanned<Option<Token<'a>>> {
        match self.peek.0 {
            Some(Token::LineEnd) => return self.peek,
            Some(_) => return Spanned(self.peek.0.take(), self.peek.1),
            _ => {}
        }
        loop {
            match self.lexer.next() {
                Some(Spanned(Ok(Token::Comment(_)), _)) => {}
                Some(Spanned(Ok(Token::LineEnd), span)) => {
                    self.peek = Spanned(Some(Token::LineEnd), span);
                    return self.peek;
                }
                Some(Spanned(Ok(ok), r)) => return Spanned(Some(ok), r),
                Some(Spanned(Err(err), span)) => self.logs.emit_error(format!("lexer: {err:?}"), span),
                None => return Spanned(None, self.lexer.eof_span())
            }
        }
    }

    fn peek_token(&mut self) -> Spanned<Option<Token<'a>>> {
        if self.peek.0.is_none() {
            self.peek = self.next_token();
        }
        self.peek
    }

    fn expect_token(&mut self, expected: Token<'a>) -> (bool, Span) {
        if let Some(Spanned(token, span)) = self.peek_token() {
            if token != expected {
                self.logs.emit_error(
                    format!("unexpected {:#}, expected {:}", token, expected),
                    span,
                );
                (false, span)
            } else {
                self.next_token();
                (true, span)
            }
        } else {
            self.logs.emit_error(
                format!("unexpected eof expected {:#}", expected),
                self.eof(),
            );
            (false, self.eof())
        }
    }

    fn parse_symbol(&mut self) -> Spanned<Symbol<'a>> {
        match self.next_token() {
            Spanned(Some(Token::Tilde), r) => Spanned(Symbol::Epsilon, r),
            Spanned(Some(Token::Ident("epsilon")), r) => Spanned(Symbol::Epsilon, r),
            Spanned(Some(Token::Ident(super::EPSILON_LOWER)), r) => Spanned(Symbol::Epsilon, r),
            Spanned(Some(Token::Ident(ident)), r) => Spanned(Symbol::Ident(ident), r),
            Spanned(Some(got), span) => {
                self.logs.emit_error(
                    format!(
                        "unexpected token {:#}, expected {:}|{:} (symbol)",
                        got,
                        Token::Tilde,
                        Token::Ident("")
                    ),
                    span,
                );
                Spanned(Symbol::Ident("<INVALID>"), span)
            }
            Spanned(None, span) => {
                self.logs.emit_error(
                    format!(
                        "unexpected eof expected {:}|{:} (symbol)",
                        Token::Tilde,
                        Token::Ident("")
                    ),
                    span,
                );
                Spanned(Symbol::Ident("<INVALID>"), self.eof())
            }
        }
    }

    fn parse_tupple(&mut self) -> Spanned<Tuple<'a>> {
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
            match self.peek_token() {
                None => {
                    self.logs.emit_error(
                        format!("unexpected eof expected {:}", Token::RPar),
                        self.eof(),
                    );
                    return Spanned(Tuple(items), start.join(self.eof()));
                }
                Some(Spanned(Token::LineEnd, span)) => {
                    self.logs
                        .emit_error(format!("unexpected eol expected {:}", Token::RPar), span);
                    return Spanned(Tuple(items), start.join(span));
                }
                _ => {}
            }
        }

        let (_, end) = self.expect_token(Token::RPar);

        Spanned(Tuple(items), start.join(end))
    }

    fn parse_item(&mut self) -> Spanned<Item<'a>> {
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
                        "unexpected token {:#}, expected {:}|{:}|{:}|{:}|{:} (item)",
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
                        "unexpected eof expected {:}|{:}|{:}|{:}|{:} (item)",
                        Token::Tilde,
                        Token::Ident(""),
                        Token::LPar,
                        Token::LBrace,
                        Token::LBracket,
                    ),
                    self.eof(),
                );
                Spanned(Item::Symbol(Symbol::Ident("<INVALID>")), self.eof())
            }
        }
    }

    fn parse_list(&mut self) -> Spanned<List<'a>> {
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
                return Spanned(List(Vec::new(), ListKind::BracketComma), span);
            }
            None => {
                self.logs.emit_error(
                    format!(
                        "unexpected eof expected {:}|{:}",
                        Token::RBrace,
                        Token::RBracket
                    ),
                    self.eof(),
                );
                return Spanned(List(Vec::new(), ListKind::BracketComma), self.eof());
            }
        };

        let mut comma = false;
        while self.peek_token().map(|t| t.0) != Some(match_end) {
            list.push(self.parse_item());
            if matches!(self.peek_token(), Some(Spanned(Token::Comma, _))) {
                comma = true;
                self.next_token();
            }
            match self.peek_token() {
                None => {
                    self.logs.emit_error(
                        format!("unexpected eof expected {:}", match_end),
                        self.eof(),
                    );
                    return Spanned(List(list, ListKind::BraceComma), start.join(self.eof()));
                }
                Some(Spanned(Token::LineEnd, span)) => {
                    self.logs
                        .emit_error(format!("unexpected eol expected {:}", match_end), span);
                    return Spanned(List(list, ListKind::BraceComma), start.join(span));
                }
                _ => {}
            }
        }
        let (_, end) = self.expect_token(match_end);
        let kind = match (comma, match_end) {
            (true, Token::RBrace) => ListKind::BraceComma,
            (false, Token::RBrace) => ListKind::Brace,
            (true, Token::RBracket) => ListKind::BracketComma,
            (false, Token::RBracket) => ListKind::Bracket,
            _ => unreachable!(),
        };
        Spanned(List(list, kind), start.join(end))
    }

    fn parse_regex(&mut self) -> Spanned<Regex<'a>> {
        todo!()
    }

    fn parse_production_rule(
        &mut self,
        sym: Symbol<'a>,
        start: Span,
    ) -> Option<Spanned<TopLevel<'a>>> {
        let mut lhs_group = ProductionGroup(vec![Spanned(sym, start)]);
        let mut lhs_group_end = start;
        while !matches!(
            self.peek_token(),
            None | Some(Spanned(Token::LSmallArrow | Token::LineEnd, _))
        ) {
            let sym = self.parse_symbol();
            lhs_group_end = sym.1;
            lhs_group.0.push(sym);
        }
        if !self.expect_token(Token::LSmallArrow).0{
            return Some(Spanned(TopLevel::ProductionRule(Spanned(lhs_group, start.join(lhs_group_end)), Spanned(vec![], lhs_group_end)), start.join(lhs_group_end)))
        }

        let mut groups = Vec::new();
        
        while !matches!(self.peek_token(), None | Some(Spanned(Token::LineEnd, _))){
            let mut group = ProductionGroup(vec![]);
            while !matches!(self.peek_token(), None | Some(Spanned(Token::LineEnd|Token::Or, _))){
                group.0.push(self.parse_symbol());
            }  
            if group.0.is_empty(){
                let span = if let Some(Spanned(_, span)) = self.peek_token(){
                    span
                }else{
                    self.eof()
                };
                self.logs.emit_error("cannot have empty production rule", span);
            }
            if matches!(self.peek_token(), Some(Spanned(Token::Or, _))){
                self.next_token();
                // if matches!(self.peek_token(), None|Spanned(Token::Or|Token::LineEnd))
            } 
            let group_start = group.0.first().map(|g|g.1).unwrap_or(start);
            let group_end = group.0.last().map(|g|g.1).unwrap_or(start);
            groups.push(Spanned(group, group_start.join(group_end)))
        }

         if groups.is_empty(){
            self.logs.emit_error("cannot have empty production rule", start.join(lhs_group_end));
        }

        let rules_start = groups.first().map(|f|f.1).unwrap_or(start);
        let rules_end = groups.last().map(|f|f.1).unwrap_or(start);

        Some(Spanned(TopLevel::ProductionRule(Spanned(lhs_group, start.join(lhs_group_end)), Spanned(groups, rules_start.join(rules_end))), start.join(rules_end)))
    }

    fn parse_transition_function(
        &mut self,
        ident: &'a str,
        start: Span,
    ) -> Option<Spanned<TopLevel<'a>>> {
        let tuple = self.parse_tupple();
        let span = start.join(tuple.1);
        let dest = Spanned((Spanned(ident, start), tuple), span);
        if !self.expect_token(Token::Eq).0 {
            return None;
        }
        let item = self.parse_item();
        let span = start.join(item.1);
        Some(Spanned(TopLevel::TransitionFunc(dest, item), span))
    }

    pub fn next_element(&mut self) -> Option<Spanned<TopLevel<'a>>> {
        let result = loop {
            let next = self.next_token()?;
            match (next, self.peek_token()) {
                (Spanned(Token::LineEnd, _), _) => self.advance_line(),
                (Spanned(Token::Ident(ident), start), Some(Spanned(Token::LPar, _))) => {
                    if let Some(tf) = self.parse_transition_function(ident, start) {
                        break Some(tf);
                    }
                }
                (
                    Spanned(
                        Token::Ident(EPSILON_LOWER) | Token::Ident("epsilon") | Token::Tilde,
                        start,
                    ),
                    Some(Spanned(Token::LSmallArrow | Token::Ident(_) | Token::Tilde, _)),
                ) => {
                    if let Some(pr) = self.parse_production_rule(Symbol::Epsilon, start) {
                        break Some(pr);
                    }
                }
                (
                    Spanned(Token::Ident(ident), start),
                    Some(Spanned(Token::LSmallArrow | Token::Ident(_) | Token::Tilde, _)),
                ) => {
                    if let Some(pr) = self.parse_production_rule(Symbol::Ident(ident), start) {
                        break Some(pr);
                    }
                }
                (Spanned(Token::Ident(ident), start), _) => {
                    let name = Spanned(ident, start);
                    if !self.expect_token(Token::Eq).0 {
                        continue;
                    }
                    let item = self.parse_item();
                    let span = start.join(item.1);
                    break Some(Spanned(TopLevel::Item(name, item), span));
                }
                _ => {
                    self.logs.emit_error(
                        format!(
                            "unexpected token {:#}, expected {:}",
                            next.0,
                            Token::Ident("")
                        ),
                        next.1,
                    );
                    while !matches!(self.next_token(), None|Some(Spanned(Token::LineEnd, _))){

                    }
            },
            }
        };
        self.advance_line();
        result
    }

    pub fn parse_elements(mut self) -> (Vec<Spanned<TopLevel<'a>>>, Logs<'a>) {
        let mut result = Vec::new();

        while let Some(next) = self.next_element() {
            result.push(next)
        }

        (result, self.logs)
    }

    pub fn logs(&self) -> impl Iterator<Item = LogEntryDisplay<'_>> {
        self.logs.displayable()
    }
}
