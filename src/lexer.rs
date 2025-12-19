#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Token<'a> {
    LPar,
    RPar,

    LBrace,
    RBrace,

    LBracket,
    RBracket,

    Tilde,
    Eq,
    Comma,

    Or,
    Plus,
    Star,
    And,

    LSmallArrow,
    LBigArrow,

    Comment(&'a str),

    Ident(&'a str),
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LPar => write!(f, "')'"),
            Token::RPar => write!(f, "'('"),
            Token::LBrace => write!(f, "'{{'"),
            Token::RBrace => write!(f, "'}}'"),
            Token::LBracket => write!(f, "'['"),
            Token::RBracket => write!(f, "']'"),
            Token::Tilde => write!(f, "'~'"),
            Token::Eq => write!(f, "'='"),
            Token::Comma => write!(f, "','"),
            Token::Or => write!(f, "'|'"),
            Token::Plus => write!(f, "'+'"),
            Token::Star => write!(f, "'*'"),
            Token::And => write!(f, "'&'"),
            Token::LSmallArrow => write!(f, "'->'"),
            Token::LBigArrow => write!(f, "'=>'"),
            Token::Comment(_) => write!(f, "<comment>"),
            Token::Ident(ident) if f.alternate() => write!(f, "{ident:?}"),
            Token::Ident(_) => write!(f, "ident"),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Span(pub usize, pub usize);
impl Span {
    pub fn join(&self, end: Span) -> Span {
        Span(self.0, end.1)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Spanned<T>(pub T, pub Span);
impl<T> Spanned<T> {
    pub fn map<R>(self, map: impl Fn(T) -> R) -> Spanned<R> {
        Spanned(map(self.0), self.1)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Lexer<'a> {
    input: &'a str,

    start: usize,
    position: usize,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Error {
    InvalidChar(char),
    UnclosedMultiLine,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            start: 0,
            position: 0,
        }
    }

    fn consume(&mut self) -> Option<char> {
        let next = self.input.get(self.position..)?.chars().next()?;
        self.position += next.len_utf8();
        Some(next)
    }

    fn peek(&mut self) -> Option<char> {
        self.input.get(self.position..)?.chars().next()
    }

    fn backtrack(&mut self) {
        if let Some(consumed) = self.input.get(..self.position)
            && let Some(previous) = consumed.chars().next_back()
        {
            self.position -= previous.len_utf8();
        }
    }

    pub fn eof_span(&self) -> Span {
        Span(self.input.len(), self.input.len())
    }
}

impl<'a> std::iter::Iterator for Lexer<'a> {
    type Item = Spanned<Result<Token<'a>, Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.peek()
            && c.is_whitespace()
        {
            self.consume();
        }
        self.start = self.position;

        let res = match self.consume()? {
            '(' => Ok(Token::LPar),
            ')' => Ok(Token::RPar),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            '[' => Ok(Token::LBracket),
            ']' => Ok(Token::RBracket),
            '~' => Ok(Token::Tilde),
            '+' => Ok(Token::Plus),
            '*' => Ok(Token::Star),
            '&' => Ok(Token::And),
            ',' => Ok(Token::Comma),
            '|' => Ok(Token::Or),
            '=' => match self.peek() {
                Some('>') => {
                    self.consume();
                    Ok(Token::LBigArrow)
                }
                _ => Ok(Token::Eq),
            },
            '-' => match self.peek() {
                Some('>') => {
                    self.consume();
                    Ok(Token::LSmallArrow)
                }
                _ => Err(Error::InvalidChar('-')),
            },

            '/' => match self.consume() {
                Some('/') => loop {
                    if let Some('\n') | None = self.consume() {
                        break Ok(Token::Comment(&self.input[self.start + 2..self.position]));
                    }
                },
                Some('*') => loop {
                    match self.consume() {
                        Some('*') if self.peek() == Some('/') => {
                            self.consume();
                            break Ok(Token::Comment(
                                &self.input[self.start + 2..self.position - 2],
                            ));
                        }
                        Some(_) => {}
                        None => break Err(Error::UnclosedMultiLine),
                    }
                },
                Some(_) => {
                    self.backtrack();
                    Err(Error::InvalidChar('/'))
                }
                None => Err(Error::InvalidChar('/')),
            },

            c if c.is_alphabetic() || c == '_' => loop {
                match self.consume() {
                    Some(c) if c.is_alphanumeric() || c == '_' => {}
                    Some(_) => {
                        self.backtrack();
                        break Ok(Token::Ident(&self.input[self.start..self.position]));
                    }
                    None => break Ok(Token::Ident(&self.input[self.start..self.position])),
                }
            },

            c => Err(Error::InvalidChar(c)),
        };
        let span = Span(self.start, self.position);
        self.start = self.position;
        Some(Spanned(res, span))
    }
}

#[test]
fn tokenizer() {
    let tests = [
        "",
        "/*",
        "/**",
        "/*/",
        "/**/",
        "/",
        "//",
        "()[]{}~=>==>->-+*&|, hello _th012is__ a wondweful",
    ];

    for test in tests {
        println!("'{test}': {:?}", Lexer::new(test).collect::<Vec<_>>())
    }
}
