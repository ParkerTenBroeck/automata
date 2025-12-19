use std::collections::HashMap;

pub mod dfa;
pub mod dpda;
pub mod nfa;
pub mod npda;
pub mod ntm;
pub mod tm;

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct State(u16);

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct Symbol(u16);

#[derive(Clone, Debug)]
pub struct StateMap<T>(Vec<T>);

trait Get<Idx> {
    type Output;
    fn get(&self, index: Idx) -> Option<&Self::Output>;
}

impl<T> Get<State> for StateMap<T> {
    type Output = T;

    fn get(&self, index: State) -> Option<&Self::Output> {
        self.0.get(index.0 as usize)
    }
}

#[derive(Clone, Debug)]
pub struct SymbolMap<T>(Vec<T>);

impl<T> Get<Symbol> for SymbolMap<T> {
    type Output = T;

    fn get(&self, index: Symbol) -> Option<&Self::Output> {
        self.0.get(index.0 as usize)
    }
}

#[derive(Clone, Debug, Default)]
pub struct StateSymbolMap<T> {
    map: Vec<T>,
    max_state: u16,
}

impl<T> Get<(State, Symbol)> for StateSymbolMap<T> {
    type Output = T;

    fn get(&self, (state, symbol): (State, Symbol)) -> Option<&Self::Output> {
        self.map
            .get(state.0 as usize + self.max_state as usize * symbol.0 as usize)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CharMap<T>(HashMap<char, T>);
impl<T> Get<char> for CharMap<T> {
    type Output = T;

    fn get(&self, index: char) -> Option<&Self::Output> {
        self.0.get(&index)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CharEpsilonMap<T>(HashMap<Option<char>, T>);

impl<T> Get<char> for CharEpsilonMap<T> {
    type Output = T;

    fn get(&self, index: char) -> Option<&Self::Output> {
        self.0.get(&Some(index))
    }
}

impl<T> Get<Option<char>> for CharEpsilonMap<T> {
    type Output = T;

    fn get(&self, index: Option<char>) -> Option<&Self::Output> {
        self.0.get(&index)
    }
}
