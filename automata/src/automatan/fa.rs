use std::collections::HashSet;

use super::*;

use crate::{
    delta_lower, dual_struct_serde, epsilon,
    loader::{
        Context, INITIAL_STATE, Spanned,
        ast::{self, Symbol as Sym, TopLevel},
        log::LogSink,
    },
    sigma_upper,
};

dual_struct_serde! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct TransitionFrom<'a> {
        #[serde(borrow)]
        pub state: State<'a>,
        pub letter: Option<Letter<'a>>,
    }
}

dual_struct_serde! {
    #[derive(Debug, PartialEq, Eq, Clone, Hash)]
    pub struct TransitionTo<'a> {
        #[serde(borrow)]
        pub state: State<'a>,

        pub transition: Span,
        pub function: Span,
    }
}

dual_struct_serde! { {#[serde_with::serde_as]}
    #[derive(Clone, Debug)]
    pub struct Fa<'a> {
        #[serde(borrow)]
        pub initial_state: State<'a>,

        #[serde(borrow)]
        pub states: HashMap<State<'a>, StateInfo>,

        #[serde(borrow)]
        pub alphabet: HashMap<Letter<'a>, LetterInfo>,

        #[serde(borrow)]
        pub final_states: HashMap<State<'a>, StateInfo>,

        #[serde(borrow)]
        #[serde_as(as = "serde_with::Seq<(_, _)>")]
        pub transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>>,
    }
}

impl<'a> Fa<'a> {
    pub fn compile(
        items: impl Iterator<Item = Spanned<ast::TopLevel<'a>>>,
        ctx: &mut Context<'a>,
        options: Options,
    ) -> Option<Fa<'a>> {
        FaCompiler::new(ctx, options).compile(items)
    }
}

pub struct FaCompiler<'a, 'b> {
    ctx: &'b mut Context<'a>,
    options: Options,

    initial_state: Option<(State<'a>, Span)>,

    states: HashMap<State<'a>, StateInfo>,
    states_def: Option<Span>,

    alphabet: HashMap<Letter<'a>, LetterInfo>,
    alphabet_def: Option<Span>,

    final_states: HashMap<State<'a>, StateInfo>,
    final_states_def: Option<Span>,

    transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>>,
}

impl<'a, 'b> FaCompiler<'a, 'b> {
    pub fn new(ctx: &'b mut Context<'a>, options: Options) -> Self {
        Self {
            ctx,
            options,

            initial_state: Default::default(),
            states: Default::default(),
            states_def: Default::default(),
            alphabet: Default::default(),
            alphabet_def: Default::default(),
            final_states: Default::default(),
            final_states_def: Default::default(),
            transitions: Default::default(),
        }
    }

    pub fn compile(
        mut self,
        items: impl Iterator<Item = Spanned<ast::TopLevel<'a>>>,
    ) -> Option<Fa<'a>> {
        for Spanned(element, span) in items {
            self.compile_top_level(element, span);
        }

        if self.states_def.is_none() {
            self.ctx
                .emit_error_locless("states never defined")
                .emit_help_logless("add: Q = {...}");
        }

        if self.alphabet_def.is_none() {
            self.ctx
                .emit_error_locless("alphabet never defined")
                .emit_help_logless("add: E = {...}")
                .emit_info_logless(concat!("E can be ", sigma_upper!(str)));
        }

        if self.final_states_def.is_none() {
            self.ctx
                .emit_error_locless("final states never defined")
                .emit_help_logless("add: F = {...}");
        }

        let initial_state = match self.initial_state {
            Some(some) => some.0,
            None => {
                if self.states.contains_key(&State("q0")) {
                    self.ctx
                        .emit_warning_locless("initial state not defined, defaulting to 'q0'")
                        .emit_help_logless(format!("add: {INITIAL_STATE} = q0"));
                } else {
                    self.ctx
                        .emit_error_locless("initial state not defined")
                        .emit_help_logless(format!("add: {INITIAL_STATE} = ..."));
                }
                State("q0")
            }
        };

        if self.transitions.is_empty() {
            self.ctx
                .emit_warning_locless("no transitions defined")
                .emit_help_logless(
                    "consider defining one: d(state, letter|epsilon) = state | {state, ...}",
                )
                .emit_info_logless(concat!("d can be ", delta_lower!(str)))
                .emit_info_logless(concat!("epsilon can be ", epsilon!(str)));
        }

        if self.ctx.contains_errors() {
            return None;
        }

        Some(Fa {
            initial_state,
            states: self.states,
            alphabet: self.alphabet,
            final_states: self.final_states,
            transitions: self.transitions,
        })
    }

    fn compile_top_level(&mut self, element: TopLevel<'a>, span: Span) {
        use Spanned as S;
        use ast::TopLevel as TL;
        match element {
            TL::Item(S("Q", _), list) => self.compile_states(list, span),
            TL::Item(S(sigma_upper!(pat), _), list) => self.compile_alphabet(list, span),
            TL::Item(S("F", _), list) => self.compile_final_states(list, span),
            TL::Item(S(INITIAL_STATE, _), item) => self.compile_initial_state(item, span),
            TL::Item(S(name, dest_s), _) => {
                self.ctx.emit_error(format!("unknown item {name:?}, expected states, alphabet, final states, initial state"), dest_s);
            }

            TL::TransitionFunc(S((S(delta_lower!(pat), _), args), _), list) => {
                self.compile_transition_function(args, list)
            }
            TL::TransitionFunc(S((S(name, _), _), dest_s), _) => {
                self.ctx.emit_error(
                    format!(
                        "unknown function {name:?}, expected transition function ( {} )",
                        delta_lower!(str)
                    ),
                    dest_s,
                );
            }

            TL::ProductionRule(_, _) => {
                self.ctx.emit_error("unexpected production rule", span);
            }
            TL::Table() => _ = self.ctx.emit_error("unexpected table", span),
        }
    }

