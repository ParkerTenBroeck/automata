use std::collections::HashSet;

use super::*;

use crate::{delta_lower, dual_struct_serde, gamma_upper, loader::{
    Context, INITIAL_STACK, INITIAL_STATE, Spanned, ast::{self, Symbol as Sym}, log::LogSink
}, sigma_upper};

dual_struct_serde! {
    #[derive(Debug, PartialEq, Eq, Clone, Hash)]
    pub struct TransitionFrom<'a> {
        #[serde(borrow)]
        pub state: State<'a>,
        #[serde(borrow)]
        pub letter: Option<Letter<'a>>,
        #[serde(borrow)]
        pub symbol: Symbol<'a>,
    }
}

dual_struct_serde! {
    #[derive(Debug, PartialEq, Eq, Clone, Hash)]
    pub struct TransitionTo<'a> {
        #[serde(borrow)]
        pub state: State<'a>,
        #[serde(borrow)]
        pub stack: Vec<Symbol<'a>>,

        pub transition: Span,
        pub function: Span,
    }
}

dual_struct_serde! { {#[serde_with::serde_as]}
    #[derive(Clone, Debug)]
    pub struct Pda<'a> {
        #[serde(borrow)]
        pub initial_state: State<'a>,
        #[serde(borrow)]
        pub initial_stack: Symbol<'a>,
        #[serde(borrow)]
        pub states: HashMap<State<'a>, StateInfo>,
        #[serde(borrow)]
        pub symbols: HashMap<Symbol<'a>, SymbolInfo>,
        #[serde(borrow)]
        pub alphabet: HashMap<Letter<'a>, LetterInfo>,

        #[serde(borrow)]
        pub final_states: Option<HashMap<State<'a>, StateInfo>>,

        #[serde(borrow)]
        #[serde_as(as = "serde_with::Seq<(_, _)>")]
        pub transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>>,
    }
}

impl<'a> Pda<'a> {
    pub fn compile(
        items: impl Iterator<Item = Spanned<ast::TopLevel<'a>>>,
        ctx: &mut Context<'a>,
        options: Options,
    ) -> Option<Pda<'a>> {
        let mut initial_state = None;
        let mut initial_stack = None;

        let mut states = HashMap::new();
        let mut symbols = HashMap::new();
        let mut alphabet = HashMap::new();
        let mut final_states = None;

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
                TL::Item(S(sigma_upper!(pat), _), list) => {
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
                    if final_states.is_some() {
                        ctx.emit_error("final states already set", span);
                    }
                    let mut map = HashMap::new();
                    let Some(list) = list.expect_set(ctx) else {
                        continue;
                    };
                    for item in list {
                        let Some(ident) = item.expect_ident(ctx) else {
                            continue;
                        };
                        if states.contains_key(&State(ident)) {
                            if map
                                .insert(State(ident), StateInfo { definition: item.1 })
                                .is_some()
                            {
                                ctx.emit_error("final state redefined", item.1);
                            }
                        } else {
                            ctx.emit_error("final state not defined in set of states", item.1);
                        }
                    }
                    final_states = Some(map);
                }
                TL::Item(S(gamma_upper!(pat), _), list) => {
                    if !symbols.is_empty() {
                        ctx.emit_error("stack symbols already set", span);
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
                            ctx.emit_error("stack symbol redefined", item.1);
                        }
                    }

