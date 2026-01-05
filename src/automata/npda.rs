use std::collections::HashSet;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct To(State, Vec<Symbol>);

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct TransitionTable {
    initial_state: State,
    initial_stack: Symbol,
    state_names: StateMap<String>,
    symbol_names: SymbolMap<String>,
    alphabet: HashSet<char>,

    final_states: Option<StateMap<bool>>,
    transitions: StateSymbolMap<CharEpsilonMap<Vec<To>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NPDA {
    pub state: State,
    pub stack: Vec<Symbol>,
    pub position: usize,
}

pub struct Simulator {
    input: String,
    table: TransitionTable,
    running: Vec<NPDA>,
}

pub enum SimulatorResult {
    Pending,
    Reject,
    Accept(NPDA),
}

impl Simulator {
    pub fn begin(input: impl Into<String>, table: TransitionTable) -> Self {
        Self {
            input: input.into(),
            running: vec![NPDA {
                state: table.initial_state,
                stack: vec![table.initial_stack],
                position: 0,
            }],
            table,
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
                .table
                .transitions
                .get((npda.state, top))
                .and_then(|t| t.get(None))
                .iter()
                .flat_map(|t| t.iter())
            {
                let mut stack = npda.stack.clone();
                stack.extend_from_slice(&to.1);
                new.push(NPDA {
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
                if let Some(final_states) = &self.table.final_states
                    && final_states.get(npda.state).copied().unwrap_or_default()
                {
                    return SimulatorResult::Accept(npda.clone());
                } else if npda.stack == [self.table.initial_stack] {
                    return SimulatorResult::Accept(npda.clone());
                } else {
                    continue;
                }
            };

            for to in self
                .table
                .transitions
                .get((npda.state, top))
                .and_then(|t| t.get(Some(next)))
                .iter()
                .flat_map(|t| t.iter())
            {
                let mut stack = npda.stack.clone();
                stack.extend_from_slice(&to.1);
                new.push(NPDA {
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
    DELTA_LOWER, GAMMA_UPPER, SIGMA_UPPER, Spanned,
    ast::{self, Symbol as Sym},
    lexer::Lexer,
    log::Logs,
    parser::Parser,
};

impl TransitionTable {
    pub fn load_table<'a>(input: &'a str) -> Result<(TransitionTable, Logs<'a>), Logs<'a>> {
        let (ast, logs) = Parser::new(Lexer::new(input)).parse_elements();
        if logs.contains_errors() {
            return Err(logs);
        }
        Self::load_from_ast(&ast, logs)
    }

    pub fn load_from_ast<'a>(
        ast: &Vec<Spanned<ast::TopLevel<'a>>>,
        mut logs: Logs<'a>,
    ) -> Result<(TransitionTable, Logs<'a>), Logs<'a>> {
        let mut initial_state = None;
        let mut initial_stack = None;

        let mut states = HashMap::new();
        let mut stack_symbols = HashMap::new();
        let mut alphabet = HashSet::new();
        let mut final_states = None;

        let mut transitions_map = HashMap::new();

        for Spanned(element, span) in ast {
            use Spanned as S;
            use ast::TopLevel as TL;
            match element {
                TL::Item(S("Q", _), list) => {
                    if !states.is_empty() {
                        logs.emit_error("states already set", *span);
                    }
                    let Some(list) = list.expect_set(&mut logs) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(&mut logs) else {
                            continue;
                        };
                        let state = match states.len().try_into() {
                            Ok(ok) => State(ok),
                            Err(_) => {
                                logs.emit_error("too many states defined", item.1);
                                State(0)
                            }
                        };
                        if states.insert(ident, state).is_some() {
                            logs.emit_error("state redefined", item.1);
                        }
                    }

                    if list.is_empty() {
                        logs.emit_error("states cannot be empty", *span);
                    }
                }
                TL::Item(S("E" | SIGMA_UPPER | "sigma", _), list) => {
                    if !alphabet.is_empty() {
                        logs.emit_error("alphabet already set", *span);
                    }
                    let Some(list) = list.expect_set(&mut logs) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(&mut logs) else {
                            continue;
                        };

                        if ident.chars().count() != 1 {
                            logs.emit_error("letter cannot be longer than one char", item.1);
                        }

                        if !alphabet.insert(ident.chars().next().unwrap_or_default()) {
                            logs.emit_error("letter redefined", item.1);
                        }
                    }
                    if list.is_empty() {
                        logs.emit_error("alphabet cannot be empty", *span);
                    }
                }
                TL::Item(S("F", _), list) => {
                    if final_states.is_some() {
                        logs.emit_error("final states already set", *span);
                    }
                    let mut map = HashSet::new();
                    let Some(list) = list.expect_set(&mut logs) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(&mut logs) else {
                            continue;
                        };
                        if let Some(state) = states.get(ident) {
                            if !map.insert(*state) {
                                logs.emit_error("final state redefined", item.1);
                            }
                        } else {
                            logs.emit_error("final state not defined in set of states", item.1);
                        }
                    }
                    final_states = Some(map);
                }
                TL::Item(S("T" | GAMMA_UPPER | "gamma", _), list) => {
                    if !stack_symbols.is_empty() {
                        logs.emit_error("stack symbols already set", *span);
                    }
                    let Some(list) = list.expect_set(&mut logs) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(&mut logs) else {
                            continue;
                        };
                        let symbol = match stack_symbols.len().try_into() {
                            Ok(ok) => Symbol(ok),
                            Err(_) => {
                                logs.emit_error("too many stack symbols defined", item.1);
                                Symbol(0)
                            }
                        };
                        if stack_symbols.insert(ident, symbol).is_some() {
                            logs.emit_error("stack symbol redefined", item.1);
                        }
                    }

                    if list.is_empty() {
                        logs.emit_error("stack symbols cannot be empty", *span);
                    }
                }
                TL::Item(S("I" | "q0", _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if initial_state.is_some() {
                            logs.emit_error("initial state already set", *span);
                        }
                        if let Some(initial) = states.get(ident) {
                            initial_state = Some(*initial)
                        } else {
                            logs.emit_error("initial state symbol not defined as a state", *src_d);
                        }
                    }
                    _ => logs.emit_error("expected ident", *src_d),
                },
                TL::Item(S("S" | "z0", _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if initial_stack.is_some() {
                            logs.emit_error("initial stack already set", *span);
                        }
                        if let Some(initial) = stack_symbols.get(ident) {
                            initial_stack = Some(*initial)
                        } else {
                            logs.emit_error(
                                "initial stack symbol not defined as a stack symbol",
                                *src_d,
                            );
                        }
                    }
                    _ => logs.emit_error("expected ident", *src_d),
                },
                TL::Item(S(name, dest_s), _) => {
                    logs.emit_error(format!("unknown item {name:?}, expected 'Q'|'E'|'{SIGMA_UPPER}'|'sigma'|'F'|'T'|'{GAMMA_UPPER}'|'gamma'|'I'|'q0'|'S'|'z0'"), *dest_s);
                }

                TL::TransitionFunc(
                    S((S("d" | DELTA_LOWER | "delta", _), tuple), _),
                    list,
                ) => {
                    let list = list.set_weak();
                    let Some((state, letter, stack_symbol)) =
                        tuple.as_ref().expect_npda_transition_function(&mut logs)
                    else {
                        continue;
                    };
                    let Some(state) = states.get(state.0).copied() else {
                        logs.emit_error("transition state not defined as state", state.1);
                        continue;
                    };
                    let Some(stack_symbol) = stack_symbols.get(stack_symbol.0).copied() else {
                        logs.emit_error(
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
                                    logs.emit_error(
                                        "transition letter not defined in alphabet",
                                        letter.1,
                                    );
                                }
                                Some(char)
                            } else {
                                logs.emit_error(
                                    "transition letter can only be single character",
                                    letter.1,
                                );
                                None
                            }
                        }
                    };

                    for item in list {
                        let Some((next_state, stack)) = item
                            .expect_tuple(&mut logs)
                            .and_then(|item| item.expect_npda_transition(&mut logs))
                        else {
                            continue;
                        };

                        let Some(next_state) = states.get(next_state.0).copied() else {
                            logs.emit_error("transition state not defined as state", next_state.1);
                            continue;
                        };

                        let stack: Vec<_> = stack
                            .iter()
                            .rev()
                            .filter_map(|symbol| {
                                if matches!(symbol.0, ast::Item::Symbol(Sym::Epsilon)) {
                                    return None;
                                }
                                let ident = symbol.expect_ident(&mut logs)?;

                                let Some(symbol) = stack_symbols.get(ident).copied() else {
                                    logs.emit_error(
                                        "transition stack symbol not defined",
                                        symbol.1,
                                    );
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
                            logs.emit_warning("duplicate transition", item.1);
                        }
                    }
                }
                TL::TransitionFunc(S((S(name, _), _), dest_s), _) => {
                    logs.emit_error(
                        format!("unknown function {name:?}, expected 'd'|'delta'|'{DELTA_LOWER}'"),
                        *dest_s,
                    );
                }

                TL::ProductionRule(_, _) => {
                    logs.emit_error("unexpected production rule", *span);
                }
                TL::Table() => logs.emit_error("unexpected table", *span),
            }
        }

        if stack_symbols.is_empty() {
            logs.emit_error_locless("stack symbols never defined");
        }

        if alphabet.is_empty() {
            logs.emit_error_locless("alphabet never defined");
        }

        if states.is_empty() {
            logs.emit_error_locless("states never defined");
        }

        let initial_stack = match initial_stack {
            Some(some) => some,
            None => {
                if let Some(initial) = stack_symbols.get("z0") {
                    logs.emit_warning_locless(
                        "initial stack symbol not defined, defaulting to 'z0'",
                    );
                    *initial
                } else {
                    logs.emit_error_locless("initial stack symbol not defined");
                    Symbol(0)
                }
            }
        };

        let initial_state = match initial_state {
            Some(some) => some,
            None => {
                if let Some(initial) = states.get("q0") {
                    logs.emit_warning_locless("initial state not defined, defaulting to 'q0'");
                    *initial
                } else {
                    logs.emit_error_locless("initial state not defined");
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

        let table = TransitionTable {
            initial_state,
            initial_stack,
            state_names,
            symbol_names,
            alphabet,
            final_states,
            transitions,
        };

        Ok((table, logs))
    }
}
