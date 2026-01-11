use std::collections::HashMap;

use automata::loader::{self, Context, Span, Spanned, lexer::Lexer};

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Kind {
    Ident = "ident",
    Keyword = "keyword",
    Error = "error",
    Comment = "comment",
    Punc = "punc",

    LPar = "lpar",
    LBrace = "lbrace",
    LBracket = "lbracket",

    RPar = "rpar",
    RBrace = "rbrace",
    RBracket = "rbracket",
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Tok {
    pub start: usize,
    pub end: usize,
    pub scope_level: usize,
    pub kind: Kind,
}

#[wasm_bindgen]
pub fn lex(input: &str) -> Vec<Tok> {
    let mut scope_level = 0;
    let mut index_utf16 = 0;
    let mut index_utf8 = 0;
    Lexer::new(input)
        .map(|Spanned(tok, Span(start_utf8, end_utf8))| {
            let since_last = &input[index_utf8..start_utf8];
            let since_start = &input[start_utf8..end_utf8];

            index_utf8 = end_utf8;
            let start = index_utf16 + since_last.chars().map(char::len_utf16).sum::<usize>();
            let end = start + since_start.chars().map(char::len_utf16).sum::<usize>();
            index_utf16 = end;

            let Ok(tok) = tok else {
                return Tok {
                    start,
                    end,
                    kind: Kind::Error,
                    scope_level,
                };
            };
            use automata::loader::lexer::Token;
            let kind = match tok {
                Token::LPar => Kind::LPar,
                Token::RPar => Kind::RPar,
                Token::LBrace => Kind::LBrace,
                Token::RBrace => Kind::RBrace,
                Token::LBracket => Kind::LBracket,
                Token::RBracket => Kind::RBracket,
                Token::Tilde => Kind::Keyword,
                Token::Eq => Kind::Punc,
                Token::Comma => Kind::Punc,
                Token::Or => Kind::Punc,
                Token::Plus => Kind::Punc,
                Token::Star => Kind::Punc,
                Token::And => Kind::Punc,
                Token::LSmallArrow => Kind::Punc,
                Token::LBigArrow => Kind::Punc,
                Token::Comment(_) => Kind::Comment,
                Token::Ident(_)
                    if input[..start_utf8]
                        .split("\n")
                        .last()
                        .unwrap_or_default()
                        .trim()
                        .is_empty() =>
                {
                    Kind::Keyword
                }
                Token::Ident(
                    loader::EPSILON_LOWER
                    | "epsilon"
                    | loader::DELTA_LOWER
                    | "delta"
                    | loader::GAMMA_UPPER
                    | "gamma"
                    | loader::GAMMA_LOWER
                    | loader::SIGMA_UPPER
                    | "sigma",
                ) => Kind::Keyword,
                Token::Ident(_) => Kind::Ident,
                Token::LineEnd => Kind::Punc,
            };

            let scope_level = match kind {
                Kind::LPar | Kind::LBrace | Kind::LBracket => {
                    scope_level = scope_level.saturating_add(1);
                    scope_level.saturating_sub(1)
                }
                Kind::RPar | Kind::RBrace | Kind::RBracket => {
                    scope_level = scope_level.saturating_sub(1);
                    scope_level
                }
                _ => scope_level,
            };
            Tok {
                start,
                end,
                kind,
                scope_level,
            }
        })
        .collect()
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum LogLevel {
    Info = "info",
    Warning = "warning",
    Error = "error",
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct CompileLog {
    pub level: LogLevel,
    pub message: String,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

#[derive(Serialize, Debug)]
pub struct Graph<'a> {
    initial: &'a str,
    final_states: Vec<&'a str>,
    states: Vec<&'a str>,
    transitions: HashMap<String, String>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct CompileResult {
    pub log: Vec<CompileLog>,
    pub ansi_log: String,
    pub machine: Option<String>,
}

#[wasm_bindgen]
pub fn compile(input: &str) -> CompileResult {
    let mut ctx = Context::new(input);
    let result = automata::loader::parse_universal(&mut ctx);

    let machine = result.map(|result| serde_json::to_string(&result).unwrap());

    use std::fmt::Write;
    let ansi_log = ctx.logs_display().fold(String::new(), |mut s, e| {
        write!(&mut s, "{e}").unwrap();
        s
    });

    let log = ctx
        .into_logs()
        .into_entries()
        .map(|e| CompileLog {
            level: match e.level {
                loader::log::LogLevel::Info => LogLevel::Info,
                loader::log::LogLevel::Warning => LogLevel::Warning,
                loader::log::LogLevel::Error => LogLevel::Error,
            },
            message: e.message,
            start: e
                .span
                .map(|span| input[..span.0].chars().map(char::len_utf16).count()),
            end: e
                .span
                .map(|span| input[..span.1].chars().map(char::len_utf16).count()),
        })
        .collect();

    CompileResult {
        log,
        ansi_log,
        machine,
    }
}
