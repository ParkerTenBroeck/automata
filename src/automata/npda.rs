use std::collections::HashSet;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct To(State, Vec<Symbol>);

#[derive(Clone, Debug)]
pub struct TransitionTable {
    pub(in super::npda) initial_state: State,
    initial_stack: Symbol,
    state_names: Vec<String>,
    symbol_names: Vec<String>,
    alphabet: HashSet<char>,

    accept_empty: bool,
    final_states: Vec<bool>,
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

    pub fn step(&mut self) -> Option<NPDA> {
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
                if self.table.final_states[npda.state.0 as usize]
                    || self.table.accept_empty && npda.stack == [self.table.initial_stack]
                {
                    return Some(npda.clone());
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
        None
    }
}

// ------ parser/semantics

use crate::loader::{
    DELTA_LOWER, GAMMA_UPPER, SIGMA_UPPER, Spanned,
    ast::{self, Symbol as Sym, Tuple},
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

        let mut states = HashSet::new();
        let mut stack_symbols = HashSet::new();
        let mut alphabet = HashSet::new();
        let mut final_states = None;
        let mut accept_empty = false;

        for Spanned(element, span) in ast {
            use Spanned as S;
            use ast::Dest;
            use ast::TopLevel as TL;
            match element {
                TL::Assignment(S(Dest::Ident("Q"), _), list) => {
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
                        if !states.insert(ident) {
                            logs.emit_error("state redefined", item.1);
                        }
                    }   

                    if list.is_empty(){
                        logs.emit_error("states cannot be empty", *span);
                    }
                }
                TL::Assignment(S(Dest::Ident("E" | SIGMA_UPPER | "sigma"), _), list) => {
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

                        if !alphabet.insert(ident) {
                            logs.emit_error("letter redefined", item.1);
                        }
                    }
                    if list.is_empty(){
                        logs.emit_error("alphabet cannot be empty", *span);
                    }
                }
                TL::Assignment(S(Dest::Ident("F"), _), list) => {
                    let mut map = HashSet::new();
                    let Some(list) = list.expect_set(&mut logs) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(&mut logs) else {
                            continue;
                        };
                        if !states.contains(ident) {
                            logs.emit_error("final state not defined in set of states", item.1);
                        }
                        if !map.insert(ident) {
                            logs.emit_error("final states redefined", item.1);
                        }
                    }

                    if final_states.is_some() {
                        logs.emit_error("final states already set", *span);
                    }
                    final_states = Some(map);
                }
                TL::Assignment(S(Dest::Ident("T" | GAMMA_UPPER | "gamma"), _), list) => {
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
                        if !stack_symbols.insert(ident) {
                            logs.emit_error("stack symbol redefined", item.1);
                        }
                    }

                    if list.is_empty(){
                        logs.emit_error("stack symbols cannot be empty", *span);
                    }
                }
                TL::Assignment(S(Dest::Ident("I" | "q0"), _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if !states.contains(ident) {
                            logs.emit_error("initial state symbol not defined as a state", *src_d);
                        }
                        if initial_state.is_some() {
                            logs.emit_error("initial state already set", *span);
                        }
                        initial_state = Some(ident)
                    }
                    _ => logs.emit_error("expected ident", *src_d),
                },
                TL::Assignment(S(Dest::Ident("S" | "z0"), _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if !stack_symbols.contains(ident)
                        {
                            logs.emit_error(
                                "initial stack symbol not defined as a stack symbol",
                                *src_d,
                            );
                        }
                        if initial_stack.is_some() {
                            logs.emit_error("initial stack already set", *span);
                        }
                        initial_stack = Some(ident)
                    }
                    _ => logs.emit_error("expected ident", *src_d),
                },
                TL::Assignment(S(Dest::Ident(name), dest_s), _) => {
                    logs.emit_error(format!("unknown item {name:?}, expected 'Q'|'E'|'{SIGMA_UPPER}'|'sigma'|'F'|'T'|'{GAMMA_UPPER}'|'gamma'|'I'|'q0'|'S'|'z0'"), *dest_s);
                }

                TL::Assignment(
                    S(Dest::Function(S("d" | DELTA_LOWER | "delta", _), tuple), _),
                    list,
                ) => {
                    let list = list.set_weak();
                    let Some((state, letter, sym)) =
                        tuple.as_ref().expect_npda_transition_function(&mut logs)
                    else {
                        continue;
                    };
                    if !states.contains(state.0){
                        logs.emit_error("transition state not defined as state", state.1);
                    }
                    if !stack_symbols.contains(sym.0){
                        logs.emit_error("transition stack symbol not defined as stack symbol", sym.1);
                    }
                    
                    for item in list {
                        let Some((next_state, stack)) = item
                            .expect_tuple(&mut logs)
                            .and_then(|item| item.expect_npda_transition(&mut logs))
                        else {
                            continue;
                        };

                        if !states.contains(next_state.0){
                            logs.emit_error("transition state not defined as state", next_state.1);
                        }
                    }
                }
                TL::Assignment(S(Dest::Function(S(name, _), _), dest_s), _) => {
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

        let table = TransitionTable {
            initial_state: crate::automata::State(0),
            initial_stack: crate::automata::Symbol(0),
            state_names: Vec::new(),
            symbol_names: Vec::new(),
            alphabet: HashSet::new(),
            accept_empty: false,
            final_states: Vec::new(),
            transitions: Default::default(),
        };

        Ok((table, logs))
    }
}