    fn compile_states(&mut self, list: Spanned<ast::Item<'a>>, top_level: Span) {
        if let Some(previous) = self.states_def {
            self.ctx
                .emit_error("states already set", top_level)
                .emit_info("previously defined here", previous);
        }
        let Some(list) = list.expect_set(self.ctx) else {
            return;
        };
        for item in list {
            let Some(ident) = item.expect_ident(self.ctx) else {
                continue;
            };
            if let Some(previous) = self
                .states
                .insert(State(ident), StateInfo { definition: item.1 })
            {
                self.ctx
                    .emit_error("state redefined", item.1)
                    .emit_info("previously defined here", previous.definition);
            }
        }

        if list.is_empty() {
            self.ctx.emit_error("states cannot be empty", top_level);
        }
        self.states_def = Some(top_level);
    }

    fn compile_alphabet(&mut self, list: Spanned<ast::Item<'a>>, top_level: Span) {
        if let Some(previous) = self.alphabet_def {
            self.ctx
                .emit_error("alphabet already set", top_level)
                .emit_info("previously defined here", previous);
        }
        let Some(list) = list.expect_set(self.ctx) else {
            return;
        };
        for item in list {
            let Some(ident) = item.expect_ident(self.ctx) else {
                continue;
            };

            if ident.chars().count() != 1 {
                self.ctx
                    .emit_error("letter cannot be longer than one char", item.1);
            }

            if let Some(previous) = self
                .alphabet
                .insert(Letter(ident), LetterInfo { definition: item.1 })
            {
                self.ctx
                    .emit_error("letter redefined", item.1)
                    .emit_help("previously defined here", previous.definition);
            }
        }
        if list.is_empty() {
            self.ctx.emit_error("alphabet cannot be empty", top_level);
        }
        self.alphabet_def = Some(top_level);
    }

    fn compile_final_states(&mut self, list: Spanned<ast::Item<'a>>, top_level: Span) {
        if let Some(previous) = self.final_states_def {
            self.ctx
                .emit_error("final states already set", top_level)
                .emit_help("previously defined here", previous);
        }
        let Some(list) = list.expect_set(self.ctx) else {
            return;
        };
        for item in list {
            let Some(ident) = item.expect_ident(self.ctx) else {
                continue;
            };
            if self.states.contains_key(&State(ident)) {
                if self
                    .final_states
                    .insert(State(ident), StateInfo { definition: item.1 })
                    .is_some()
                {
                    self.ctx.emit_error("final state redefined", item.1);
                }
            } else {
                self.ctx
                    .emit_error("final state not defined in set of states", item.1);
            }
        }
        self.final_states_def = Some(top_level);
    }

    fn compile_initial_state(
        &mut self,
        Spanned(src, src_d): Spanned<ast::Item<'a>>,
        top_level: Span,
    ) {
        match src {
            ast::Item::Symbol(Sym::Ident(ident)) => {
                if let Some((_, previous)) = self.initial_state {
                    self.ctx
                        .emit_error("initial state already set", top_level)
                        .emit_help("previously defined here", previous);
                }
                if self.states.contains_key(&State(ident)) {
                    self.initial_state = Some((State(ident), top_level))
                } else {
                    self.ctx
                        .emit_error("initial state symbol not defined as a state", src_d);
                }
            }
            _ => _ = self.ctx.emit_error("expected ident", src_d),
        }
    }
    fn compile_transition_function(
        &mut self,
        args: Spanned<ast::Tuple<'a>>,
        list: Spanned<ast::Item<'a>>,
    ) {
        let list = list.set_weak();
        let Some((state, letter)) = args.as_ref().expect_fa_transition_function(self.ctx) else {
            return;
        };
        if !self.states.contains_key(&State(state.0)) {
            self.ctx
                .emit_error("transition state not defined as state", state.1);
            return;
        };

        let letter: Option<Letter<'_>> = match letter.0 {
            Sym::Epsilon(_) => {
                if !self.options.epsilon_moves {
                    self.ctx.emit_error("epsilon moves not permitted", letter.1);
                }
                None
            }
            Sym::Ident(val) => {
                if !self.alphabet.contains_key(&Letter(val)) {
                    self.ctx
                        .emit_error("transition letter not defined in alphabet", letter.1);
                }
                Some(Letter(val))
            }
        };

        for item in list {
            let Some(next_state) = item.expect_ident(self.ctx) else {
                continue;
            };
            let next_state = Spanned(next_state, item.1);

            if !self.states.contains_key(&State(next_state.0)) {
                self.ctx
                    .emit_error("transition state not defined as state", next_state.1);
                continue;
            };

            let entry: &mut _ = self
                .transitions
                .entry(TransitionFrom {
                    letter,
                    state: State(state.0),
                })
                .or_default();
            if let Some(entry) = entry.iter().next()
                && !self.options.non_deterministic
            {
                self.ctx.emit_error("transition already defined for this starting point (non determinism not permitted)", item.1)
                            .emit_info("previously defined here", entry.transition);
            }
            if let Some(previous) = entry.replace(TransitionTo {
                state: State(next_state.0),

                function: args.1,
                transition: item.1,
            }) {
                self.ctx
                    .emit_warning("duplicate transition", item.1)
                    .emit_info("previously defined here", previous.transition);
            }
        }
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
            _ => {
                _ = ctx.emit_error(
                    "expected FA transition function (state, letter|epsilon)",
                    self.1,
                )
            }
        }
        None
    }
}
