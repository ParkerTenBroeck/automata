use crate::{automata::npda, loader::ast::TopLevel};

pub mod ast;
pub mod lexer;
pub mod log;
pub mod parser;

pub const EPSILON_LOWER: &str = "Ɛ";
pub const EPSILON_LOWER_MATH: &str = "𝛆";
pub const DELTA_LOWER: &str = "δ";
pub const SIGMA_UPPER: &str = "Σ";
pub const GAMMA_UPPER: &str = "Γ";
pub const GAMMA_LOWER: &str = "γ";

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

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned(&self.0, self.1)
    }
}


pub struct Context<'a>{
    logs: log::Logs,
    src: &'a str
}

impl<'a> Context<'a>{
    pub fn new(src: &'a str) -> Self{
        Self { logs: log::Logs::new(), src }
    }

    pub fn src(&self) -> &'a str{
        self.src
    }

    pub fn logs_display(&self) -> impl Iterator<Item = log::LogEntryDisplay<'_>>{
        self.logs.displayable_with(self.src)
    }

    pub fn eof(&self) -> Span{
        Span(self.src.len(), self.src.len())
    }

    pub fn emit(&mut self, entry: log::LogEntry) {
        self.logs.emit(entry);
    }

    pub fn emit_error_locless(&mut self, msg: impl Into<String>) {
        self.logs.emit_error_locless(msg);
    }

    pub fn emit_error(&mut self, msg: impl Into<String>, span: Span) {
        self.logs.emit_error(msg, span);
    }

    pub fn emit_warning(&mut self, msg: impl Into<String>, span: Span) {
        self.logs.emit_warning(msg, span);
    }

    pub fn emit_warning_locless(&mut self, msg: impl Into<String>) {
        self.logs.emit_warning_locless(msg);
    }

    pub fn emit_info(&mut self, msg: impl Into<String>, span: Span) {
        self.logs.emit_info(msg, span);
    }
    
    pub fn contains_errors(&self) -> bool {
        self.logs.contains_errors()
    }

    pub fn into_logs(self) -> log::Logs{
        self.logs
    }
}


pub enum Machine{
    Npda(npda::Npda)
}

pub fn parse_universal(ctx: &mut Context<'_>) -> Option<Machine> {
    let mut items = parser::Parser::new(ctx).collect::<Vec<_>>().into_iter();
    if ctx.logs.contains_errors(){
        return None;
    }

    use Spanned as S;

    #[derive(Debug)]
    enum Type{
        Dfa,
        Nfa,
        Dpda,
        Npda,
        Tm,
        Ntm
    }

    fn parse_type<'a>(item: Option<S<TopLevel<'a>>>, ctx: &mut Context<'a>) -> Option<Type>{
        let (str, span) = match item{
            Some(S(TopLevel::Item(S("type", _), item@S(_,span)), _)) => (item.expect_ident(ctx)?, span),
            Some(S(_, span)) => {
                ctx.emit_error("expected type=<type> as first item", span);
                return None;
            }
            None => {
                ctx.emit_error("expected type=<type> as first item", ctx.eof());
                return None;
            }
        };

        Some(match str{
            "dfa"|"DFA" => Type::Dfa,
            "nfa"|"NFA" => Type::Nfa,
            "dpda"|"DPDA" => Type::Dpda,
            "npdaA"|"NPDA" => Type::Npda,
            "tm"|"TM" => Type::Tm,
            "ntm"|"NTM" => Type::Ntm,
            _ => {
                ctx.emit_error("unknown type, expected 'DFA' | 'NFA' | 'DPDA' | 'NPDA' | 'TM' | 'NTM'", span);
                return None;
            },
        })
    }

    Some(match parse_type(items.next(), ctx)?{
        Type::Npda => Machine::Npda(npda::Npda::load_from_ast(items, ctx)?),
        ty => {
            ctx.emit_error_locless(format!("currently unsupported type {ty:?}"));
            return None;
        }
    })
}