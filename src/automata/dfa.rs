use super::*;

pub struct TransitionTable {
    initial: State,
    state_names: Vec<String>,
    // transitions: Vec<SymbolMap<State>>,
    final_states: Vec<bool>,
}

pub struct DFA {
    state: State,
}