                    if list.is_empty() {
                        ctx.emit_error("stack symbols cannot be empty", span);
                    }
                }
                TL::Item(S(INITIAL_STATE, _), S(src, src_d)) => match src {
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
                TL::Item(S(INITIAL_STACK, _), S(src, src_d)) => match src {
                    ast::Item::Symbol(Sym::Ident(ident)) => {
                        if initial_stack.is_some() {
                            ctx.emit_error("initial stack already set", span);
                        }
                        if symbols.contains_key(&Symbol(ident)) {
                            initial_stack = Some(Symbol(ident));
                        } else {
                            ctx.emit_error(
                                "initial stack symbol not defined as a stack symbol",
                                src_d,
                            );
                        }
                    }
                    _ => _ = ctx.emit_error("expected ident", src_d),
                },
                TL::Item(S(name, dest_s), _) => {
                    ctx.emit_error(format!("unknown item {name:?}, expected states, alphabet, symbols, final states, initial state, initial stack"), dest_s);
                }

                TL::TransitionFunc(S((S(delta_lower!(pat), _), tuple), _), list) => {
                    let list = list.set_weak();
                    let Some((state, letter, stack_symbol)) =
                        tuple.as_ref().expect_pda_transition_function(ctx)
                    else {
                        continue;
                    };
                    if !states.contains_key(&State(state.0)) {
                        ctx.emit_error("transition state not defined as state", state.1);
                        continue;
                    };
                    if !symbols.contains_key(&Symbol(stack_symbol.0)) {
                        ctx.emit_error(
                            "transition stack symbol not defined as stack symbol",
                            stack_symbol.1,
                        );
                        continue;
                    };

                    let letter: Option<Letter<'_>> = match letter.0 {
                        Sym::Epsilon(_) => {
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
                        let Some((next_state, stack)) = item
                            .expect_tuple(ctx)
                            .and_then(|item| item.expect_pda_transition(ctx))
                        else {
                            continue;
                        };

                        if !states.contains_key(&State(next_state.0)) {
                            ctx.emit_error("transition state not defined as state", next_state.1);
                            continue;
                        };

                        let stack: Vec<_> = stack
                            .iter()
                            .rev()
                            .filter_map(|symbol| {
                                if matches!(symbol.0, ast::Item::Symbol(Sym::Epsilon(_))) {
                                    return None;
                                }
                                let ident = symbol.expect_ident(ctx)?;

                                if !symbols.contains_key(&Symbol(ident)) {
                                    ctx.emit_error("transition stack symbol not defined", symbol.1);
                                    return None;
                                };
                                Some(Symbol(ident))
                            })
                            .collect();

                        let entry: &mut _ = transitions
                            .entry(TransitionFrom {
                                letter,
                                state: State(state.0),
                                symbol: Symbol(stack_symbol.0),
                            })
                            .or_default();
                        if !entry.is_empty() && !options.non_deterministic {
                            ctx.emit_error("transition already defined for this starting point (non determinism not permitted)", item.1);
                        }
                        if !entry.insert(TransitionTo {
                            state: State(next_state.0),
                            stack,

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
                if symbols.contains_key(&Symbol("Z0")) {
                    ctx.emit_warning_locless(
                        "initial stack symbol not defined, defaulting to 'Z0'",
                    );
                } else {
                    ctx.emit_error_locless("initial stack symbol not defined");
                }
                Symbol("Z0")
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

        Some(Pda {
            initial_state,
            initial_stack,
            states,
            symbols,
            alphabet,
            final_states,
            transitions,
        })
    }
}

impl<'a, 'b> Spanned<&'b ast::Tuple<'a>> {
    fn expect_pda_transition_function(
        &self,
        ctx: &mut Context<'a>,
    ) -> Option<(Spanned<&'a str>, Spanned<ast::Symbol<'a>>, Spanned<&'a str>)> {
        match &self.0.0[..] {
            [
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(state)), state_span),
                Spanned(ast::Item::Symbol(letter), letter_span),
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(symbol)), symbol_span),
            ] => {
                return Some((
                    Spanned(state, *state_span),
                    Spanned(*letter, *letter_span),
                    Spanned(symbol, *symbol_span),
                ));
            }
            _ => _ = ctx.emit_error(
                "expected PDA transition function (state, letter|epsilon, symbol)",
                self.1,
            ),
        }
        None
    }
    fn expect_pda_transition(
        &self,
        ctx: &mut Context<'a>,
    ) -> Option<(Spanned<&'a str>, &'b [Spanned<ast::Item<'a>>])> {
        match &self.0.0[..] {
            [
                Spanned(ast::Item::Symbol(ast::Symbol::Ident(state)), state_span),
                list,
            ] => {
                return Some((Spanned(state, *state_span), list.list_weak()));
            }
            _ => _ = ctx.emit_error("expected PDA transition (state, symbol|[symbol])", self.1),
        }
        None
    }
}
