use std::collections::HashSet;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct To(State, Vec<Symbol>);

impl To{
    pub fn state(&self) -> State{
        self.0
    }

    pub fn stack(&self) -> &[Symbol]{
        &self.1
    }
}

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Npda {
    initial_state: State,
    initial_stack: Symbol,
    state_names: StateMap<String>,
    symbol_names: SymbolMap<String>,
    alphabet: HashSet<char>,

    final_states: Option<StateMap<bool>>,
    transitions: StateSymbolMap<CharEpsilonMap<Vec<To>>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateTransition<T> {
    pub from: T,
    pub to: T,
}

impl Npda {
    pub fn get_state_name(&self, state: State) -> Option<&str>{
        self.state_names.get(state).map(String::as_str)
    }

    pub fn get_symbol_name(&self, symbol: Symbol) -> Option<&str>{
        self.symbol_names.get(symbol).map(String::as_str)
    }
    
    pub fn initial_state(&self) -> State{
        self.initial_state
    }

    pub fn initial_stack(&self) -> Symbol{
        self.initial_stack
    }

    pub fn final_states(&self) -> Option<impl Iterator<Item = State>>{
        Some(self.final_states.as_ref()?.entries().filter(|&(_, f)| *f).map(|(s, _)| s))
    }
    
    pub fn states(&self) -> impl Iterator<Item = (State, &str)>{
        self.state_names.entries().map(|s|(s.0, s.1.as_str()))
    }

    pub fn symbols(&self) -> impl Iterator<Item = (Symbol, &str)>{
        self.symbol_names.entries().map(|s|(s.0, s.1.as_str()))
    }

    pub fn transitions(&self) -> &StateSymbolMap<CharEpsilonMap<Vec<To>>>{
        &self.transitions
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NpdaState {
    pub state: State,
    pub stack: Vec<Symbol>,
    pub position: usize,
}

pub struct Simulator {
    input: String,
    machine: Npda,
    running: Vec<NpdaState>,
}

pub enum SimulatorResult {
    Pending,
    Reject,
    Accept(NpdaState),
}

impl Simulator {
    pub fn begin(input: impl Into<String>, machine: Npda) -> Self {
        Self {
            input: input.into(),
            running: vec![NpdaState {
                state: machine.initial_state,
                stack: vec![machine.initial_stack],
                position: 0,
            }],
            machine,
        }
    }

    pub fn step(&mut self) -> SimulatorResult {
        println!("step, ({}) paths", self.running.len());
        let mut new = Vec::new();
        for mut npda in self.running.drain(..) {
            let Some(top) = npda.stack.pop() else {
                continue;
            };

            for to in self
                .machine
                .transitions
                .get((npda.state, top))
                .and_then(|t| t.get(None))
                .iter()
                .flat_map(|t| t.iter())
            {
                let mut stack = npda.stack.clone();
                stack.extend_from_slice(&to.1);
                new.push(NpdaState {
                    state: to.0,
                    stack,
                    position: npda.position,
                });
            }

            let Some(next) = self
                .input
                .get(npda.position..)
                .and_then(|c| c.chars().next())
            else {
                if let Some(final_states) = &self.machine.final_states
                    && final_states.get(npda.state).copied().unwrap_or_default()
                {
                    return SimulatorResult::Accept(npda.clone());
                } else if npda.stack == [self.machine.initial_stack] {
                    return SimulatorResult::Accept(npda.clone());
                } else {
                    continue;
                }
            };

            for to in self
                .machine
                .transitions
                .get((npda.state, top))
                .and_then(|t| t.get(Some(next)))
                .iter()
                .flat_map(|t| t.iter())
            {
                let mut stack = npda.stack.clone();
                stack.extend_from_slice(&to.1);
                new.push(NpdaState {
                    state: to.0,
                    stack,
                    position: npda.position + next.len_utf8(),
                });
            }
        }
        self.running = new;
        if self.running.is_empty() {
            SimulatorResult::Reject
        } else {
            SimulatorResult::Pending
        }
    }
}

// ------ parser/semantics

use crate::loader::{
    Context, DELTA_LOWER, GAMMA_UPPER, SIGMA_UPPER, Spanned,
    ast::{self, Symbol as Sym},
};

impl Npda {
    pub fn load_from_ast<'a>(
        items: impl Iterator<Item = Spanned<ast::TopLevel<'a>>>,
        ctx: &mut Context<'a>,
    ) -> Option<Npda> {
        let mut initial_state = None;
        let mut initial_stack = None;

        let mut states = HashMap::new();
        let mut stack_symbols = HashMap::new();
        let mut alphabet = HashSet::new();
        let mut final_states = None;

        let mut transitions_map = HashMap::new();

        for Spanned(element, span) in items {
            use Spanned as S;
            use ast::TopLevel as TL;
            match element {
                TL::Item(S("Q", _), list) => {
                    if !states.is_empty() {
                        ctx.emit_error("states already set", span);
                    }
                    let Some(list) = list.expect_set(ctx) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(ctx) else {
                            continue;
                        };
                        let state = match states.len().try_into() {
                            Ok(ok) => State(ok),
                            Err(_) => {
                                ctx.emit_error("too many states defined", item.1);
                                State(0)
                            }
                        };
                        if let Some(old) = states.insert(ident, state) {
                            ctx.emit_error("state redefined", item.1);
                            states.insert(ident, old);
                        }
                    }

                    if list.is_empty() {
                        ctx.emit_error("states cannot be empty", span);
                    }
                }
                TL::Item(S("E" | SIGMA_UPPER | "sigma", _), list) => {
                    if !alphabet.is_empty() {
                        ctx.emit_error("alphabet already set", span);
                    }
                    let Some(list) = list.expect_set(ctx) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(ctx) else {
                            continue;
                        };

                        if ident.chars().count() != 1 {
                            ctx.emit_error("letter cannot be longer than one char", item.1);
                        }

                        if !alphabet.insert(ident.chars().next().unwrap_or_default()) {
                            ctx.emit_error("letter redefined", item.1);
                        }
                    }
                    if list.is_empty() {
                        ctx.emit_error("alphabet cannot be empty", span);
                    }
                }
                TL::Item(S("F", _), list) => {
                    if final_states.is_some() {
                        ctx.emit_error("final states already set", span);
                    }
                    let mut map = HashSet::new();
                    let Some(list) = list.expect_set(ctx) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(ctx) else {
                            continue;
                        };
                        if let Some(state) = states.get(ident) {
                            if !map.insert(*state) {
                                ctx.emit_error("final state redefined", item.1);
                            }
                        } else {
                            ctx.emit_error("final state not defined in set of states", item.1);
                        }
                    }
                    final_states = Some(map);
                }
                TL::Item(S("T" | GAMMA_UPPER | "gamma", _), list) => {
                    if !stack_symbols.is_empty() {
                        ctx.emit_error("stack symbols already set", span);
                    }
                    let Some(list) = list.expect_set(ctx) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(ctx) else {
                            continue;
                        };
                        let symbol = match stack_symbols.len().try_into() {
                            Ok(ok) => Symbol(ok),
                            Err(_) => {
                                ctx.emit_error("too many stack symbols defined", item.1);
                                Symbol(0)
                            }
                        };
                        if let Some(old) = stack_symbols.insert(ident, symbol) {
                            ctx.emit_error("stack symbol redefined", item.1);
                            stack_symbols.insert(ident, old);
                        }
                    }

                    if list.is_empty() {
                        ctx.emit_error("stack symbols cannot be empty", span);
                    }
                }
                TL::Item(S("I" | "q0", _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if initial_state.is_some() {
                            ctx.emit_error("initial state already set", span);
                        }
                        if let Some(initial) = states.get(ident) {
                            initial_state = Some(*initial)
                        } else {
                            ctx.emit_error("initial state symbol not defined as a state", src_d);
                        }
                    }
                    _ => ctx.emit_error("expected ident", src_d),
                },
                TL::Item(S("S" | "z0", _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if initial_stack.is_some() {
                            ctx.emit_error("initial stack already set", span);
                        }
                        if let Some(initial) = stack_symbols.get(ident) {
                            initial_stack = Some(*initial)
                        } else {
                            ctx.emit_error(
                                "initial stack symbol not defined as a stack symbol",
                                src_d,
                            );
                        }
                    }
                    _ => ctx.emit_error("expected ident", src_d),
                },
                TL::Item(S(name, dest_s), _) => {
                    ctx.emit_error(format!("unknown item {name:?}, expected 'Q' | 'E' | '{SIGMA_UPPER}' | 'sigma' | 'F' | 'T' | '{GAMMA_UPPER}' | 'gamma' | 'I' | 'q0' | 'S' | 'z0'"), dest_s);
                }

                TL::TransitionFunc(S((S("d" | DELTA_LOWER | "delta", _), tuple), _), list) => {
                    let list = list.set_weak();
                    let Some((state, letter, stack_symbol)) =
                        tuple.as_ref().expect_npda_transition_function(ctx)
                    else {
                        continue;
                    };
                    let Some(state) = states.get(state.0).copied() else {
                        ctx.emit_error("transition state not defined as state", state.1);
                        continue;
                    };
                    let Some(stack_symbol) = stack_symbols.get(stack_symbol.0).copied() else {
                        ctx.emit_error(
                            "transition stack symbol not defined as stack symbol",
                            stack_symbol.1,
                        );
                        continue;
                    };

                    let char = match letter.0 {
                        Sym::Epsilon => None,
                        Sym::Ident(val) => {
                            if let Some(char) = val.chars().next()
                                && val.chars().count() == 1
                            {
                                if !alphabet.contains(&char) {
                                    ctx.emit_error(
                                        "transition letter not defined in alphabet",
                                        letter.1,
                                    );
                                }
                                Some(char)
                            } else {
                                ctx.emit_error(
                                    "transition letter can only be single character",
                                    letter.1,
                                );
                                None
                            }
                        }
                    };

                    for item in list {
                        let Some((next_state, stack)) = item
                            .expect_tuple(ctx)
                            .and_then(|item| item.expect_npda_transition(ctx))
                        else {
                            continue;
                        };

                        let Some(next_state) = states.get(next_state.0).copied() else {
                            ctx.emit_error("transition state not defined as state", next_state.1);
                            continue;
                        };

                        let stack: Vec<_> = stack
                            .iter()
                            .rev()
                            .filter_map(|symbol| {
                                if matches!(symbol.0, ast::Item::Symbol(Sym::Epsilon)) {
                                    return None;
                                }
                                let ident = symbol.expect_ident(ctx)?;

                                let Some(symbol) = stack_symbols.get(ident).copied() else {
                                    ctx.emit_error("transition stack symbol not defined", symbol.1);
                                    return None;
                                };
                                Some(symbol)
                            })
                            .collect();

                        if !transitions_map
                            .entry((state, char, stack_symbol))
                            .or_insert(HashSet::new())
                            .insert((next_state, stack))
                        {
                            ctx.emit_warning("duplicate transition", item.1);
                        }
                    }
                }
                TL::TransitionFunc(S((S(name, _), _), dest_s), _) => {
                    ctx.emit_error(
                        format!(
                            "unknown function {name:?}, expected 'd' | 'delta' | '{DELTA_LOWER}'"
                        ),
                        dest_s,
                    );
                }

                TL::ProductionRule(_, _) => {
                    ctx.emit_error("unexpected production rule", span);
                }
                TL::Table() => ctx.emit_error("unexpected table", span),
            }
        }

        if stack_symbols.is_empty() {
            ctx.emit_error_locless("stack symbols never defined");
        }

        if alphabet.is_empty() {
            ctx.emit_error_locless("alphabet never defined");
        }

        if states.is_empty() {
            ctx.emit_error_locless("states never defined");
        }

        let initial_stack = match initial_stack {
            Some(some) => some,
            None => {
                if let Some(initial) = stack_symbols.get("z0") {
                    ctx.emit_warning_locless(
                        "initial stack symbol not defined, defaulting to 'z0'",
                    );
                    *initial
                } else {
                    ctx.emit_error_locless("initial stack symbol not defined");
                    Symbol(0)
                }
            }
        };

        let initial_state = match initial_state {
            Some(some) => some,
            None => {
                if let Some(initial) = states.get("q0") {
                    ctx.emit_warning_locless("initial state not defined, defaulting to 'q0'");
                    *initial
                } else {
                    ctx.emit_error_locless("initial state not defined");
                    State(0)
                }
            }
        };

        let state_names = StateMap(states.iter().fold(
            vec![String::new(); states.len()],
            |mut a, (k, v)| {
                a[v.0 as usize] = k.to_string();
                a
            },
        ));
        let symbol_names = SymbolMap(stack_symbols.iter().fold(
            vec![String::new(); stack_symbols.len()],
            |mut a, (k, v)| {
                a[v.0 as usize] = k.to_string();
                a
            },
        ));

        let final_states = final_states.map(|f| {
            StateMap(f.iter().fold(vec![false; states.len()], |mut a, k| {
                a[k.0 as usize] = true;
                a
            }))
        });

        let mut transitions: StateSymbolMap<CharEpsilonMap<Vec<To>>> = StateSymbolMap {
            map: vec![CharEpsilonMap::default(); stack_symbols.len() * states.len()],
            max_state: states.len() as u16,
        };

        for ((q, c, s), to) in transitions_map {
            let from = &mut transitions[(q, s)];
            for (n, ss) in to {
                from.get_mut_or_insert_default(c).push(To(n, ss));
            }
        }

        if ctx.contains_errors() {
            return None;
        }

        Some(Npda {
            initial_state,
            initial_stack,
            state_names,
            symbol_names,
            alphabet,
            final_states,
            transitions,
        })
    }
}
