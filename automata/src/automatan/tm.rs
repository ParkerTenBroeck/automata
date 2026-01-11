use std::collections::HashSet;

use super::*;

use crate::{delta_lower, dual_struct_serde, gamma_upper, loader::{
    BLANK_SYMBOL, Context, Spanned, ast::{self, Symbol as Sym}, log::LogSink
}};
dual_struct_serde! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct TransitionFrom<'a> {
        #[serde(borrow)]
        pub state: State<'a>,
        #[serde(borrow)]
        pub symbol: Symbol<'a>,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    Left,
    Right,
    None,
}

dual_struct_serde! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct TransitionTo<'a> {
        #[serde(borrow)]
        pub state: State<'a>,
        #[serde(borrow)]
        pub symbol: Symbol<'a>,
        pub direction: Direction,

        pub transition: Span,
        pub function: Span,
    }
}

dual_struct_serde! {{#[serde_with::serde_as]}
    #[derive(Clone, Debug)]
    pub struct Tm<'a> {
        #[serde(borrow)]
        pub initial_state: State<'a>,
        #[serde(borrow)]
        pub initial_tape: Symbol<'a>,
        #[serde(borrow)]
        pub states: HashMap<State<'a>, StateInfo>,
        #[serde(borrow)]
        pub symbols: HashMap<Symbol<'a>, SymbolInfo>,

        #[serde(borrow)]
        pub final_states: HashMap<State<'a>, StateInfo>,

        
        #[serde(borrow)]
        #[serde_as(as = "serde_with::Seq<(_, _)>")]
        pub transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>>,
    }
}

impl<'a> Tm<'a> {
    pub fn compile(
        items: impl Iterator<Item = Spanned<ast::TopLevel<'a>>>,
        ctx: &mut Context<'a>,
        options: Options,
    ) -> Option<Tm<'a>> {
        let mut initial_state = None;
        let mut initial_tape = None;

        let mut states = HashMap::new();
        let mut symbols = HashMap::new();
        let mut final_states = HashMap::new();

