use crate::{
    automatan::*,
    dual_enum_serde,
    loader::{
        ast::TopLevel,
        log::{LogEntry, LogSink},
    },
};

pub mod ast;
pub mod lexer;
pub mod log;
pub mod parser;

#[macro_export]
macro_rules! maker {
    (pat: $($pat:pat),*) => {
      $($pat)|*
    };
    (arr: $($expr:expr),*) => {
        [$($expr),*]
    };
    (str: $first:literal, $($remainder:literal),+) => {
        concat!($crate::maker!(str: $first), " | ", $crate::maker!(str: $($remainder),*))
    };
    (str: $first:literal) => {
        concat!("'",$first,"'")
    };
}

pub const INITIAL_STATE: &str = "q0";
pub const INITIAL_STACK: &str = "z0";
pub const BLANK_SYMBOL: &str = "B";

#[macro_export]
macro_rules! epsilon {
    ($ident: ident) => {
      $crate::maker!($ident: "epsilon","~", "Ɛ", "ε", "ϵ", "𝛆", "𝛜", "𝜀", "𝜖", "𝜺", "𝝐", "𝝴", "𝞊", "𝞮", "𝟄", "ɛ")
    };
}

#[macro_export]
macro_rules! delta_lower {
    ($ident: ident) => {
      $crate::maker!($ident: "delta","D","d","ẟ","δ", "𝛅", "𝛿", "𝜹", "𝝳", "𝞭")
    };
}

#[macro_export]
macro_rules! sigma_upper {
    ($ident: ident) => {
      $crate::maker!($ident: "E","S", "sigma","Σ","𝚺", "𝛴", "𝜮", "𝝨", "𝞢", "∑")
    };
}

#[macro_export]
macro_rules! gamma_upper {
    ($ident: ident) => {
      $crate::maker!($ident: "T","G","gamma","Γ","Ⲅ", "𝚪", "𝛤", "𝜞", "𝝘", "𝞒")
    };
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned(&self.0, self.1)
    }
}

pub struct Context<'a> {
    logs: log::Logs,
    src: &'a str,
}

impl<'a> LogSink for Context<'a> {
    fn emit(&mut self, entry: log::LogEntry) -> &mut LogEntry {
        self.logs.emit(entry)
    }
}

impl<'a> Context<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            logs: log::Logs::new(),
            src,
        }
    }

    pub fn src(&self) -> &'a str {
        self.src
    }

    pub fn logs_display(&self) -> impl Iterator<Item = log::LogEntryDisplay<'_>> {
        self.logs.displayable_with(self.src)
    }

    pub fn eof(&self) -> Span {
        Span(self.src.len(), self.src.len())
    }

    pub fn contains_errors(&self) -> bool {
        self.logs.contains_errors()
    }

    pub fn into_logs(self) -> log::Logs {
        self.logs
    }
}

dual_enum_serde! {
    {#[serde(tag = "type")] #[serde(rename_all = "snake_case")]}
    #[derive(Clone, Debug)]
    pub enum Machine<'a> {
        Fa(#[serde(borrow)] fa::Fa<'a>),
        Pda(#[serde(borrow)] pda::Pda<'a>),
        Tm(#[serde(borrow)] tm::Tm<'a>),
    }
}

pub fn parse_universal<'a>(ctx: &mut Context<'a>) -> Option<Machine<'a>> {
    let mut items = parser::Parser::new(ctx).collect::<Vec<_>>().into_iter();
    if ctx.logs.contains_errors() {
        return None;
    }

    use Spanned as S;

    #[derive(Debug)]
    enum Type {
        Dfa,
        Nfa,
        Dpda,
        Npda,
        Tm,
        Ntm,
    }

    fn parse_type<'a>(item: Option<S<TopLevel<'a>>>, ctx: &mut Context<'a>) -> Option<Type> {
        let (str, span) = match item {
            Some(S(TopLevel::Item(S("type", _), item @ S(_, span)), _)) => {
                (item.expect_ident(ctx)?, span)
            }
            Some(S(_, span)) => {
                ctx.emit_error("expected type=<type> as first item", span)
                    .emit_help_logless("add: type = ...");
                return None;
            }
            None => {
                ctx.emit_error("expected type=<type> as first item", ctx.eof())
                    .emit_help_logless("add: type = ...");
                return None;
            }
        };

        Some(match str {
            "dfa" | "DFA" => Type::Dfa,
            "nfa" | "NFA" => Type::Nfa,
            "dpda" | "DPDA" => Type::Dpda,
            "npdaA" | "NPDA" => Type::Npda,
            "tm" | "TM" => Type::Tm,
            "ntm" | "NTM" => Type::Ntm,
            _ => {
                ctx.emit_error(
                    "unknown type, expected 'DFA' | 'NFA' | 'DPDA' | 'NPDA' | 'TM' | 'NTM'",
                    span,
                );
                return None;
            }
        })
    }

    const D: Options = Options {
        non_deterministic: false,
        epsilon_moves: false,
    };

    const N: Options = Options {
        non_deterministic: true,
        epsilon_moves: true,
    };

    Some(match parse_type(items.next(), ctx)? {
        Type::Dfa => Machine::Fa(fa::Fa::compile(items, ctx, D)?),
        Type::Nfa => Machine::Fa(fa::Fa::compile(items, ctx, N)?),
        Type::Dpda => Machine::Pda(pda::Pda::compile(items, ctx, D)?),
        Type::Npda => Machine::Pda(pda::Pda::compile(items, ctx, N)?),
        Type::Tm => Machine::Tm(tm::Tm::compile(items, ctx, D)?),
        Type::Ntm => Machine::Tm(tm::Tm::compile(items, ctx, N)?),
    })
}
