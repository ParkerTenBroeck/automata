use std::collections::HashSet;

use super::*;

use crate::loader::{
    Context, DELTA_LOWER, GAMMA_UPPER, SIGMA_UPPER, Spanned,
    ast::{self, Symbol as Sym},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TransitionFrom<'a> {
    pub state: State<'a>,
    pub letter: Option<Letter<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TransitionTo<'a> {
    pub state: State<'a>,

    pub transition: Span,
    pub function: Span,
}

#[derive(Clone, Debug)]
#[allow(unused)]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Fa<'a> {
    pub initial_state: State<'a>,
    pub states: HashMap<State<'a>, StateInfo>,
    pub alphabet: HashMap<Letter<'a>, LetterInfo>,
    pub final_states: HashMap<State<'a>, StateInfo>,
    
    #[cfg(feature = "serde")]
    #[serde_as(as = "serde_with::Seq<(_, _)>")]
    pub transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>>,
    #[cfg(not(feature = "serde"))]
    pub transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>>,
}

impl<'a> Fa<'a> {
    pub fn parse(
        items: impl Iterator<Item = Spanned<ast::TopLevel<'a>>>,
        ctx: &mut Context<'a>,
        options: Options,
    ) -> Option<Fa<'a>> {

        let mut initial_state = None;

        let mut states = HashMap::new();
        let mut alphabet = HashMap::new();
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

                        if alphabet
                            .insert(Letter(ident), LetterInfo { definition: item.1 })
                            .is_some()
                        {
                            ctx.emit_error("letter redefined", item.1);
                        }
                    }
                    if list.is_empty() {
                        ctx.emit_error("alphabet cannot be empty", span);
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
                                .is_some()
                            {
                                ctx.emit_error("final state redefined", item.1);
                            }
                        } else {
                            ctx.emit_error("final state not defined in set of states", item.1);
                        }
                    }
                }
                TL::Item(S("I" | "q0", _), S(src, src_d)) => match src {
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
                    _ => ctx.emit_error("expected ident", src_d),
                },
                TL::Item(S(name, dest_s), _) => {
                    ctx.emit_error(format!("unknown item {name:?}, expected 'Q' | 'E' | '{SIGMA_UPPER}' | 'sigma' | 'F' | 'T' | '{GAMMA_UPPER}' | 'gamma' | 'I' | 'q0' | 'S' | 'z0'"), dest_s);
                }

                TL::TransitionFunc(S((S("d" | DELTA_LOWER | "delta", _), tuple), _), list) => {
                    let list = list.set_weak();
                    let Some((state, letter)) = tuple.as_ref().expect_fa_transition_function(ctx)
                    else {
                        continue;
                    };
                    if !states.contains_key(&State(state.0)) {
                        ctx.emit_error("transition state not defined as state", state.1);
                        continue;
                    };

                    let letter: Option<Letter<'_>> = match letter.0 {
                        Sym::Epsilon => {
                            if !options.epsilon_moves {
                                ctx.emit_error("epsilon moves not permitted", letter.1);
                            }
                            None
                        }
                        Sym::Ident(val) => {
                            if !alphabet.contains_key(&Letter(val)) {
                                ctx.emit_error(
                                    "transition letter not defined in alphabet",
                                    letter.1,
                                );
                            }
                            Some(Letter(val))
                        }
                    };

                    for item in list {
                        let Some(next_state) = item.expect_ident(ctx) else {
                            continue;
                        };
                        let next_state = Spanned(next_state, item.1);

                        if !states.contains_key(&State(next_state.0)) {
                            ctx.emit_error("transition state not defined as state", next_state.1);
                            continue;
                        };

                        let entry: &mut _ = transitions
                            .entry(TransitionFrom {
                                letter,
                                state: State(state.0),
                            })
                            .or_default();
                        if !entry.is_empty() && !options.non_deterministic {
                            ctx.emit_error("transition already defined for this starting point (non determinism not permitted)", item.1);
                        }
                        if !entry.insert(TransitionTo {
                            state: State(next_state.0),

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

        if alphabet.is_empty() {
            ctx.emit_error_locless("alphabet never defined");
        }

        if states.is_empty() {
            ctx.emit_error_locless("states never defined");
        }

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

        Some(Fa {
            initial_state,
            states,
            alphabet,
            final_states,
            transitions,
        })
    }
}

impl<'a> Spanned<&ast::Tuple<'a>> {
    fn expect_fa_transition_function(
        &self,
        ctx: &mut Context<'a>,
    ) -> Option<(Spanned<&'a str>, Spanned<ast::Symbol<'a>>)> {
        match &self.0.0[..] {
            [
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(state)), state_span),
                Spanned(ast::Item::Symbol(letter), letter_span),
            ] => {
                return Some((Spanned(state, *state_span), Spanned(*letter, *letter_span)));
            }
            _ => ctx.emit_error("expected FA transition function (ident, ident|~)", self.1),
        }
        None
    }
}
