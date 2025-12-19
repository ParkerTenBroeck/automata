use std::iter::Peekable;

use crate::lexer::{Lexer, Span, Spanned, Token};

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
    List(List<'a>)
}

#[derive(Clone, Debug)]
pub struct List<'a>(pub Vec<Spanned<Item<'a>>>);

#[derive(Clone, Debug)]
pub enum TopLevel<'a> {
    Assignment(Spanned<Dest<'a>>, Spanned<Item<'a>>),
    Table(),
}

pub enum LogKind {
    Lexer,
    UnexpectedToken,
}

pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct Log {
    pub message: String,
    pub range: Span,
    pub level: LogLevel,
    pub kind: LogKind,
}

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    log: Vec<Log>,
    eof: Span,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self{
        Parser { eof: lexer.eof_span(), lexer: lexer.peekable(), log: Vec::new() }
    }

    fn next_token(&mut self) -> Option<Spanned<Token<'a>>> {
        loop {
            match self.lexer.next()? {
                Spanned(Ok(Token::Comment(_)), _) => {}
                Spanned(Ok(ok), r) => return Some(Spanned(ok, r)),
                Spanned(Err(err), r) => self.log.push(Log {
                    message: format!("{err:?}"),
                    range: r,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                }),
            }
        }
    }

    fn peek_token(&mut self) -> Option<Spanned<Token<'a>>> {
        loop {
            match *self.lexer.peek()? {
                // not a heavy clone but because of range
                Spanned(Ok(ok), r) => return Some(Spanned(ok, r)),
                Spanned(Err(err), r) => self.log.push(Log {
                    message: format!("{err:?}"),
                    range: r,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                }),
            }
        }
    }

        fn expect_token(&mut self, expected: Token<'a>) -> (bool, Span) {
        if let Some(Spanned(token, range)) = self.next_token() {
            if token != expected {
                self.log.push(Log {
                    message: format!("unexpected token {:#}, expected {:}", token, expected),
                    range,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                });
                (false, range)
            }else{
                (true, range)
            }
        } else {
            self.log.push(Log {
                message: format!("unexpected eof expected {:#}", expected),
                range: self.eof,
                level: LogLevel::Error,
                kind: LogKind::Lexer,
            });
            (false, self.eof)
        }
    }

    pub fn parse_symbol(&mut self) -> Spanned<Symbol<'a>> {
        match self.next_token() {
            Some(Spanned(Token::Tilde, r)) => Spanned(Symbol::Epsilon, r),
            Some(Spanned(Token::Ident("epsilon"), r)) => Spanned(Symbol::Epsilon, r),
            Some(Spanned(Token::Ident("ε"), r)) => Spanned(Symbol::Epsilon, r),
            Some(Spanned(Token::Ident(ident), r)) => Spanned(Symbol::Ident(ident), r),
            Some(Spanned(got, r)) => {
                self.log.push(Log {
                    message: format!(
                        "unexpected token {:#}, expected {:}|{:}",
                        got,
                        Token::Tilde,
                        Token::Ident("")
                    ),
                    range: self.eof,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                });
                Spanned(Symbol::Ident("<INVALID>"), r)
            }
            None => {
                self.log.push(Log {
                    message: format!(
                        "unexpected eof expected {:}|{:}",
                        Token::Tilde,
                        Token::Ident("")
                    ),
                    range: self.eof,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                });
                Spanned(Symbol::Ident("<INVALID>"), self.eof)
            }
        }
    }

    pub fn parse_tupple(&mut self) -> Spanned<Tuple<'a>> {
        let mut items = Vec::new();
        let (matched, start) = self.expect_token(Token::LPar);
        if !matched{
            return Spanned(Tuple(Vec::new()), start)
        }

        while !matches!(self.peek_token(), Some(Spanned(Token::RPar, _))) {
            items.push(self.parse_symbol());
            if matches!(self.peek_token(), Some(Spanned(Token::Comma, _))) {
                self.next_token();
            }
            if self.peek_token().is_none(){
                self.log.push(Log {
                    message: format!(
                        "unexpected eof expected {:}",
                        Token::RPar
                    ),
                    range: self.eof,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                });
                break;
            }
        }

        let (_, end) = self.expect_token(Token::RPar);

        Spanned(Tuple(items), start.join(end))
    }

    pub fn parse_item(&mut self) -> Spanned<Item<'a>>{
        match self.peek_token(){
            Some(Spanned(Token::Ident(_)|Token::Tilde, _)) => self.parse_symbol().map(Item::Symbol),
            Some(Spanned(Token::LPar, _)) => self.parse_tupple().map(Item::Tuple),
            Some(Spanned(Token::LBrace, _)) => self.parse_list().map(Item::List),
            Some(Spanned(got, r)) => {
                self.log.push(Log {
                    message: format!(
                        "unexpected token {:#}, expected {:}|{:}|{:}|{:}",
                        got,
                        Token::Tilde,
                        Token::Ident(""),
                        Token::LPar,
                        Token::LBrace
                    ),
                    range: self.eof,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                });
                Spanned(Item::Symbol(Symbol::Ident("<INVALID>")), r)
            }
            None => {
                self.log.push(Log {
                    message: format!(
                        "unexpected eof expected {:}|{:}|{:}|{:}",
                        Token::Tilde,
                        Token::Ident(""),
                        Token::LPar,
                        Token::LBrace
                    ),
                    range: self.eof,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                });
                Spanned(Item::Symbol(Symbol::Ident("<INVALID>")), self.eof)
            }
        }
    }

    pub fn parse_list(&mut self) -> Spanned<List<'a>>{
        let mut list = Vec::new();
        let (matched, start) = self.expect_token(Token::LBrace);
        if !matched{
            return Spanned(List(Vec::new()), start)
        }

        while !matches!(self.peek_token(), Some(Spanned(Token::RBrace, _))) {
            list.push(self.parse_item());
            if matches!(self.peek_token(), Some(Spanned(Token::Comma, _))) {
                self.next_token();
            }
            if self.peek_token().is_none(){
                self.log.push(Log {
                    message: format!(
                        "unexpected eof expected {:}",
                        Token::RBrace
                    ),
                    range: self.eof,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                });
                break;
            }
        }
        let (_, end) = self.expect_token(Token::RBrace);
        Spanned(List(list), start.join(end))
    }

    pub fn parse_elements(&mut self) -> Vec<Spanned<TopLevel<'a>>> {
        let mut result = Vec::new();

        loop {
            let Some(next) = self.next_token() else { break };
            match next {
                Spanned(Token::Ident(ident), ident_range) => {
                    let dest @ Spanned(_, start) = if matches!(self.peek_token(), Some(Spanned(Token::LPar, _))) {
                        let tuple = self.parse_tupple();
                        let span = ident_range.join(tuple.1);
                        Spanned(Dest::Function(Spanned(ident, ident_range), tuple), span)
                    } else {
                        Spanned(Dest::Ident(ident), ident_range)
                    };
                    self.expect_token(Token::Eq);

                    let item = self.parse_item();
                    let span = start.join(item.1);
                    result.push(Spanned(TopLevel::Assignment(dest, item), span));
                }
                _ => self.log.push(Log {
                    message: format!("unexpected token {:#}, expected {:}", next.0, Token::Ident("")),
                    range: next.1,
                    level: LogLevel::Error,
                    kind: LogKind::Lexer,
                }),
            }
        }
        result
    }
}
