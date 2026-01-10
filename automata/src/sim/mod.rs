use std::collections::HashMap;

pub mod dfa;
pub mod dpda;
pub mod nfa;
pub mod npda;
pub mod ntm;
pub mod tm;

pub trait Get<Idx> {
    type Output;
    fn get(&self, idx: Idx) -> Option<&Self::Output>;
    fn get_mut(&mut self, idx: Idx) -> Option<&mut Self::Output>;
}

pub trait GetDefault<Idx> {
    type Output: Default;
    fn get_or_insert_default(&mut self, idx: Idx) -> &Self::Output;
    fn get_mut_or_insert_default(&mut self, idx: Idx) -> &mut Self::Output;
}

macro_rules! index {
    ($ty: ident, $self:ident, $collection: expr, $index_calc: expr, $index: pat = $index_ty: ty $(, $default: expr)?) => {
        impl<T> Get<$index_ty> for $ty<T> {
            type Output = T;
            fn get(&$self, $index: $index_ty) -> Option<&T>{
                $collection.get($index_calc)
            }

            fn get_mut(&mut $self, $index: $index_ty) -> Option<&mut T>{
                $collection.get_mut($index_calc)
            }
        }

        impl<T> std::ops::Index<$index_ty> for $ty<T>{
            type Output = T;

            fn index(& $self, $index: $index_ty) -> &T{
                $collection.get($index_calc).unwrap()
            }
        }

        impl<T> std::ops::IndexMut<$index_ty> for $ty<T>{
            fn index_mut(&mut $self, $index: $index_ty) -> &mut T{
                $collection.get_mut($index_calc).unwrap()
            }
        }

        $(
            impl<T: Default> GetDefault<$index_ty> for $ty<T> {
                type Output = T;
                fn get_or_insert_default(&mut $self, $index: $index_ty) -> &T{
                    $default
                }

                fn get_mut_or_insert_default(&mut $self, $index: $index_ty) -> &mut T{
                    $default
                }
            }
        )?
    };
}

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct State(u16);

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct Symbol(u16);

#[derive(Clone, Debug)]
pub struct StateMap<T>(Vec<T>);

index!(StateMap, self, self.0, index.0 as usize, index = State);

impl<T> StateMap<T>{
    pub fn entries(&self) -> impl Iterator<Item = (State, &T)>{
        self.0.iter().enumerate().map(|(i, v)|(State(i as u16), v))
    }
}

#[derive(Clone, Debug)]
pub struct SymbolMap<T>(Vec<T>);

impl<T> SymbolMap<T>{
    pub fn entries(&self) -> impl Iterator<Item = (Symbol, &T)>{
        self.0.iter().enumerate().map(|(i, v)|(Symbol(i as u16), v))
    }
}

index!(SymbolMap, self, self.0, index.0 as usize, index = Symbol);

#[derive(Clone, Debug, Default)]
pub struct StateSymbolMap<T> {
    map: Vec<T>,
    max_state: u16,
}

impl<T> StateSymbolMap<T>{
    pub fn entries(&self) -> impl Iterator<Item = ((State, Symbol), &T)>{
        self.map.iter().enumerate().map(|(i, v)|{
            let state = State((i % self.max_state as usize) as u16);
            let symbol = Symbol((i / self.max_state as usize) as u16);
            ((state, symbol), v)
        })
    }
}

index!(
    StateSymbolMap,
    self,
    self.map,
    state.0 as usize + self.max_state as usize * symbol.0 as usize,
    (state, symbol) = (State, Symbol)
);
index!(
    StateSymbolMap,
    self,
    self.map,
    state.0 as usize + self.max_state as usize * symbol.0 as usize,
    (symbol, state) = (Symbol, State)
);

#[derive(Clone, Debug, Default)]
pub struct CharMap<T>(HashMap<char, T>);

impl<T> CharMap<T>{
    pub fn entries(&self) -> impl Iterator<Item = (char, &T)>{
        self.0.iter().map(|(k, v)|(*k, v))
    }
}

index!(
    CharMap,
    self,
    self.0,
    &char,
    char = char,
    self.0.entry(char).or_default()
);

#[derive(Clone, Debug, Default)]
pub struct CharEpsilonMap<T>(HashMap<Option<char>, T>);

impl<T> CharEpsilonMap<T>{
    pub fn entries(&self) -> impl Iterator<Item = (Option<char>, &T)>{
        self.0.iter().map(|(k, v)|(*k, v))
    }
}

index!(
    CharEpsilonMap,
    self,
    self.0,
    &Some(char),
    char = char,
    self.0.entry(Some(char)).or_default()
);
index!(CharEpsilonMap, self, self.0, &char, char = Option<char>, self.0.entry(char).or_default());