        let mut transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>> =
            HashMap::new();

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
                        if states
                            .insert(State(ident), StateInfo { definition: item.1 })
                            .is_some()
                        {
                            ctx.emit_error("state redefined", item.1);
                        }
                    }

                    if list.is_empty() {
                        ctx.emit_error("states cannot be empty", span);
                    }
                }
                TL::Item(S("F", _), list) => {
                    if !final_states.is_empty() {
                        ctx.emit_error("final states already set", span);
                    }
                    let Some(list) = list.expect_set(ctx) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(ctx) else {
                            continue;
                        };
                        if states.contains_key(&State(ident)) {
                            if final_states
                                .insert(State(ident), StateInfo { definition: item.1 })
                                .is_none()
                            {
                                ctx.emit_error("final state redefined", item.1);
                            }
                        } else {
                            ctx.emit_error("final state not defined in set of states", item.1);
                        }
                    }
                }
                TL::Item(S(gamma_upper!(pat), _), list) => {
                    if !symbols.is_empty() {
                        ctx.emit_error("tape symbols already set", span);
                    }
                    let Some(list) = list.expect_set(ctx) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(ctx) else {
                            continue;
                        };

                        if symbols
                            .insert(Symbol(ident), SymbolInfo { definition: item.1 })
                            .is_some()
                        {
                            ctx.emit_error("tape symbol redefined", item.1);
                        }
                    }

                    if list.is_empty() {
                        ctx.emit_error("tape symbols cannot be empty", span);
                    }
                }
                TL::Item(S("q0", _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if initial_state.is_some() {
                            ctx.emit_error("initial state already set", span);
                        }
                        if states.contains_key(&State(ident)) {
                            initial_state = Some(State(ident))
                        } else {
                            ctx.emit_error("initial state symbol not defined as a state", src_d);
                        }
                    }
                    _ => _ = ctx.emit_error("expected ident", src_d),
                },
                TL::Item(S(BLANK_SYMBOL, _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if initial_tape.is_some() {
                            ctx.emit_error("initial tape symbol already set", span);
                        }
                        if symbols.contains_key(&Symbol(ident)) {
                            initial_tape = Some(Symbol(ident));
                        } else {
                            ctx.emit_error(
                                "initial tape symbol not defined as a tape symbol",
                                src_d,
                            );
                        }
                    }
                    _ => _ = ctx.emit_error("expected ident", src_d),
                },
                TL::Item(S(name, dest_s), _) => {
                    ctx.emit_error(format!("unknown item {name:?}, expected states, symbols, final states, initial state, blank symbol"), dest_s);
                }

                TL::TransitionFunc(S((S(delta_lower!(pat), _), tuple), _), list) => {
                    let list = list.set_weak();
                    let Some((from_state, from_tape)) =
                        tuple.as_ref().expect_tm_transition_function(ctx)
                    else {
                        continue;
                    };
                    if !states.contains_key(&State(from_state.0)) {
                        ctx.emit_error("transition state not defined as state", from_state.1);
                        continue;
                    };
                    if !symbols.contains_key(&Symbol(from_tape.0)) {
                        ctx.emit_error(
                            "transition tape symbol not defined as tape symbol",
                            from_tape.1,
                        );
                        continue;
                    };

                    for item in list {
                        let Some((to_state, to_tape, direction)) = item
                            .expect_tuple(ctx)
                            .and_then(|item| item.expect_tm_transition(ctx))
                        else {
                            continue;
                        };

                        if !states.contains_key(&State(to_state.0)) {
                            ctx.emit_error("transition state not defined as state", to_state.1);
                            continue;
                        };

                        let entry: &mut _ = transitions
                            .entry(TransitionFrom {
                                state: State(from_state.0),
                                symbol: Symbol(from_tape.0),
                            })
                            .or_default();
                        if !entry.is_empty() && !options.non_deterministic {
                            ctx.emit_error("transition already defined for this starting point (non determinism not permitted)", item.1);
                        }
                        if !entry.insert(TransitionTo {
                            state: State(to_state.0),
                            symbol: Symbol(to_tape.0),
                            direction: direction.0,

                            function: tuple.1,
                            transition: item.1,
                        }) {
                            ctx.emit_warning("duplicate transition", item.1);
                        }
                    }
                }
                TL::TransitionFunc(S((S(name, _), _), dest_s), _) => {
                 ctx.emit_error(
                        format!(
                            "unknown function {name:?}, expected transition function ( {} )", delta_lower!(str)
                        ),
                        dest_s,
                    );
                }

                TL::ProductionRule(_, _) => {
                    ctx.emit_error("unexpected production rule", span);
                }
                TL::Table() => _ = ctx.emit_error("unexpected table", span),
            }
        }

        if symbols.is_empty() {
            ctx.emit_error_locless("tape symbols never defined");
        }

        if states.is_empty() {
            ctx.emit_error_locless("states never defined");
        }

        let initial_tape = match initial_tape {
            Some(some) => some,
            None => {
                if symbols.contains_key(&Symbol("z0")) {
                    ctx.emit_warning_locless("initial tape symbol not defined, defaulting to 'z0'");
                } else {
                    ctx.emit_error_locless("initial tape symbol not defined");
                }
                Symbol("z0")
            }
        };

        let initial_state = match initial_state {
            Some(some) => some,
            None => {
                if states.contains_key(&State("q0")) {
                    ctx.emit_warning_locless("initial state not defined, defaulting to 'q0'");
                } else {
                    ctx.emit_error_locless("initial state not defined");
                }
                State("q0")
            }
        };

        if ctx.contains_errors() {
            return None;
        }

        Some(Tm {
            initial_state,
            initial_tape,
            states,
            symbols,
            final_states,
            transitions,
        })
    }
}

impl<'a> Spanned<&ast::Tuple<'a>> {
    fn expect_tm_transition_function(
        &self,
        ctx: &mut Context<'a>,
    ) -> Option<(Spanned<&'a str>, Spanned<&'a str>)> {
        match &self.0.0[..] {
            [
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(state)), state_span),
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(tape)), tape_span),
            ] => {
                return Some((Spanned(state, *state_span), Spanned(*tape, *tape_span)));
            }
            _ => _ = ctx.emit_error("expected TM transition function (state, symbol)", self.1),
        }
        None
    }

    fn expect_tm_transition(
        &self,
        ctx: &mut Context<'a>,
    ) -> Option<(Spanned<&'a str>, Spanned<&'a str>, Spanned<Direction>)> {
        match &self.0.0[..] {
            [
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(state)), state_span),
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(tape)), tape_span),
                Spanned(ast::Item::Symbol(direction), direction_span),
            ] => {
                let direction = match direction {
                    ast::Symbol::Ident("left" | "L" | "<") => Direction::Left,
                    ast::Symbol::Ident("right" | "R" | ">") => Direction::Right,
                    ast::Symbol::Epsilon(_) | ast::Symbol::Ident("~") => Direction::None,
                    ast::Symbol::Ident(ident) => {
                        ctx.emit_error(
                            format!("invalid direction specified '{ident}'"),
                            *direction_span,
                        );
                        Direction::None
                    }
                };
                return Some((
                    Spanned(state, *state_span),
                    Spanned(*tape, *tape_span),
                    Spanned(direction, *direction_span),
                ));
            }
            _ => _ = ctx.emit_error(
                "expected TM transition function (state, symbol, direction)",
                self.1,
            ),
        }
        None
    }
}
