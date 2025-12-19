pub mod ast;
pub mod lexer;
pub mod log;
pub mod parser;

pub const EPSILON_LOWER: &str = "Ɛ";
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
