use std::collections::HashSet;

use super::*;

use crate::{
    delta_lower, dual_struct_serde,
    loader::{
        BLANK_SYMBOL, Context, INITIAL_STATE, Spanned,
        ast::{self, Symbol as Sym},
        log::LogSink,
    },
};
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
        pub blank_symbol: Symbol<'a>,
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
        TmCompiler::new(ctx, options).compile(items)
    }
}

pub struct TmCompiler<'a, 'b> {
    ctx: &'b mut Context<'a>,
    options: Options,

    initial_state: Option<(State<'a>, Span)>,
    blank_symbol: Option<(Symbol<'a>, Span)>,

    states: HashMap<State<'a>, StateInfo>,
    states_def: Option<Span>,

    symbols: HashMap<Symbol<'a>, SymbolInfo>,
    symbols_def: Option<Span>,

    final_states: HashMap<State<'a>, StateInfo>,
    final_states_def: Option<Span>,

    transitions: HashMap<TransitionFrom<'a>, HashSet<TransitionTo<'a>>>,
}

impl<'a, 'b> TmCompiler<'a, 'b> {
    pub fn new(ctx: &'b mut Context<'a>, options: Options) -> Self {
        Self {
            ctx,
            options,

            initial_state: Default::default(),
            blank_symbol: Default::default(),
            states: Default::default(),
            states_def: Default::default(),
            symbols: Default::default(),
            symbols_def: Default::default(),
            final_states: Default::default(),
            final_states_def: Default::default(),
            transitions: Default::default(),
        }
    }

    pub fn compile(
        mut self,
        items: impl Iterator<Item = Spanned<ast::TopLevel<'a>>>,
    ) -> Option<Tm<'a>> {
        for Spanned(element, span) in items {
            self.compile_top_level(element, span);
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
                        .emit_help_logless(format!("add: {BLANK_SYMBOL} = ..."));
                }
                State("q0")
            }
        };

        let blank_symbol = match self.blank_symbol {
            Some(some) => some.0,
            None => {
                if self.symbols.contains_key(&Symbol("B")) {
                    self.ctx
                        .emit_warning_locless("blank symbol not defined, defaulting to 'B'")
                        .emit_help_logless(format!("add: {BLANK_SYMBOL} = B"));
                } else {
                    self.ctx
                        .emit_error_locless("blank symbol not defined")
                        .emit_help_logless(format!("add: {BLANK_SYMBOL} = ..."));
                }
                Symbol("B")
            }
        };

        if self.transitions.is_empty() {
            self.ctx
                .emit_warning_locless("no transitions defined")
                .emit_help_logless(
                    "consider defining one: d(state, symbol) = (state, symbol, direction) | {(state, symbol, direction), ...}",
                )
                .emit_info_logless(concat!("d can be ", delta_lower!(str)));
        }

        if self.ctx.contains_errors() {
            return None;
        }

        Some(Tm {
            initial_state,
            blank_symbol,
            states: self.states,
            symbols: self.symbols,
            final_states: self.final_states,
            transitions: self.transitions,
        })
    }

    fn compile_top_level(&mut self, element: ast::TopLevel<'a>, span: Span) {
        use Spanned as S;
        use ast::TopLevel as TL;
        match element {
            TL::Item(S("Q", _), list) => self.compile_states(list, span),
            TL::Item(S(delta_lower!(pat), _), list) => self.compile_symbols(list, span),
            TL::Item(S("F", _), list) => self.compile_final_states(list, span),
            TL::Item(S(INITIAL_STATE, _), item) => self.compile_initial_state(item, span),
            TL::Item(S(BLANK_SYMBOL, _), item) => self.compile_blank_symbol(item, span),
            TL::Item(S(name, dest_s), _) => {
                self.ctx.emit_error(format!("unknown item {name:?}, expected states, symbols, final states, initial state, blank symbol"), dest_s);
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

    fn compile_symbols(&mut self, list: Spanned<ast::Item<'a>>, top_level: Span) {
        if let Some(previous) = self.symbols_def {
            self.ctx
                .emit_error("stack symbols already set", top_level)
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
                .symbols
                .insert(Symbol(ident), SymbolInfo { definition: item.1 })
            {
                self.ctx
                    .emit_error("stack symbol redefined", item.1)
                    .emit_info("previously defined here", previous.definition);
            }
        }

        if list.is_empty() {
            self.ctx.emit_error("states cannot be empty", top_level);
        }
        self.symbols_def = Some(top_level);
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

    fn compile_blank_symbol(
        &mut self,
        Spanned(src, src_d): Spanned<ast::Item<'a>>,
        top_level: Span,
    ) {
        match src {
            ast::Item::Symbol(Sym::Ident(ident)) => {
                if let Some((_, previous)) = self.blank_symbol {
                    self.ctx
                        .emit_error("blank symbol already set", top_level)
                        .emit_help("previously defined here", previous);
                }
                if self.states.contains_key(&State(ident)) {
                    self.blank_symbol = Some((Symbol(ident), top_level))
                } else {
                    self.ctx
                        .emit_error("blank symbol not defined as a state", src_d);
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
        let Some((from_state, from_tape)) = args.as_ref().expect_tm_transition_function(self.ctx)
        else {
            return;
        };
        if !self.states.contains_key(&State(from_state.0)) {
            self.ctx
                .emit_error("transition state not defined as state", from_state.1);
            return;
        };
        if !self.symbols.contains_key(&Symbol(from_tape.0)) {
            self.ctx.emit_error(
                "transition tape symbol not defined as tape symbol",
                from_tape.1,
            );
            return;
        };

        for item in list {
            let Some((to_state, to_tape, direction)) = item
                .expect_tuple(self.ctx)
                .and_then(|item| item.expect_tm_transition(self.ctx))
            else {
                continue;
            };

            if !self.states.contains_key(&State(to_state.0)) {
                self.ctx
                    .emit_error("transition state not defined as state", to_state.1);
                continue;
            };

            let entry: &mut _ = self
                .transitions
                .entry(TransitionFrom {
                    state: State(from_state.0),
                    symbol: Symbol(from_tape.0),
                })
                .or_default();
            if !entry.is_empty() && !self.options.non_deterministic {
                self.ctx.emit_error("transition already defined for this starting point (non determinism not permitted)", item.1);
            }
            if !entry.insert(TransitionTo {
                state: State(to_state.0),
                symbol: Symbol(to_tape.0),
                direction: direction.0,

                function: args.1,
                transition: item.1,
            }) {
                self.ctx.emit_warning("duplicate transition", item.1);
            }
        }
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
            _ => {
                _ = ctx.emit_error(
                    "expected TM transition function (state, symbol, direction)",
                    self.1,
                )
            }
        }
        None
    }
}
