use std::collections::HashMap;

use crate::loader::Span;

pub mod fa;
pub mod pda;
pub mod tm;


#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub non_deterministic: bool,
    pub epsilon_moves: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
pub struct State<'a>(pub &'a str);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
pub struct Symbol<'a>(pub &'a str);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
pub struct Letter<'a>(pub &'a str);

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateInfo {
    pub definition: Span,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolInfo {
    pub definition: Span,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LetterInfo {
    pub definition: Span,
}
